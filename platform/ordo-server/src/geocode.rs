//! Address search for the incident's map (ADR-0066): the server asks a Nominatim-compatible
//! geocoder for the devices, so devices never contact a third party themselves and the
//! geocoder's usage policy is kept in one place. The OpenStreetMap Foundation's policy
//! (<https://operations.osmfoundation.org/policies/nominatim/>): at most one request per
//! second, an identifying User-Agent, results cached, no search as you type (the app
//! searches when the person submits).

use std::collections::HashMap;
use std::time::{Duration, Instant};

use ordo_sync::message::{GEOCODE_QUERY_MAX, GeocodeStatus, GeocodedPlace};

/// The public Nominatim of the OpenStreetMap Foundation, the default geocoder.
pub const DEFAULT_URL: &str = "https://nominatim.openstreetmap.org";
/// Places per answer.
const LIMIT: usize = 5;
/// The geocoder's pace: one request per second at most.
const PACE: Duration = Duration::from_secs(1);
/// How long a request may take.
const TIMEOUT: Duration = Duration::from_secs(10);
/// How long an answer is kept, and how many.
const CACHE_FOR: Duration = Duration::from_secs(24 * 3600);
const CACHE_SIZE: usize = 1000;

/// The geocoder, its pace and its answers.
pub struct Geocoder {
    /// Base URL of a Nominatim-compatible API; `None`: switched off.
    url: Option<String>,
    /// ISO 3166-1 codes the search is limited to, e.g. `de` (optional).
    countries: Option<String>,
    http: reqwest::Client,
    /// When the last request was sent; held while a request runs, so requests queue.
    last: tokio::sync::Mutex<Option<Instant>>,
    cache: std::sync::Mutex<HashMap<String, (Instant, Vec<GeocodedPlace>)>>,
}

