//! The map region of each incident (MAP-8, ADR-0057, ADR-0062): cut from a PMTiles world
//! map, by URL or from a file, around a centre with a radius, into
//! `<map dir>/<incident ULID>.pmtiles`. One cut at a time on a server; the new file replaces
//! the incident's old one only when complete. Devices read it with a ticket they ask for over
//! their sync connection, so a region reaches only the people of its incident.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::{Path as UrlPath, Request, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use ordo_core::{IncidentId, Ulid};
use ordo_sync::message::{MapRegion, MapRegionRequest, MapRegionState};
use pmtiles::{
    AsyncBackend, AsyncPmTilesReader, Compression, HashMapCache, PmTilesWriter, TileCoord, TileId,
};
use serde::{Deserialize, Serialize};

use crate::app::App;

/// Radius limits in metres.
pub const RADIUS_M: std::ops::RangeInclusive<u32> = 1_000..=50_000;
/// Tiles fetched at the same time.
const PARALLEL: usize = 16;
/// Metres per degree of latitude.
const M_PER_DEGREE: f64 = 111_320.0;
/// How long a ticket to read a region is valid (asked anew on every connection).
const TICKET_TTL: Duration = Duration::from_secs(12 * 3600);
/// Most tickets kept at once (old ones are dropped first).
const MAX_TICKETS: usize = 10_000;

/// What the server remembers of a region next to its file.
#[derive(Serialize, Deserialize)]
struct Remembered {
    area: MapRegionRequest,
    at: u64,
}

/// The regions of all incidents and the cutting of a new one.
pub struct MapJob {
    dir: PathBuf,
    source: Option<String>,
    /// The incident (ULID) of the running or last cut, and its state.
    current: Mutex<Option<(Ulid, MapRegion)>>,
    done: AtomicU64,
    /// Tickets to read a region: token → incident and expiry.
    tickets: Mutex<HashMap<String, (Ulid, Instant)>>,
}

impl MapJob {
    /// The regions in `dir`, cut from `source` (a URL or a path; none: the Protomaps build
    /// of the day before).
    pub fn new(dir: PathBuf, source: Option<String>) -> Self {
        Self {
            dir,
            source,
            current: Mutex::new(None),
            done: AtomicU64::new(0),
            tickets: Mutex::new(HashMap::new()),
        }
    }

