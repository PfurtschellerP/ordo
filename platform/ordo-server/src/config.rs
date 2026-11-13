//! Configuration from environment variables (twelve-factor, OPS-7).

use std::net::SocketAddr;
use std::path::PathBuf;

use ordo_core::{CompartmentId, DeviceId, ServerName, Ulid, UserId};
use ordo_crypto::seal::DataKey;
use serde::Deserialize;

/// Server configuration.
pub struct Config {
    /// Name of this server (`ORDO_SERVER_NAME`, required).
    pub server_name: ServerName,
    /// Listen address (`ORDO_LISTEN`, default `0.0.0.0:8080`).
    pub listen: SocketAddr,
    /// Directory for the database (`ORDO_DATA_DIR`, default `/data`).
    pub data_dir: PathBuf,
    /// Key that encrypts all stored keys (`ORDO_MASTER_KEY_FILE`: 64 hex characters).
    pub master_key: DataKey,
    /// Users and devices (`ORDO_SEED_FILE`, optional).
    pub seed: Seed,
    /// The installation's vehicles (`ORDO_VEHICLES_FILE`, optional, TOML with
    /// `[[vehicle]]` entries), offered when adding units (UNIT-1).
    pub vehicles: Vec<ordo_sync::message::Vehicle>,
    /// Compartments whose payloads must be sealed (`ORDO_PROTECTED`, default
    /// `med.clinical,people`, comma-separated).
    pub protected: Vec<CompartmentId>,
    /// Directory of the built web app, served at `/` (`ORDO_APP_DIR`, optional).
    pub app_dir: Option<PathBuf>,
    /// Directory of the incidents' map regions, one PMTiles file per incident served at
    /// `/tiles/<incident ULID>.pmtiles` (`ORDO_MAP_DIR`, default `maps` in the data
    /// directory, ADR-0062); the incident command cuts them from `map_source` (ADR-0057).
    pub map_dir: PathBuf,
    /// The world map regions are cut from: a PMTiles URL or file (`ORDO_MAP_SOURCE`,
    /// default the Protomaps build of the day before).
    pub map_source: Option<String>,
    /// Address search for the incident's map (ADR-0066): base URL of a Nominatim-compatible
    /// API (`ORDO_GEOCODER`, default the OpenStreetMap Foundation's public Nominatim; `off`
    /// switches the search off).
    pub geocoder: Option<String>,
    /// Countries the address search is limited to, ISO 3166-1 alpha-2 codes,
    /// comma-separated, e.g. `de` (`ORDO_GEOCODER_COUNTRIES`, optional).
    pub geocoder_countries: Option<String>,
    /// Directory with the organisation's own Cedar policies, `*.cedar`
    /// (`ORDO_POLICY_DIR`, optional, ADR-0035).
    pub policy_dir: Option<PathBuf>,
    /// Login with OpenID Connect (optional, ADR-0038).
    pub oidc: Option<OidcConfig>,
    /// Days after an incident is closed until its helpers' data (compartment `people`) is
    /// deleted (`ORDO_PEOPLE_RETENTION_DAYS`, default 90, from 1 to 3650; HELP-4, ADR-0071).
    pub people_retention_days: u64,
    /// How long a device's login lasts without re-verification (`ORDO_ACCESS_TIMEOUT_HOURS`,
    /// default 48, from 24 to 720 hours, i.e. 1 to 30 days; ADR-0040).
    pub access_timeout: std::time::Duration,
    /// Administrators by user name, e.g. `dora` for `dora@<server>` (`ORDO_ADMINS`,
    /// comma-separated). They plan incidents and grant roles (ADR-0046).
    pub admins: Vec<UserId>,
    /// Whether the server looks up profile pictures at Gravatar by the hash of the
    /// e-mail address, when neither ORDO nor the identity provider has one
    /// (`ORDO_GRAVATAR`, `on` or `off`, default `on`; ADR-0047). Devices never contact
    /// Gravatar themselves.
    pub gravatar: bool,
    /// Largest attachment in bytes (`ORDO_MAX_BLOB_MB`, default 20, 1 to 200; ADR-0033).
    /// Attachments are stored in `<ORDO_DATA_DIR>/blobs`, or in an S3-compatible store.
    pub max_blob_bytes: u64,
    /// S3-compatible store for attachments (`ORDO_S3_BUCKET` and the other `ORDO_S3_*`
    /// variables, optional; ADR-0033). Without it, a local directory is used.
    pub s3: Option<S3Config>,
    /// Dex's gRPC API for local accounts (`ORDO_DEX_API` and its certificate files,
    /// optional; ADR-0060). Without it, accounts are not managed in ORDO.
    pub dex_api: Option<crate::dex_api::DexApiConfig>,
}

