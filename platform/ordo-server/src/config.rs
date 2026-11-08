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
}

/// Users and devices. Prototype: read from a file; later managed in the admin UI and
/// paired by QR code (ORG-3).
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seed {
    /// Users and the compartments they may read and write.
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
    /// Compartments the user may read and write.
    pub compartments: Vec<CompartmentId>,
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
    fn invalid(name: &str, message: impl Into<String>) -> Self {
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
