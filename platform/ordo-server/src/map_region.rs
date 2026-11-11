//! The map region the server offers its devices (MAP-8, ADR-0057): cut from a PMTiles world
//! map, by URL or from a file, around a centre with a radius. One cut at a time; the new
//! file replaces the old one only when complete.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ordo_sync::message::{MapRegion, MapRegionRequest, MapRegionState};
use pmtiles::{
    AsyncBackend, AsyncPmTilesReader, Compression, HashMapCache, PmTilesWriter, TileCoord, TileId,
};
use serde::{Deserialize, Serialize};

/// Radius limits in metres.
pub const RADIUS_M: std::ops::RangeInclusive<u32> = 1_000..=50_000;
/// Tiles fetched at the same time.
const PARALLEL: usize = 16;
/// Metres per degree of latitude.
const M_PER_DEGREE: f64 = 111_320.0;

/// What the server remembers of the region next to its file.
#[derive(Serialize, Deserialize)]
struct Remembered {
    area: MapRegionRequest,
    at: u64,
}

/// The map region and the cutting of a new one.
pub struct MapJob {
    file: PathBuf,
    source: Option<String>,
    state: Mutex<MapRegion>,
    done: AtomicU64,
}

impl MapJob {
    /// The job for the region file `file`, cut from `source` (a URL or a path; none: the
    /// Protomaps build of the day before).
    pub fn new(file: PathBuf, source: Option<String>) -> Self {
        let state = Self::on_disk(&file);
        Self {
            file,
            source,
            state: Mutex::new(state),
            done: AtomicU64::new(0),
        }
    }

    fn sidecar(file: &Path) -> PathBuf {
        file.with_extension("json")
    }