/// An S3-compatible object store, e.g. Garage (ADR-0033).
#[derive(Clone)]
pub struct S3Config {
    /// Address of the S3 API, e.g. `http://garage:3900` (`ORDO_S3_ENDPOINT`).
    pub endpoint: String,
    /// Bucket (`ORDO_S3_BUCKET`).
    pub bucket: String,
    /// Region name the store expects (`ORDO_S3_REGION`, default `garage`).
    pub region: String,
    /// Access key ID (`ORDO_S3_ACCESS_KEY_ID`).
    pub access_key_id: String,
    /// Secret access key, read from a file (`ORDO_S3_SECRET_ACCESS_KEY_FILE`).
    pub secret_access_key: String,
}

impl std::fmt::Debug for S3Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("S3Config")
            .field("endpoint", &self.endpoint)
            .field("bucket", &self.bucket)
            .field("region", &self.region)
            .field("access_key_id", &self.access_key_id)
            .finish_non_exhaustive()
    }
}

/// Default access timeout in hours (ADR-0040).
pub const ACCESS_TIMEOUT_HOURS: u64 = 48;
/// Default days until the helpers' data of a closed incident is deleted (HELP-4).
pub const PEOPLE_RETENTION_DAYS: u64 = 90;
/// Allowed access timeouts in hours: 1 to 30 days.
pub const ACCESS_TIMEOUT_RANGE: std::ops::RangeInclusive<u64> = 24..=720;

/// The organisation's identity provider (all of `ORDO_OIDC_ISSUER`,
/// `ORDO_OIDC_CLIENT_ID`, `ORDO_OIDC_CLIENT_SECRET_FILE` and `ORDO_PUBLIC_URL`).
#[derive(Debug, Clone)]
pub struct OidcConfig {
    /// Issuer URL of the identity provider, e.g. `https://id.example.org/dex`.
    pub issuer: String,
    /// Client ID of ORDO at the identity provider.
    pub client_id: String,
    /// Client secret, read from a file.
    pub client_secret: String,
    /// Address under which people reach ORDO, e.g. `https://ordo.example.org`; the login
    /// returns to `<public URL>/auth/callback`.
    pub public_url: String,
}

/// Users and devices. Prototype: read from a file; later managed in the admin UI and
/// paired by QR code (ORG-3).
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seed {
    /// Users and their roles.
    #[serde(default, rename = "user")]
    pub users: Vec<SeedUser>,
    /// Paired devices.
    #[serde(default, rename = "device")]
    pub devices: Vec<SeedDevice>,
}

/// A user in the seed file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedUser {
    /// User ID, e.g. `anna@ordo.example.org`.
    pub id: UserId,
    /// Roles, e.g. `medic`, `commander`. What a role may do is decided by the Cedar
    /// policies (ADR-0035), never per person.
    pub roles: Vec<String>,
}

/// A device in the seed file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedDevice {
    /// Device ID (ULID).
    pub id: String,
    /// The user the device is paired with.
    pub user: UserId,
    /// Ed25519 public key, 64 hex characters.
    pub public_key: String,
}

impl SeedDevice {
    /// The device ID.
    pub fn device_id(&self) -> Result<DeviceId, ConfigError> {
        self.id
            .parse::<Ulid>()
            .map(DeviceId)
            .map_err(|e| ConfigError::invalid("device id", e.to_string()))
    }

    /// The public key bytes.
    pub fn key_bytes(&self) -> Result<[u8; 32], ConfigError> {
        parse_hex(&self.public_key)
            .ok_or_else(|| ConfigError::invalid("public_key", "expected 64 hex characters"))
    }
}

/// The configuration is incomplete or invalid.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A required variable is not set.
    #[error("{0} is not set")]
    Missing(&'static str),
    /// A value is invalid.
    #[error("{name}: {message}")]
    Invalid {
        /// Variable or field.
        name: String,
        /// What is wrong.
        message: String,
    },
    /// A file could not be read.
    #[error("could not read {file}: {source}")]
    Io {
        /// The file.
        file: String,
        /// The error.
        #[source]
        source: std::io::Error,
    },
}

