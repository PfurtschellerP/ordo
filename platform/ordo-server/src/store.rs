//! SQLite storage (ADR-0004).
//!
//! Events are stored exactly as ordered: signed, and sealed in protected compartments.
//! Data keys and the server's signing key are stored only wrapped with the master key.

use std::path::Path;
use std::str::FromStr;

use ordo_core::codec;
use ordo_core::{
    CompartmentId, DeviceId, IncidentId, KeyId, SequencedEvent, ServerName, Ulid, UserId,
};
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
    // People who logged in with OpenID Connect (ADR-0038): the identity provider's
    // subject maps to a stable ORDO user ID; roles come from the provider's groups.
    "CREATE TABLE IF NOT EXISTS identities (
        issuer TEXT NOT NULL,
        subject TEXT NOT NULL,
        user_id TEXT NOT NULL UNIQUE,
        roles TEXT NOT NULL,
        PRIMARY KEY (issuer, subject)
    ) STRICT",
    // Paired devices and their public keys.
    "CREATE TABLE IF NOT EXISTS devices (
        device_id TEXT PRIMARY KEY,
        user_id TEXT NOT NULL,
        public_key BLOB NOT NULL,
        paired_ms INTEGER NOT NULL
    ) STRICT",
];

/// Columns added after the first release of a table; adding them again fails harmlessly.
const ADDED_COLUMNS: &[&str] = &[
    // End of a device's login (ms), ADR-0040.
    "ALTER TABLE devices ADD COLUMN valid_until INTEGER",
    // The identity provider's renewal token, wrapped with the master key (ADR-0040).
    "ALTER TABLE identities ADD COLUMN refresh BLOB",
    // When a device was revoked (SEC-4); revoked devices are refused for good.
    "ALTER TABLE devices ADD COLUMN revoked_ms INTEGER",
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
        for statement in ADDED_COLUMNS {
            if let Err(error) = sqlx::query(*statement).execute(&pool).await
                && !error.to_string().contains("duplicate column")
            {
                return Err(error.into());
            }
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
                // [impl->dsn~device-revocation~1]
                Change::Revoked { device } => {
                    // Devices paired by hand (seed) have no row; they stay revoked in the
                    // hub until the server restarts, then their seed entry must be removed.
                    sqlx::query("UPDATE devices SET revoked_ms = ? WHERE device_id = ?")
                        .bind(ms(crate::app::now_ms()))
                        .bind(device.ulid().to_string())
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

    /// The ORDO user of an identity-provider subject; creates the mapping on first login.
    /// `wanted` is the preferred user name; if another person has it, a number is added.
    /// The roles are replaced by the ones from this login.
    pub async fn identity(
        &self,
        issuer: &str,
        subject: &str,
        wanted: &str,
        server: &ServerName,
        roles: &[String],
    ) -> Result<UserId, StoreError> {
        let roles_text = roles.join("\n");
        if let Some(row) =
            sqlx::query("SELECT user_id FROM identities WHERE issuer = ? AND subject = ?")
                .bind(issuer)
                .bind(subject)
                .fetch_optional(&self.pool)
                .await?
        {
            let user: String = row.get("user_id");
            sqlx::query("UPDATE identities SET roles = ? WHERE issuer = ? AND subject = ?")
                .bind(&roles_text)
                .bind(issuer)
                .bind(subject)
                .execute(&self.pool)
                .await?;
            return UserId::from_str(&user).map_err(|e| StoreError::Invalid(e.to_string()));
        }
        for n in 1..1000 {
            let local = if n == 1 {
                wanted.to_owned()
            } else {
                format!("{wanted}-{n}")
            };
            let user =
                UserId::new(&local, server).map_err(|e| StoreError::Invalid(e.to_string()))?;
            let inserted = sqlx::query(
                "INSERT INTO identities (issuer, subject, user_id, roles) VALUES (?, ?, ?, ?)
                 ON CONFLICT (user_id) DO NOTHING",
            )
            .bind(issuer)
            .bind(subject)
            .bind(user.as_str())
            .bind(&roles_text)
            .execute(&self.pool)
            .await?;
            if inserted.rows_affected() == 1 {
                return Ok(user);
            }
        }
        Err(StoreError::Invalid(format!(
            "no free user name for {wanted}"
        )))
    }

    /// Roles of every user who logged in with the identity provider.
    pub async fn load_roles(&self) -> Result<Vec<(UserId, Vec<String>)>, StoreError> {
        let rows = sqlx::query("SELECT user_id, roles FROM identities")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|row| {
                let user: String = row.get("user_id");
                let roles: String = row.get("roles");
                let user =
                    UserId::from_str(&user).map_err(|e| StoreError::Invalid(e.to_string()))?;
                let roles = roles
                    .lines()
                    .filter(|r| !r.is_empty())
                    .map(str::to_owned)
                    .collect();
                Ok((user, roles))
            })
            .collect()
    }

    /// Stores a paired device whose login lasts until `valid_until` (ms). Pairing it again
    /// replaces its key, but only for the same user: a device ID of another person cannot
    /// be taken over (returns `false`).
    pub async fn save_device(
        &self,
        device: &DeviceId,
        user: &UserId,
        key: &[u8; 32],
        now_ms: u64,
        valid_until: u64,
    ) -> Result<bool, StoreError> {
        let saved = sqlx::query(
            "INSERT INTO devices (device_id, user_id, public_key, paired_ms, valid_until)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT (device_id) DO UPDATE SET public_key = excluded.public_key,
                 paired_ms = excluded.paired_ms, valid_until = excluded.valid_until,
                 revoked_ms = NULL
             WHERE devices.user_id = excluded.user_id",
        )
        .bind(device.ulid().to_string())
        .bind(user.as_str())
        .bind(key.as_slice())
        .bind(ms(now_ms))
        .bind(ms(valid_until))
        .execute(&self.pool)
        .await?;
        Ok(saved.rows_affected() == 1)
    }

    /// Revoked devices.
    pub async fn load_revoked(&self) -> Result<Vec<DeviceId>, StoreError> {
        let rows = sqlx::query("SELECT device_id FROM devices WHERE revoked_ms IS NOT NULL")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|row| {
                let device: String = row.get("device_id");
                device
                    .parse::<Ulid>()
                    .map(DeviceId)
                    .map_err(|e| StoreError::Invalid(e.to_string()))
            })
            .collect()
    }

    /// When each device was paired (ms).
    pub async fn load_paired(&self) -> Result<Vec<(DeviceId, u64)>, StoreError> {
        let rows = sqlx::query("SELECT device_id, paired_ms FROM devices WHERE revoked_ms IS NULL")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|row| {
                let device: String = row.get("device_id");
                let paired: i64 = row.get("paired_ms");
                Ok((
                    DeviceId(
                        device
                            .parse::<Ulid>()
                            .map_err(|e| StoreError::Invalid(e.to_string()))?,
                    ),
                    u64::try_from(paired).unwrap_or_default(),
                ))
            })
            .collect()
    }

    /// Sets the end of the login of all devices of `user` (re-verified or ended).
    pub async fn set_valid_until(&self, user: &UserId, valid_until: u64) -> Result<(), StoreError> {
        sqlx::query("UPDATE devices SET valid_until = ? WHERE user_id = ?")
            .bind(ms(valid_until))
            .bind(user.as_str())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Stores (or with `None` removes) the identity provider's renewal token of a person,
    /// wrapped with the master key.
    pub async fn set_refresh(
        &self,
        issuer: &str,
        subject: &str,
        token: Option<&str>,
        master: &DataKey,
    ) -> Result<(), StoreError> {
        let wrapped = token.map(|t| {
            seal::wrap(
                master,
                random_nonce(),
                &refresh_label(issuer, subject),
                t.as_bytes(),
            )
        });
        sqlx::query("UPDATE identities SET refresh = ? WHERE issuer = ? AND subject = ?")
            .bind(wrapped)
            .bind(issuer)
            .bind(subject)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Everyone with a renewal token: (issuer, subject, user, token).
    pub async fn load_refresh(
        &self,
        master: &DataKey,
    ) -> Result<Vec<(String, String, UserId, String)>, StoreError> {
        let rows = sqlx::query(
            "SELECT issuer, subject, user_id, refresh FROM identities WHERE refresh IS NOT NULL",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                let issuer: String = row.get("issuer");
                let subject: String = row.get("subject");
                let user: String = row.get("user_id");
                let wrapped: Vec<u8> = row.get("refresh");
                let token = seal::unwrap(master, &refresh_label(&issuer, &subject), &wrapped)
                    .map_err(|_| StoreError::Invalid("renewal token cannot be unwrapped".into()))?;
                Ok((
                    issuer,
                    subject,
                    UserId::from_str(&user).map_err(|e| StoreError::Invalid(e.to_string()))?,
                    String::from_utf8(token).map_err(|e| StoreError::Invalid(e.to_string()))?,
                ))
            })
            .collect()
    }

    /// All paired devices: (device, user, public key, end of login).
    pub async fn load_devices(
        &self,
    ) -> Result<Vec<(DeviceId, UserId, [u8; 32], Option<u64>)>, StoreError> {
        let rows = sqlx::query(
            "SELECT device_id, user_id, public_key, valid_until FROM devices
             WHERE revoked_ms IS NULL",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                let invalid = |e: String| StoreError::Invalid(e);
                let device: String = row.get("device_id");
                let user: String = row.get("user_id");
                let key: Vec<u8> = row.get("public_key");
                Ok((
                    DeviceId(device.parse::<Ulid>().map_err(|e| invalid(e.to_string()))?),
                    UserId::from_str(&user).map_err(|e| invalid(e.to_string()))?,
                    key.try_into()
                        .map_err(|_| invalid("device key length".into()))?,
                    row.get::<Option<i64>, _>("valid_until")
                        .map(|v| u64::try_from(v).unwrap_or_default()),
                ))
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

fn ms(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn refresh_label(issuer: &str, subject: &str) -> String {
    format!("refresh/{issuer}/{subject}")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn user(name: &str) -> UserId {
        format!("{name}@ordo.example.org").parse().unwrap()
    }

    #[tokio::test]
    async fn a_paired_device_cannot_be_taken_over_by_another_person() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("ordo.db")).await.unwrap();
        let device = DeviceId(Ulid::from_parts(1, 1).unwrap());
        assert!(
            store
                .save_device(&device, &user("anna"), &[1; 32], 1, 100)
                .await
                .unwrap()
        );
        assert!(
            store
                .save_device(&device, &user("anna"), &[2; 32], 2, 200)
                .await
                .unwrap(),
            "the same person may pair the device again with a new key"
        );
        assert!(
            !store
                .save_device(&device, &user("mallory"), &[3; 32], 3, 300)
                .await
                .unwrap(),
            "another person may not take the device ID over"
        );
        let devices = store.load_devices().await.unwrap();
        assert_eq!(devices, vec![(device, user("anna"), [2; 32], Some(200))]);
        store.set_valid_until(&user("anna"), 5).await.unwrap();
        assert_eq!(store.load_devices().await.unwrap()[0].3, Some(5));
    }

    // [utest->dsn~device-revocation~1]
    #[tokio::test]
    async fn a_revoked_device_stays_revoked_after_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ordo.db");
        let device = DeviceId(Ulid::from_parts(1, 1).unwrap());
        {
            let store = Store::open(&path).await.unwrap();
            store
                .save_device(&device, &user("anna"), &[1; 32], 1, 100)
                .await
                .unwrap();
            let master = DataKey::from_bytes([7; 32]);
            store
                .save(&[Change::Revoked { device }], &master)
                .await
                .unwrap();
        }
        let store = Store::open(&path).await.unwrap();
        assert_eq!(store.load_revoked().await.unwrap(), vec![device]);
        assert!(store.load_devices().await.unwrap().is_empty());
        assert!(store.load_paired().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn identities_get_stable_user_names() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("ordo.db")).await.unwrap();
        let server: ServerName = "ordo.example.org".parse().unwrap();
        let medic = vec!["medic".to_owned()];
        let a = store
            .identity("https://id", "sub-1", "anna", &server, &medic)
            .await
            .unwrap();
        let b = store
            .identity("https://id", "sub-2", "anna", &server, &[])
            .await
            .unwrap();
        let again = store
            .identity("https://id", "sub-1", "renamed", &server, &[])
            .await
            .unwrap();
        assert_eq!(a, user("anna"));
        assert_eq!(
            b,
            user("anna-2"),
            "another person with the same name gets a number"
        );
        assert_eq!(
            again, a,
            "the user ID stays when the name at the provider changes"
        );
        let roles = store.load_roles().await.unwrap();
        assert!(
            roles.contains(&(user("anna"), Vec::new())),
            "roles follow the latest login"
        );
    }
}