    /// The region on disk, if any.
    fn on_disk(file: &Path) -> MapRegion {
        let Ok(meta) = std::fs::metadata(file) else {
            return MapRegion::default();
        };
        let remembered: Option<Remembered> = std::fs::read(Self::sidecar(file))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok());
        MapRegion {
            state: MapRegionState::Ready,
            bytes: Some(meta.len()),
            area: remembered.as_ref().map(|r| r.area),
            at: remembered.map(|r| r.at),
            ..MapRegion::default()
        }
    }

    // [impl->dsn~map-region~1]
    /// The current state, with the progress of a running cut.
    pub fn state(&self) -> MapRegion {
        let mut state = self.state.lock().expect("map job lock").clone();
        if state.state == MapRegionState::Running {
            state.done = Some(self.done.load(Ordering::Relaxed));
        }
        state
    }

    /// Starts cutting `area`, unless a cut is running; returns the state.
    pub fn start(self: &Arc<Self>, area: MapRegionRequest, now_ms: u64) -> MapRegion {
        {
            let mut state = self.state.lock().expect("map job lock");
            if state.state == MapRegionState::Running {
                return state.clone();
            }
            if !RADIUS_M.contains(&area.radius_m)
                || !(-850_000_000..=850_000_000).contains(&area.lat_e7)
                || !(-1_800_000_000..=1_800_000_000).contains(&area.lon_e7)
            {
                *state = MapRegion {
                    state: MapRegionState::Failed,
                    error: Some("centre or radius out of range".into()),
                    ..Self::on_disk(&self.file)
                };
                state.state = MapRegionState::Failed;
                return state.clone();
            }
            *state = MapRegion {
                state: MapRegionState::Running,
                area: Some(area),
                done: Some(0),
                total: Some(0),
                ..MapRegion::default()
            };
        }
        self.done.store(0, Ordering::Relaxed);
        let job = self.clone();
        tokio::spawn(async move {
            let result = job.cut(area).await;
            let mut state = job.state.lock().expect("map job lock");
            *state = match result {
                Ok(()) => {
                    let remembered = Remembered { area, at: now_ms };
                    let _ = std::fs::write(
                        Self::sidecar(&job.file),
                        serde_json::to_vec(&remembered).unwrap_or_default(),
                    );
                    tracing::info!(file = %job.file.display(), "map region cut");
                    Self::on_disk(&job.file)
                }
                Err(error) => {
                    tracing::warn!(%error, "cutting the map region failed");
                    MapRegion {
                        state: MapRegionState::Failed,
                        error: Some(error),
                        ..Self::on_disk(&job.file)
                    }
                }
            };
        });
        self.state()
    }

    /// The source: as configured, else the Protomaps build of the day before.
    fn source(&self) -> String {
        self.source.clone().unwrap_or_else(|| {
            let day = time::OffsetDateTime::now_utc() - time::Duration::days(1);
            format!(
                "https://build.protomaps.com/{:04}{:02}{:02}.pmtiles",
                day.year(),
                u8::from(day.month()),
                day.day()
            )
        })
    }

    async fn cut(self: &Arc<Self>, area: MapRegionRequest) -> Result<(), String> {
        let source = self.source();
        if source.starts_with("http://") || source.starts_with("https://") {
            // TLS with ring, like the rest of the server.
            let _ = rustls::crypto::ring::default_provider().install_default();
            let client = pmtiles::reqwest::Client::builder()
                .user_agent(concat!("ORDO/", env!("CARGO_PKG_VERSION")))
                .build()
                .map_err(|e| e.to_string())?;
            let reader = AsyncPmTilesReader::new_with_url_and_cache(
                client,
                &source,
                HashMapCache::default(),
            )
            .await
            .map_err(|e| format!("{source}: {e}"))?;
            self.cut_from(reader, area).await
        } else {
            let reader =
                AsyncPmTilesReader::new_with_path_and_cache(&source, HashMapCache::default())
                    .await
                    .map_err(|e| format!("{source}: {e}"))?;
            self.cut_from(reader, area).await
        }
    }

    async fn cut_from<B: AsyncBackend + Send + Sync + 'static>(
        self: &Arc<Self>,
        reader: AsyncPmTilesReader<B, HashMapCache>,
        area: MapRegionRequest,
    ) -> Result<(), String> {
        let (tile_type, tile_compression, min_zoom, max_zoom) = {
            let h = reader.get_header();
            (h.tile_type, h.tile_compression, h.min_zoom, h.max_zoom)
        };
        let metadata = reader.get_metadata().await.map_err(|e| e.to_string())?;
        let coords = tiles(area, min_zoom, max_zoom);
        if let Ok(mut state) = self.state.lock() {
            state.total = Some(coords.len() as u64);
        }
        let reader = Arc::new(reader);
        let limit = Arc::new(tokio::sync::Semaphore::new(PARALLEL));
        let mut tasks = tokio::task::JoinSet::new();
        for coord in coords {
            let (reader, limit, job) = (reader.clone(), limit.clone(), self.clone());
            tasks.spawn(async move {
                let _permit = limit.acquire().await;
                let tile = reader.get_tile(coord).await;
                job.done.fetch_add(1, Ordering::Relaxed);
                tile.map(|t| (coord, t))
            });
        }
        let mut found = Vec::new();
        while let Some(result) = tasks.join_next().await {
            let (coord, tile) = result
                .map_err(|e| e.to_string())?
                .map_err(|e| e.to_string())?;
            if let Some(tile) = tile {
                found.push((TileId::from(coord).value(), coord, tile));
            }
        }
        found.sort_by_key(|(id, _, _)| *id);

        // Written next to the region, then moved over it: devices never get half a file.
        if let Some(dir) = self.file.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let partial = self.file.with_extension("partial");
        let output = std::fs::File::create(&partial).map_err(|e| e.to_string())?;
        let (min_lon, min_lat, max_lon, max_lat) = bounds(area);
        let mut writer = PmTilesWriter::new(tile_type)
            .precompressed(if tile_compression == Compression::Unknown {
                Compression::None
            } else {
                tile_compression
            })
            .min_zoom(min_zoom)
            .max_zoom(max_zoom)
            .bounds(min_lon, min_lat, max_lon, max_lat)
            .center(f64::from(area.lon_e7) / 1e7, f64::from(area.lat_e7) / 1e7)
            .center_zoom(max_zoom.min(13))
            .metadata(metadata)
            .create(std::io::BufWriter::new(output))
            .map_err(|e| e.to_string())?;
        for (_, coord, tile) in &found {
            writer.add_tile(*coord, tile).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
        std::fs::rename(&partial, &self.file).map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// The square around the circle: west, south, east, north in degrees.
fn bounds(area: MapRegionRequest) -> (f64, f64, f64, f64) {
    let (lat, lon) = (f64::from(area.lat_e7) / 1e7, f64::from(area.lon_e7) / 1e7);
    let r = f64::from(area.radius_m);
    let dlat = r / M_PER_DEGREE;
    let dlon = r / (M_PER_DEGREE * lat.to_radians().cos().max(0.01));
    (
        (lon - dlon).max(-180.0),
        (lat - dlat).max(-85.051_128),
        (lon + dlon).min(180.0),
        (lat + dlat).min(85.051_128),
    )
}

/// Web Mercator tile of a point at zoom `z`.
fn tile_of(lon: f64, lat: f64, z: u8) -> (u32, u32) {
    let n = f64::from(1u32 << z);
    let x = ((lon + 180.0) / 360.0 * n).floor();
    let lat = lat.to_radians();
    let y = ((1.0 - (lat.tan() + 1.0 / lat.cos()).ln() / std::f64::consts::PI) / 2.0 * n).floor();
    let clamp = |v: f64| v.clamp(0.0, n - 1.0) as u32;
    (clamp(x), clamp(y))
}

/// Every tile from `min_zoom` to `max_zoom` that touches the square around the circle.
fn tiles(area: MapRegionRequest, min_zoom: u8, max_zoom: u8) -> Vec<TileCoord> {
    let (west, south, east, north) = bounds(area);
    (min_zoom..=max_zoom)
        .flat_map(|z| {
            let (x0, y0) = tile_of(west, north, z);
            let (x1, y1) = tile_of(east, south, z);
            (x0..=x1).flat_map(move |x| (y0..=y1).filter_map(move |y| TileCoord::new(z, x, y).ok()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(lat: f64, lon: f64, radius_m: u32) -> MapRegionRequest {
        MapRegionRequest {
            lat_e7: (lat * 1e7) as i32,
            lon_e7: (lon * 1e7) as i32,
            radius_m,
        }
    }

    // [utest->dsn~map-region~1]
    #[test]
    fn the_tiles_cover_the_circle_at_every_zoom() {
        // Augsburg, 10 km.
        let a = area(48.37, 10.9, 10_000);
        let (w, s, e, n) = bounds(a);
        assert!((n - s - 2.0 * 10_000.0 / M_PER_DEGREE).abs() < 1e-9);
        assert!(e - w > n - s, "a degree of longitude is shorter here");
        let tiles = tiles(a, 0, 15);
        assert_eq!(tiles.iter().filter(|t| t.z() == 0).count(), 1);
        let z15: Vec<_> = tiles.iter().filter(|t| t.z() == 15).collect();
        // About 20 km × 20 km of tiles of roughly 0.8 km: some hundreds, not thousands.
        assert!((400..=900).contains(&z15.len()), "{}", z15.len());
        // The centre's tile is among them.
        let (x, y) = tile_of(10.9, 48.37, 15);
        assert!(z15.iter().any(|t| t.x() == x && t.y() == y));
        // Each zoom level once per tile, no duplicates.
        let mut ids: Vec<u64> = tiles.iter().map(|t| TileId::from(*t).value()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), tiles.len());
    }

    /// Cuts 1 km around Augsburg from the real Protomaps build (needs internet; run with
    /// `cargo test -p ordo-server -- --ignored protomaps`).
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "downloads from build.protomaps.com"]
    async fn cuts_a_small_region_from_the_protomaps_build() {
        let dir = std::env::temp_dir().join(format!("ordo-map-{}", std::process::id()));
        let job = Arc::new(MapJob::new(dir.join("region.pmtiles"), None));
        let started = std::time::Instant::now();
        job.start(area(48.37, 10.9, 1_000), 0);
        while job.state().state == MapRegionState::Running {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        let state = job.state();
        println!("{state:?} in {:?}", started.elapsed());
        assert_eq!(state.state, MapRegionState::Ready, "{state:?}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_square_stays_on_the_map() {
        let (w, s, e, n) = bounds(area(85.0, 179.9, 50_000));
        assert!(w >= -180.0 && e <= 180.0 && s >= -85.06 && n <= 85.06);
    }
}