impl ConfigError {
    pub(crate) fn invalid(name: &str, message: impl Into<String>) -> Self {
        Self::Invalid {
            name: name.to_owned(),
            message: message.into(),
        }
    }
}

impl Config {
    /// Reads the configuration from environment variables.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    // [impl->dsn~environment-configuration~1]
    /// Reads the configuration through `get` (for tests).
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let server_name = get("ORDO_SERVER_NAME")
            .ok_or(ConfigError::Missing("ORDO_SERVER_NAME"))?
            .parse()
            .map_err(|e: ordo_core::IdError| {
                ConfigError::invalid("ORDO_SERVER_NAME", e.to_string())
            })?;
        let listen = get("ORDO_LISTEN")
            .unwrap_or_else(|| "0.0.0.0:8080".into())
            .parse()
            .map_err(|e: std::net::AddrParseError| {
                ConfigError::invalid("ORDO_LISTEN", e.to_string())
            })?;
        let data_dir = PathBuf::from(get("ORDO_DATA_DIR").unwrap_or_else(|| "/data".into()));
        let key_file =
            get("ORDO_MASTER_KEY_FILE").ok_or(ConfigError::Missing("ORDO_MASTER_KEY_FILE"))?;
        let master_key = parse_hex(read(&key_file)?.trim())
            .map(DataKey::from_bytes)
            .ok_or_else(|| {
                ConfigError::invalid(
                    "ORDO_MASTER_KEY_FILE",
                    "expected 64 hex characters (see `ordo generate-key`)",
                )
            })?;
        let seed = match get("ORDO_SEED_FILE") {
            Some(file) => toml::from_str(&read(&file)?)
                .map_err(|e| ConfigError::invalid("ORDO_SEED_FILE", e.message().to_owned()))?,
            None => Seed::default(),
        };
        // [impl->dsn~unit-register~2]
        let vehicles = match get("ORDO_VEHICLES_FILE") {
            Some(file) => {
                #[derive(serde::Deserialize)]
                struct Vehicles {
                    #[serde(default)]
                    vehicle: Vec<ordo_sync::message::Vehicle>,
                }
                toml::from_str::<Vehicles>(&read(&file)?)
                    .map_err(|e| {
                        ConfigError::invalid("ORDO_VEHICLES_FILE", e.message().to_owned())
                    })?
                    .vehicle
            }
            None => Vec::new(),
        };
        let protected = get("ORDO_PROTECTED")
            .unwrap_or_else(|| "med.clinical,people".into())
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse().map_err(|e: ordo_core::IdError| {
                    ConfigError::invalid("ORDO_PROTECTED", e.to_string())
                })
            })
            .collect::<Result<_, _>>()?;
        let timeout_hours = match get("ORDO_ACCESS_TIMEOUT_HOURS") {
            None => ACCESS_TIMEOUT_HOURS,
            Some(text) => text
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|h| ACCESS_TIMEOUT_RANGE.contains(h))
                .ok_or_else(|| {
                    ConfigError::invalid(
                        "ORDO_ACCESS_TIMEOUT_HOURS",
                        "expected hours from 24 to 720 (1 to 30 days)",
                    )
                })?,
        };
        let people_retention_days = match get("ORDO_PEOPLE_RETENTION_DAYS") {
            None => PEOPLE_RETENTION_DAYS,
            Some(text) => text
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|d| (1..=3650).contains(d))
                .ok_or_else(|| {
                    ConfigError::invalid(
                        "ORDO_PEOPLE_RETENTION_DAYS",
                        "expected days from 1 to 3650",
                    )
                })?,
        };
        let admins = get("ORDO_ADMINS")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|name| {
                UserId::new(name, &server_name)
                    .map_err(|e| ConfigError::invalid("ORDO_ADMINS", e.to_string()))
            })
            .collect::<Result<_, _>>()?;
        let gravatar = match get("ORDO_GRAVATAR").as_deref().map(str::trim) {
            None | Some("on" | "true" | "1") => true,
            Some("off" | "false" | "0") => false,
            Some(_) => return Err(ConfigError::invalid("ORDO_GRAVATAR", "expected on or off")),
        };
        let max_blob_mb = match get("ORDO_MAX_BLOB_MB") {
            None => 20,
            Some(text) => text
                .trim()
                .parse::<u64>()
                .ok()
                .filter(|m| (1..=200).contains(m))
                .ok_or_else(|| ConfigError::invalid("ORDO_MAX_BLOB_MB", "expected 1 to 200"))?,
        };
        let s3 = match get("ORDO_S3_BUCKET").filter(|b| !b.trim().is_empty()) {
            None => None,
            Some(bucket) => Some(S3Config {
                endpoint: get("ORDO_S3_ENDPOINT")
                    .ok_or(ConfigError::Missing("ORDO_S3_ENDPOINT"))?
                    .trim_end_matches('/')
                    .to_owned(),
                bucket: bucket.trim().to_owned(),
                region: get("ORDO_S3_REGION").unwrap_or_else(|| "garage".into()),
                access_key_id: get("ORDO_S3_ACCESS_KEY_ID")
                    .ok_or(ConfigError::Missing("ORDO_S3_ACCESS_KEY_ID"))?,
                secret_access_key: read(
                    &get("ORDO_S3_SECRET_ACCESS_KEY_FILE")
                        .ok_or(ConfigError::Missing("ORDO_S3_SECRET_ACCESS_KEY_FILE"))?,
                )?
                .trim()
                .to_owned(),
            }),
        };
        let map_dir = get("ORDO_MAP_DIR").map_or_else(|| data_dir.join("maps"), PathBuf::from);
        let dex_api = match get("ORDO_DEX_API").filter(|u| !u.trim().is_empty()) {
            None => None,
            Some(url) => {
                let file = |name: &'static str| -> Result<String, ConfigError> {
                    read(&get(name).ok_or(ConfigError::Missing(name))?)
                };
                Some(crate::dex_api::DexApiConfig {
                    url: url.trim().to_owned(),
                    ca: file("ORDO_DEX_API_CA_FILE")?,
                    cert: file("ORDO_DEX_API_CERT_FILE")?,
                    key: file("ORDO_DEX_API_KEY_FILE")?,
                })
            }
        };
        Ok(Self {
            dex_api,
            s3,
            max_blob_bytes: max_blob_mb * 1024 * 1024,
            gravatar,
            admins,
            access_timeout: std::time::Duration::from_secs(timeout_hours * 3600),
            people_retention_days,
            server_name,
            listen,
            data_dir,
            master_key,
            seed,
            vehicles,
            protected,
            app_dir: get("ORDO_APP_DIR").map(PathBuf::from),
            map_dir,
            map_source: get("ORDO_MAP_SOURCE").filter(|s| !s.trim().is_empty()),
            geocoder: match get("ORDO_GEOCODER").as_deref().map(str::trim) {
                None | Some("") => Some(crate::geocode::DEFAULT_URL.to_owned()),
                Some("off") => None,
                Some(url) if url.starts_with("https://") || url.starts_with("http://") => {
                    Some(url.to_owned())
                }
                Some(_) => {
                    return Err(ConfigError::invalid(
                        "ORDO_GEOCODER",
                        "an http(s) URL or off".to_owned(),
                    ));
                }
            },
            geocoder_countries: match get("ORDO_GEOCODER_COUNTRIES")
                .map(|c| c.trim().to_ascii_lowercase())
                .filter(|c| !c.is_empty())
            {
                None => None,
                Some(codes)
                    if codes.split(',').all(|c| {
                        c.trim().len() == 2 && c.trim().bytes().all(|b| b.is_ascii_lowercase())
                    }) =>
                {
                    Some(
                        codes
                            .split(',')
                            .map(str::trim)
                            .collect::<Vec<_>>()
                            .join(","),
                    )
                }
                Some(_) => {
                    return Err(ConfigError::invalid(
                        "ORDO_GEOCODER_COUNTRIES",
                        "two-letter country codes, comma-separated".to_owned(),
                    ));
                }
            },
            policy_dir: get("ORDO_POLICY_DIR").map(PathBuf::from),
            oidc: match get("ORDO_OIDC_ISSUER").filter(|issuer| !issuer.trim().is_empty()) {
                None => None,
                Some(issuer) => Some(OidcConfig {
                    issuer,
                    client_id: get("ORDO_OIDC_CLIENT_ID")
                        .ok_or(ConfigError::Missing("ORDO_OIDC_CLIENT_ID"))?,
                    client_secret: read(
                        &get("ORDO_OIDC_CLIENT_SECRET_FILE")
                            .ok_or(ConfigError::Missing("ORDO_OIDC_CLIENT_SECRET_FILE"))?,
                    )?
                    .trim()
                    .to_owned(),
                    public_url: get("ORDO_PUBLIC_URL")
                        .ok_or(ConfigError::Missing("ORDO_PUBLIC_URL"))?
                        .trim_end_matches('/')
                        .to_owned(),
                }),
            },
        })
    }
}

