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
    /// Compartments whose payloads must be sealed (`ORDO_PROTECTED`, default
    /// `med.clinical`, comma-separated).
    pub protected: Vec<CompartmentId>,
    /// Directory of the built web app, served at `/` (`ORDO_APP_DIR`, optional).
    pub app_dir: Option<PathBuf>,
    /// Directory with the organisation's own Cedar policies, `*.cedar`
    /// (`ORDO_POLICY_DIR`, optional, ADR-0035).
    pub policy_dir: Option<PathBuf>,
    /// Login with OpenID Connect (optional, ADR-0038).
    pub oidc: Option<OidcConfig>,
}

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
        let protected = get("ORDO_PROTECTED")
            .unwrap_or_else(|| "med.clinical".into())
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| {
                s.parse().map_err(|e: ordo_core::IdError| {
                    ConfigError::invalid("ORDO_PROTECTED", e.to_string())
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            server_name,
            listen,
            data_dir,
            master_key,
            seed,
            protected,
            app_dir: get("ORDO_APP_DIR").map(PathBuf::from),
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