impl Geocoder {
    /// A geocoder at `url` (`None` switches the search off), identifying ORDO with
    /// `contact` (the installation's public address or the project's).
    pub fn new(url: Option<String>, countries: Option<String>, contact: &str) -> Self {
        let agent = format!("ORDO/{} (+{contact})", env!("CARGO_PKG_VERSION"));
        let http = reqwest::Client::builder()
            .user_agent(agent)
            .timeout(TIMEOUT)
            .build()
            .expect("a client without special settings builds");
        Self {
            url: url.map(|u| u.trim_end_matches('/').to_owned()),
            countries,
            http,
            last: tokio::sync::Mutex::new(None),
            cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    // [impl->dsn~address-search~1]
    /// The places for `query`, best first: from the cache, else from the geocoder at its
    /// pace.
    pub async fn search(&self, query: &str) -> (GeocodeStatus, Vec<GeocodedPlace>) {
        let Some(url) = &self.url else {
            return (GeocodeStatus::Off, Vec::new());
        };
        let Some(key) = normalised(query) else {
            return (GeocodeStatus::Refused, Vec::new());
        };
        if let Some(places) = self.cached(&key) {
            return (GeocodeStatus::Found, places);
        }
        let mut last = self.last.lock().await;
        // Someone may have asked the same while this request waited.
        if let Some(places) = self.cached(&key) {
            return (GeocodeStatus::Found, places);
        }
        if let Some(at) = *last {
            tokio::time::sleep_until((at + PACE).into()).await;
        }
        *last = Some(Instant::now());
        let mut request = self.http.get(format!("{url}/search")).query(&[
            ("q", key.as_str()),
            ("format", "jsonv2"),
            ("limit", &LIMIT.to_string()),
        ]);
        if let Some(countries) = &self.countries {
            request = request.query(&[("countrycodes", countries.as_str())]);
        }
        let answer = match request.send().await {
            Ok(response) if response.status().is_success() => response.text().await.ok(),
            Ok(response) => {
                tracing::warn!(status = %response.status(), "the geocoder refused a search");
                None
            }
            Err(error) => {
                tracing::warn!(%error, "the geocoder did not answer");
                None
            }
        };
        drop(last);
        match answer.and_then(|text| parse(&text)) {
            Some(places) => {
                self.remember(key, places.clone());
                (GeocodeStatus::Found, places)
            }
            None => (GeocodeStatus::Failed, Vec::new()),
        }
    }

    fn cached(&self, key: &str) -> Option<Vec<GeocodedPlace>> {
        let cache = self.cache.lock().expect("geocoder cache lock");
        cache
            .get(key)
            .filter(|(at, _)| at.elapsed() < CACHE_FOR)
            .map(|(_, places)| places.clone())
    }

    fn remember(&self, key: String, places: Vec<GeocodedPlace>) {
        let mut cache = self.cache.lock().expect("geocoder cache lock");
        if cache.len() >= CACHE_SIZE {
            cache.retain(|_, (at, _)| at.elapsed() < CACHE_FOR);
        }
        if cache.len() >= CACHE_SIZE
            && let Some(oldest) = cache
                .iter()
                .min_by_key(|(_, (at, _))| *at)
                .map(|(k, _)| k.clone())
        {
            cache.remove(&oldest);
        }
        cache.insert(key, (Instant::now(), places));
    }
}

/// The query as it is sent and cached: trimmed, inner spaces folded, lower case; `None`
/// if empty or longer than the protocol allows.
fn normalised(query: &str) -> Option<String> {
    let folded = query.split_whitespace().collect::<Vec<_>>().join(" ");
    let count = folded.chars().count();
    (count > 0 && count <= GEOCODE_QUERY_MAX).then(|| folded.to_lowercase())
}

/// Nominatim's `jsonv2` answer: `[{"display_name": …, "lat": "48.1", "lon": "11.5"}, …]`.
fn parse(text: &str) -> Option<Vec<GeocodedPlace>> {
    let list: Vec<serde_json::Value> = serde_json::from_str(text).ok()?;
    Some(
        list.iter()
            .filter_map(|p| {
                let degrees = |field: &str| -> Option<i32> {
                    let v: f64 = p[field].as_str()?.parse().ok()?;
                    let e7 = (v * 1e7).round();
                    (e7.abs() <= 1_800_000_000.0).then_some(e7 as i32)
                };
                Some(GeocodedPlace {
                    label: p["display_name"].as_str()?.chars().take(300).collect(),
                    lat_e7: degrees("lat").filter(|l| l.abs() <= 900_000_000)?,
                    lon_e7: degrees("lon")?,
                })
            })
            .take(LIMIT)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~address-search~1]
    #[test]
    fn queries_are_folded_and_bounded() {
        assert_eq!(
            normalised("  Marktplatz 1,\n Augsburg ").as_deref(),
            Some("marktplatz 1, augsburg")
        );
        assert_eq!(normalised("   "), None);
        assert_eq!(normalised(&"x".repeat(GEOCODE_QUERY_MAX + 1)), None);
    }

    // [utest->dsn~address-search~1]
    #[test]
    fn nominatim_answers_are_read() {
        let text = r#"[
            {"display_name": "Rathausplatz, Augsburg", "lat": "48.3689", "lon": "10.8978"},
            {"display_name": "broken", "lat": "north", "lon": "10"},
            {"display_name": "out of range", "lat": "91", "lon": "10"}
        ]"#;
        assert_eq!(
            parse(text).unwrap(),
            vec![GeocodedPlace {
                label: "Rathausplatz, Augsburg".into(),
                lat_e7: 483_689_000,
                lon_e7: 108_978_000,
            }]
        );
        assert_eq!(parse("not json"), None);
    }

    // [utest->dsn~address-search~1]
    #[tokio::test]
    async fn without_a_geocoder_the_search_is_off() {
        let geocoder = Geocoder::new(None, None, "https://example.org");
        assert_eq!(geocoder.search("Augsburg").await.0, GeocodeStatus::Off);
    }
}