    /// The region file of an incident.
    pub fn file(&self, incident: Ulid) -> PathBuf {
        self.dir.join(format!("{incident}.pmtiles"))
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

    // [impl->dsn~map-region~2]
    /// The state of an incident's region, with the progress of a running cut.
    pub fn state(&self, incident: &IncidentId) -> MapRegion {
        let id = incident.ulid();
        let current = self.current.lock().expect("map job lock").clone();
        let mut state = match current {
            Some((of, state)) if of == id && state.state != MapRegionState::Ready => {
                let mut state = state;
                if state.state == MapRegionState::Running {
                    state.done = Some(self.done.load(Ordering::Relaxed));
                }
                state
            }
            _ => Self::on_disk(&self.file(id)),
        };
        state.incident = Some(incident.clone());
        state
    }

    // [impl->dsn~map-region~2]
    /// Starts cutting `area` for `incident`, unless a cut is running on the server; returns
    /// the incident's state.
    pub fn start(
        self: &Arc<Self>,
        incident: &IncidentId,
        area: MapRegionRequest,
        now_ms: u64,
    ) -> MapRegion {
        let id = incident.ulid();
        let file = self.file(id);
        {
            let mut current = self.current.lock().expect("map job lock");
            if let Some((of, state)) = current.as_ref()
                && state.state == MapRegionState::Running
            {
                let same = *of == id;
                drop(current);
                if same {
                    return self.state(incident);
                }
                return MapRegion {
                    incident: Some(incident.clone()),
                    state: MapRegionState::Failed,
                    error: Some("another map region is being cut".into()),
                    ..Self::on_disk(&file)
                };
            }
            if !RADIUS_M.contains(&area.radius_m)
                || !(-850_000_000..=850_000_000).contains(&area.lat_e7)
                || !(-1_800_000_000..=1_800_000_000).contains(&area.lon_e7)
            {
                let state = MapRegion {
                    state: MapRegionState::Failed,
                    error: Some("centre or radius out of range".into()),
                    ..Self::on_disk(&file)
                };
                *current = Some((id, state));
                drop(current);
                return self.state(incident);
            }
            *current = Some((
                id,
                MapRegion {
                    state: MapRegionState::Running,
                    area: Some(area),
                    done: Some(0),
                    total: Some(0),
                    ..MapRegion::default()
                },
            ));
        }
        self.done.store(0, Ordering::Relaxed);
        let job = self.clone();
        tokio::spawn(async move {
            let result = job.cut(&file, area).await;
            let state = match result {
                Ok(()) => {
                    let remembered = Remembered { area, at: now_ms };
                    let _ = std::fs::write(
                        Self::sidecar(&file),
                        serde_json::to_vec(&remembered).unwrap_or_default(),
                    );
                    tracing::info!(file = %file.display(), "map region cut");
                    Self::on_disk(&file)
                }
                Err(error) => {
                    tracing::warn!(%error, "cutting the map region failed");
                    MapRegion {
                        state: MapRegionState::Failed,
                        error: Some(error),
                        ..Self::on_disk(&file)
                    }
                }
            };
            *job.current.lock().expect("map job lock") = Some((id, state));
        });
        self.state(incident)
    }

    // [impl->dsn~map-region~2]
    /// Issues a ticket to read an incident's region; the caller has checked that the user
    /// may read the incident.
    pub fn issue_ticket(&self, incident: &IncidentId) -> String {
        let mut token = [0u8; 32];
        getrandom::fill(&mut token).expect("the operating system provides randomness");
        let token = crate::blobs::hex(&token);
        let now = Instant::now();
        let mut tickets = self.tickets.lock().expect("ticket lock");
        tickets.retain(|_, (_, expires)| *expires > now);
        if tickets.len() >= MAX_TICKETS
            && let Some(oldest) = tickets
                .iter()
                .min_by_key(|(_, (_, expires))| *expires)
                .map(|(k, _)| k.clone())
        {
            tickets.remove(&oldest);
        }
        tickets.insert(token.clone(), (incident.ulid(), now + TICKET_TTL));
        token
    }

    /// Whether `token` is a valid ticket for the region of `incident`.
    fn ticket_allows(&self, token: &str, incident: Ulid) -> bool {
        self.tickets
            .lock()
            .expect("ticket lock")
            .get(token)
            .is_some_and(|(of, expires)| *of == incident && *expires > Instant::now())
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

    async fn cut(self: &Arc<Self>, file: &Path, area: MapRegionRequest) -> Result<(), String> {
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
            self.cut_from(reader, file, area).await
        } else {
            let reader =
                AsyncPmTilesReader::new_with_path_and_cache(&source, HashMapCache::default())
                    .await
                    .map_err(|e| format!("{source}: {e}"))?;
            self.cut_from(reader, file, area).await
        }
    }

    async fn cut_from<B: AsyncBackend + Send + Sync + 'static>(
        self: &Arc<Self>,
        reader: AsyncPmTilesReader<B, HashMapCache>,
        file: &Path,
        area: MapRegionRequest,
    ) -> Result<(), String> {
        let (tile_type, tile_compression, min_zoom, max_zoom) = {
            let h = reader.get_header();
            (h.tile_type, h.tile_compression, h.min_zoom, h.max_zoom)
        };
        let metadata = reader.get_metadata().await.map_err(|e| e.to_string())?;
        let coords = tiles(area, min_zoom, max_zoom);
        if let Ok(mut current) = self.current.lock()
            && let Some((_, state)) = current.as_mut()
        {
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
        if let Some(dir) = file.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let partial = file.with_extension("partial");
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
        std::fs::rename(&partial, file).map_err(|e| e.to_string())?;
        Ok(())
    }
}

// [impl->dsn~map-region~2]
/// `GET /tiles/<incident ULID>.pmtiles`: an incident's region, with range requests, for a
/// valid ticket (header `Authorization: Bearer <ticket>`). Not found until one is cut.
pub async fn serve(
    State(app): State<Arc<App>>,
    UrlPath(name): UrlPath<String>,
    request: Request,
) -> Response {
    let Some(incident) = name
        .strip_suffix(".pmtiles")
        .and_then(|id| id.parse::<Ulid>().ok())
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let allowed = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .is_some_and(|token| app.map.ticket_allows(token, incident));
    if !allowed {
        return StatusCode::FORBIDDEN.into_response();
    }
    match tower_http::services::ServeFile::new(app.map.file(incident))
        .try_call(request)
        .await
    {
        Ok(response) => response.map(axum::body::Body::new).into_response(),
        Err(error) => {
            tracing::warn!(%error, "reading a map region failed");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
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

    // [utest->dsn~map-region~2]
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

    fn incident(n: u64) -> IncidentId {
        let server: ordo_core::ServerName = "ordo.example.org".parse().unwrap();
        IncidentId::new(Ulid::from_parts(n, u128::from(n)).unwrap(), &server)
    }

    // [utest->dsn~map-region~2]
    #[test]
    fn each_incident_has_its_region_and_one_is_cut_at_a_time() {
        let dir = tempfile::tempdir().unwrap();
        let job = Arc::new(MapJob::new(dir.path().to_owned(), None));
        let (a, b) = (incident(1), incident(2));
        assert_eq!(
            job.file(a.ulid()),
            dir.path().join(format!("{}.pmtiles", a.ulid())),
            "one file per incident, named by its ULID"
        );
        std::fs::write(job.file(a.ulid()), b"PMTiles\x03").unwrap();
        let ready = job.state(&a);
        assert_eq!((ready.state, ready.bytes), (MapRegionState::Ready, Some(8)));
        assert_eq!(
            ready.incident,
            Some(a.clone()),
            "the answer names the incident"
        );
        assert_eq!(
            job.state(&b).state,
            MapRegionState::None,
            "the other has none"
        );

        // A wrong area fails for that incident only; its old region stays.
        let failed = job.start(&a, area(48.37, 10.9, 100), 0);
        assert_eq!(failed.state, MapRegionState::Failed);
        assert_eq!(failed.bytes, Some(8), "the old region stays");
        assert_eq!(job.state(&b).state, MapRegionState::None);

        // While one is cut, another incident waits.
        *job.current.lock().unwrap() = Some((
            a.ulid(),
            MapRegion {
                state: MapRegionState::Running,
                ..MapRegion::default()
            },
        ));
        let busy = job.start(&b, area(48.37, 10.9, 1_000), 0);
        assert_eq!(busy.state, MapRegionState::Failed);
        assert_eq!(busy.incident, Some(b.clone()));
        assert_eq!(job.state(&a).state, MapRegionState::Running);
    }

    // [utest->dsn~map-region~2]
    #[test]
    fn a_ticket_reads_the_region_of_its_incident_only() {
        let job = MapJob::new(std::env::temp_dir(), None);
        let (a, b) = (incident(1), incident(2));
        let ticket = job.issue_ticket(&a);
        assert_eq!(ticket.len(), 64, "32 random bytes, hex");
        assert!(job.ticket_allows(&ticket, a.ulid()));
        assert!(
            !job.ticket_allows(&ticket, b.ulid()),
            "not for another incident"
        );
        assert!(!job.ticket_allows("guessed", a.ulid()));
        assert_ne!(job.issue_ticket(&a), ticket, "a new one every time");
    }

    /// Cuts 1 km around Augsburg from the real Protomaps build (needs internet; run with
    /// `cargo test -p ordo-server -- --ignored protomaps`).
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "downloads from build.protomaps.com"]
    async fn cuts_a_small_region_from_the_protomaps_build() {
        let dir = std::env::temp_dir().join(format!("ordo-map-{}", std::process::id()));
        let job = Arc::new(MapJob::new(dir.clone(), None));
        let started = std::time::Instant::now();
        let a = incident(1);
        job.start(&a, area(48.37, 10.9, 1_000), 0);
        while job.state(&a).state == MapRegionState::Running {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        let state = job.state(&a);
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