fn read(file: &str) -> Result<String, ConfigError> {
    std::fs::read_to_string(file).map_err(|source| ConfigError::Io {
        file: file.to_owned(),
        source,
    })
}

/// Parses `2·N` hex characters into `N` bytes.
pub fn parse_hex<const N: usize>(text: &str) -> Option<[u8; N]> {
    if text.len() != 2 * N || !text.is_ascii() {
        return None;
    }
    let mut out = [0u8; N];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(out)
}

/// Lower-case hex.
pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(timeout: Option<&str>) -> Result<Config, ConfigError> {
        let dir = std::env::temp_dir().join(format!("ordo-config-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let key = dir.join("master.key");
        std::fs::write(&key, "00".repeat(32)).unwrap();
        let key = key.to_string_lossy().into_owned();
        Config::from_lookup(|name| match name {
            "ORDO_SERVER_NAME" => Some("ordo.example.org".into()),
            "ORDO_MASTER_KEY_FILE" => Some(key.clone()),
            "ORDO_ACCESS_TIMEOUT_HOURS" => timeout.map(str::to_owned),
            _ => None,
        })
    }

    #[test]
    fn an_s3_store_needs_endpoint_key_and_secret() {
        let dir = std::env::temp_dir().join(format!("ordo-config-s3-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let key = dir.join("master.key");
        std::fs::write(&key, "00".repeat(32)).unwrap();
        let secret = dir.join("s3.secret");
        std::fs::write(&secret, "geheim\n").unwrap();
        let (key, secret) = (
            key.to_string_lossy().into_owned(),
            secret.to_string_lossy().into_owned(),
        );
        let with = |missing: &str| {
            Config::from_lookup(|name| {
                if name == missing {
                    return None;
                }
                match name {
                    "ORDO_SERVER_NAME" => Some("ordo.example.org".into()),
                    "ORDO_MASTER_KEY_FILE" => Some(key.clone()),
                    "ORDO_S3_BUCKET" => Some("ordo".into()),
                    "ORDO_S3_ENDPOINT" => Some("http://garage:3900/".into()),
                    "ORDO_S3_ACCESS_KEY_ID" => Some("GK0123".into()),
                    "ORDO_S3_SECRET_ACCESS_KEY_FILE" => Some(secret.clone()),
                    _ => None,
                }
            })
        };
        let s3 = with("").unwrap().s3.expect("S3 store configured");
        assert_eq!(s3.endpoint, "http://garage:3900");
        assert_eq!(s3.region, "garage");
        assert_eq!(s3.secret_access_key, "geheim");
        assert!(
            !format!("{s3:?}").contains("geheim"),
            "the secret is not logged"
        );
        assert!(
            with("ORDO_S3_BUCKET").unwrap().s3.is_none(),
            "local directory"
        );
        for missing in [
            "ORDO_S3_ENDPOINT",
            "ORDO_S3_ACCESS_KEY_ID",
            "ORDO_S3_SECRET_ACCESS_KEY_FILE",
        ] {
            assert!(with(missing).is_err(), "{missing}");
        }
    }

    // [utest->dsn~access-timeout~1]
    #[test]
    fn the_access_timeout_is_48_hours_and_can_be_1_to_30_days() {
        let hours = |c: Config| c.access_timeout.as_secs() / 3600;
        assert_eq!(hours(config(None).unwrap()), 48);
        assert_eq!(hours(config(Some("24")).unwrap()), 24);
        assert_eq!(hours(config(Some("720")).unwrap()), 720);
        for wrong in ["23", "721", "0", "two days"] {
            assert!(config(Some(wrong)).is_err(), "{wrong}");
        }
    }
}
