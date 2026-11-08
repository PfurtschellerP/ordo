//! SQLite storage (ADR-0004).
//!
//! Events are stored exactly as ordered: signed, and sealed in protected compartments.
//! Data keys and the server's signing key are stored only wrapped with the master key.

use std::path::Path;
use std::str::FromStr;

use ordo_core::codec;
use ordo_core::{CompartmentId, IncidentId, KeyId, SequencedEvent};
use ordo_crypto::seal::{self, DataKey};
use ordo_sync::{Change, StreamKey};
use sqlx::Row;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};

/// Storage failed.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Database error.
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
    /// Stored data is invalid or cannot be decrypted (wrong master key?).
    #[error("stored data is invalid: {0}")]
    Invalid(String),
}

const SCHEMA: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS events (
        incident TEXT NOT NULL,
        compartment TEXT NOT NULL,
        seq INTEGER NOT NULL,
        event BLOB NOT NULL,
        PRIMARY KEY (incident, compartment, seq)
    ) STRICT",
    "CREATE TABLE IF NOT EXISTS data_keys (
        incident TEXT NOT NULL,
        compartment TEXT NOT NULL,
        key_id TEXT NOT NULL,
        wrapped BLOB NOT NULL,
        PRIMARY KEY (incident, compartment)
    ) STRICT",
    "CREATE TABLE IF NOT EXISTS settings (
        name TEXT PRIMARY KEY,
        value BLOB NOT NULL
    ) STRICT",
];

/// The server's database.
#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
}

impl Store {
    /// Opens (and if needed creates) the database file.
    pub async fn open(path: &Path) -> Result<Self, StoreError> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await?;
        for statement in SCHEMA {
            sqlx::query(*statement).execute(&pool).await?;
        }
        Ok(Self { pool })
    }

    /// Whether the database answers (readiness).
    pub async fn ping(&self) -> bool {
        sqlx::query("SELECT 1").execute(&self.pool).await.is_ok()
    }

    // [impl->dsn~store-before-send~1]
    // [impl->dsn~wrapped-keys-at-rest~1]
    /// Stores changes of the hub in one transaction.
    pub async fn save(&self, changes: &[Change], master: &DataKey) -> Result<(), StoreError> {
        if changes.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for change in changes {
            match change {
                Change::Event { stream, event } => {
                    let bytes =
                        codec::to_cbor(event).map_err(|e| StoreError::Invalid(e.to_string()))?;
                    sqlx::query("INSERT INTO events (incident, compartment, seq, event) VALUES (?, ?, ?, ?)")
                        .bind(stream.0.as_str())
                        .bind(stream.1.as_str())
                        .bind(i64::try_from(event.seq).map_err(|e| StoreError::Invalid(e.to_string()))?)
                        .bind(bytes)
                        .execute(&mut *tx)
                        .await?;
                }
                Change::Key {
                    stream,
                    key_id,
                    key,
                } => {
                    let wrapped = seal::wrap(
                        master,
                        random_nonce(),
                        &key_label(stream, key_id),
                        key.as_bytes(),
                    );
                    sqlx::query("INSERT INTO data_keys (incident, compartment, key_id, wrapped) VALUES (?, ?, ?, ?)")
                        .bind(stream.0.as_str())
                        .bind(stream.1.as_str())
                        .bind(key_id.as_str())
                        .bind(wrapped)
                        .execute(&mut *tx)
                        .await?;
                }
            }
        }
        tx.commit().await?;
        Ok(())
    }

    // [impl->dsn~wrapped-keys-at-rest~1]
    /// All data keys, unwrapped.
    pub async fn load_keys(
        &self,
        master: &DataKey,
    ) -> Result<Vec<(StreamKey, KeyId, DataKey)>, StoreError> {
        let rows = sqlx::query("SELECT incident, compartment, key_id, wrapped FROM data_keys")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|row| {
                let stream = stream_key(row.get("incident"), row.get("compartment"))?;
                let key_id = KeyId::from_str(row.get("key_id"))
                    .map_err(|e| StoreError::Invalid(e.to_string()))?;
                let wrapped: Vec<u8> = row.get("wrapped");
                let bytes =
                    seal::unwrap(master, &key_label(&stream, &key_id), &wrapped).map_err(|_| {
                        StoreError::Invalid(
                            "a data key cannot be unwrapped — wrong master key?".into(),
                        )
                    })?;
                let bytes: [u8; 32] = bytes
                    .try_into()
                    .map_err(|_| StoreError::Invalid("data key length".into()))?;
                Ok((stream, key_id, DataKey::from_bytes(bytes)))
            })
            .collect()
    }

    /// All events, per stream in sequence order.
    pub async fn load_events(&self) -> Result<Vec<(StreamKey, SequencedEvent)>, StoreError> {
        let rows = sqlx::query(
            "SELECT incident, compartment, event FROM events ORDER BY incident, compartment, seq",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                let stream = stream_key(row.get("incident"), row.get("compartment"))?;
                let bytes: Vec<u8> = row.get("event");
                let event =
                    codec::from_cbor(&bytes).map_err(|e| StoreError::Invalid(e.to_string()))?;
                Ok((stream, event))
            })
            .collect()
    }

    /// A secret setting, unwrapped with the master key; created with `create` if absent.
    pub async fn secret(
        &self,
        name: &str,
        master: &DataKey,
        create: impl FnOnce() -> Vec<u8>,
    ) -> Result<Vec<u8>, StoreError> {
        let label = format!("setting/{name}");
        if let Some(row) = sqlx::query("SELECT value FROM settings WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?
        {
            let wrapped: Vec<u8> = row.get("value");
            return seal::unwrap(master, &label, &wrapped).map_err(|_| {
                StoreError::Invalid(format!(
                    "setting {name} cannot be unwrapped — wrong master key?"
                ))
            });
        }
        let value = create();
        let wrapped = seal::wrap(master, random_nonce(), &label, &value);
        sqlx::query("INSERT INTO settings (name, value) VALUES (?, ?)")
            .bind(name)
            .bind(wrapped)
            .execute(&self.pool)
            .await?;
        Ok(value)
    }
}

fn stream_key(incident: &str, compartment: &str) -> Result<StreamKey, StoreError> {
    let incident =
        IncidentId::from_str(incident).map_err(|e| StoreError::Invalid(e.to_string()))?;
    let compartment =
        CompartmentId::from_str(compartment).map_err(|e| StoreError::Invalid(e.to_string()))?;
    Ok((incident, compartment))
}

fn key_label(stream: &StreamKey, key_id: &KeyId) -> String {
    format!("data-key/{}/{}/{}", stream.0, stream.1, key_id)
}

/// 24 random bytes from the operating system.
pub fn random_nonce() -> [u8; 24] {
    let mut nonce = [0u8; 24];
    getrandom::fill(&mut nonce).expect("the operating system provides randomness");
    nonce
}
