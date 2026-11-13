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
    /// The schema could not be brought up to date (e.g. a released migration was changed).
    #[error("database schema: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    /// Stored data is invalid or cannot be decrypted (wrong master key?).
    #[error("stored data is invalid: {0}")]
    Invalid(String),
}

/// The schema migrations (`migrations/`), embedded at build time and applied in order.
static MIGRATIONS: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

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
        MIGRATIONS.run(&pool).await?;
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

    /// Records that the attachment `hash` belongs to a stream (after its upload).
    pub async fn add_blob(
        &self,
        hash: &str,
        stream: &StreamKey,
        size: u64,
        now_ms: u64,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO blobs (hash, incident, compartment, size, stored_ms) VALUES (?, ?, ?, ?, ?)
             ON CONFLICT DO NOTHING",
        )
        .bind(hash)
        .bind(stream.0.to_string())
        .bind(stream.1.as_str())
        .bind(i64::try_from(size).unwrap_or(i64::MAX))
        .bind(ms(now_ms))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Whether the attachment `hash` belongs to the stream.
    pub async fn blob_in(&self, hash: &str, stream: &StreamKey) -> Result<bool, StoreError> {
        let row =
            sqlx::query("SELECT 1 FROM blobs WHERE hash = ? AND incident = ? AND compartment = ?")
                .bind(hash)
                .bind(stream.0.to_string())
                .bind(stream.1.as_str())
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.is_some())
    }

    /// Sets (`Some`) or removes (`None`) the profile picture of `user` from `source`.
    pub async fn set_avatar(
        &self,
        user: &UserId,
        source: AvatarSource,
        image: Option<&[u8]>,
        now_ms: u64,
    ) -> Result<(), StoreError> {
        match image {
            Some(image) => {
                sqlx::query(
                    "INSERT INTO avatars (user_id, source, image, updated_ms) VALUES (?, ?, ?, ?)
                     ON CONFLICT (user_id, source) DO UPDATE SET image = excluded.image,
                         updated_ms = excluded.updated_ms",
                )
                .bind(user.as_str())
                .bind(source.as_str())
                .bind(image)
                .bind(ms(now_ms))
                .execute(&self.pool)
                .await?;
            }
            None => {
                sqlx::query("DELETE FROM avatars WHERE user_id = ? AND source = ?")
                    .bind(user.as_str())
                    .bind(source.as_str())
                    .execute(&self.pool)
                    .await?;
            }
        }
        Ok(())
    }

    // [impl->dsn~display-names~2]
    /// Stores the identity provider's name of `user` (ORG-7). Returns whether the shown
    /// name changed (not while they chose their own).
    pub async fn set_provider_name(&self, user: &UserId, name: &str) -> Result<bool, StoreError> {
        let before = self.display_name(user).await?;
        sqlx::query(
            "INSERT INTO people (user_id, provider_name) VALUES (?, ?)
             ON CONFLICT (user_id) DO UPDATE SET provider_name = excluded.provider_name",
        )
        .bind(user.as_str())
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(before.as_deref() != Some(name)
            && self.display_name(user).await?.as_deref() == Some(name))
    }

    /// Stores `name` as the provider's name of `user` only if there is none yet (a name
    /// from a local account before the first login, ORG-7). Returns whether the shown name
    /// changed.
    pub async fn fill_provider_name(&self, user: &UserId, name: &str) -> Result<bool, StoreError> {
        let before = self.display_name(user).await?;
        sqlx::query(
            "INSERT INTO people (user_id, provider_name) VALUES (?, ?)
             ON CONFLICT (user_id) DO UPDATE SET provider_name = excluded.provider_name
             WHERE people.provider_name IS NULL",
        )
        .bind(user.as_str())
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(before != self.display_name(user).await?)
    }

    /// Whether some login already holds `user`.
    pub async fn has_identity(&self, user: &UserId) -> Result<bool, StoreError> {
        let found: Option<i64> = sqlx::query_scalar("SELECT 1 FROM identities WHERE user_id = ?")
            .bind(user.as_str())
            .fetch_optional(&self.pool)
            .await?;
        Ok(found.is_some())
    }

    /// Sets the name `user` chose for themselves; `None` clears it, and the provider's
    /// name shows again.
    pub async fn set_chosen_name(
        &self,
        user: &UserId,
        name: Option<&str>,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO people (user_id, chosen_name) VALUES (?, ?)
             ON CONFLICT (user_id) DO UPDATE SET chosen_name = excluded.chosen_name",
        )
        .bind(user.as_str())
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// The display name of `user`, if known: the chosen one, else the provider's.
    pub async fn display_name(&self, user: &UserId) -> Result<Option<String>, StoreError> {
        Ok(sqlx::query_scalar(
            "SELECT COALESCE(chosen_name, provider_name) FROM people WHERE user_id = ?",
        )
        .bind(user.as_str())
        .fetch_optional(&self.pool)
        .await?
        .flatten())
    }

    // [impl->dsn~organisation-roles~1]
    /// Stores a granted organisation role (replaces an earlier grant of it).
    pub async fn grant_organisation_role(
        &self,
        user: &UserId,
        role: &str,
        valid_until: Option<u64>,
        granted_by: &UserId,
        now_ms: u64,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "INSERT INTO organisation_roles (user_id, role, valid_until, granted_by, granted_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT (user_id, role) DO UPDATE SET valid_until = excluded.valid_until,
               granted_by = excluded.granted_by, granted_at = excluded.granted_at",
        )
        .bind(user.as_str())
        .bind(role)
        .bind(valid_until.map(ms))
        .bind(granted_by.as_str())
        .bind(ms(now_ms))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Removes a granted organisation role.
    pub async fn withdraw_organisation_role(
        &self,
        user: &UserId,
        role: &str,
    ) -> Result<(), StoreError> {
        sqlx::query("DELETE FROM organisation_roles WHERE user_id = ? AND role = ?")
            .bind(user.as_str())
            .bind(role)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// The granted organisation roles: person, role, end.
    pub async fn load_organisation_roles(
        &self,
    ) -> Result<Vec<(UserId, String, Option<u64>)>, StoreError> {
        let rows = sqlx::query("SELECT user_id, role, valid_until FROM organisation_roles")
            .fetch_all(&self.pool)
            .await?;
        rows.into_iter()
            .map(|r| {
                let user: String = r.get("user_id");
                let user = user
                    .parse()
                    .map_err(|e| StoreError::Invalid(format!("user id: {e}")))?;
                let until: Option<i64> = r.get("valid_until");
                Ok((
                    user,
                    r.get("role"),
                    until.map(|v| u64::try_from(v).unwrap_or_default()),
                ))
            })
            .collect()
    }

    // [impl->dsn~vehicle-master-data~1]
    /// Stores `vehicle`, in place of the one with the call sign `replaces` if given, in one
    /// transaction. Returns false, and changes nothing, if its call sign is already taken by
    /// another vehicle or `replaces` names none.
    pub async fn save_vehicle(
        &self,
        replaces: Option<&str>,
        vehicle: &ordo_sync::message::Vehicle,
        by: &UserId,
        now_ms: u64,
    ) -> Result<bool, StoreError> {
        let mut tx = self.pool.begin().await?;
        if let Some(old) = replaces {
            let gone = sqlx::query("DELETE FROM vehicles WHERE unit = ?")
                .bind(old)
                .execute(&mut *tx)
                .await?
                .rows_affected();
            if gone == 0 {
                return Ok(false);
            }
        }
        let [leaders, sub_leaders, crew] = vehicle.strength.map_or([None; 3], |s| s.map(Some));
        let added = sqlx::query(
            "INSERT INTO vehicles (unit, unit_type, organisation, service, leaders, sub_leaders,
               crew, changed_by, changed_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT (unit) DO NOTHING",
        )
        .bind(&vehicle.unit)
        .bind(&vehicle.unit_type)
        .bind(&vehicle.organisation)
        .bind(&vehicle.service)
        .bind(leaders.map(i64::from))
        .bind(sub_leaders.map(i64::from))
        .bind(crew.map(i64::from))
        .bind(by.as_str())
        .bind(ms(now_ms))
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if added == 0 {
            return Ok(false);
        }
        tx.commit().await?;
        Ok(true)
    }

    /// Removes the vehicle with call sign `unit`; returns whether there was one.
    pub async fn remove_vehicle(&self, unit: &str) -> Result<bool, StoreError> {
        let gone = sqlx::query("DELETE FROM vehicles WHERE unit = ?")
            .bind(unit)
            .execute(&self.pool)
            .await?
            .rows_affected();
        Ok(gone > 0)
    }

    /// The vehicles added in ORDO, by call sign.
    pub async fn load_vehicles(&self) -> Result<Vec<ordo_sync::message::Vehicle>, StoreError> {
        let rows = sqlx::query(
            "SELECT unit, unit_type, organisation, service, leaders, sub_leaders, crew
             FROM vehicles ORDER BY unit",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let part = |name: &str| {
                    r.get::<Option<i64>, _>(name)
                        .and_then(|v| u32::try_from(v).ok())
                };
                let strength = match [part("leaders"), part("sub_leaders"), part("crew")] {
                    [Some(a), Some(b), Some(c)] => Some([a, b, c]),
                    _ => None,
                };
                ordo_sync::message::Vehicle {
                    unit: r.get("unit"),
                    unit_type: r.get("unit_type"),
                    organisation: r.get("organisation"),
                    service: r.get("service"),
                    strength,
                    fixed: false,
                }
            })
            .collect())
    }

    /// The profile picture of `user`: uploaded in ORDO, else from the identity provider,
    /// else from Gravatar.
    pub async fn avatar(&self, user: &UserId) -> Result<Option<Vec<u8>>, StoreError> {
        let row = sqlx::query(
            "SELECT image FROM avatars WHERE user_id = ?
             ORDER BY CASE source WHEN 'ordo' THEN 0 WHEN 'provider' THEN 1 ELSE 2 END
             LIMIT 1",
        )
        .bind(user.as_str())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.get("image")))
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

    /// Stores a paired device whose login lasts until `valid_until` (ms), with its name
    /// (ORG-8; none keeps the stored one). Pairing it again replaces its key, but only for
    /// the same user: a device ID of another person cannot be taken over (returns `false`).
    pub async fn save_device(
        &self,
        device: &DeviceId,
        user: &UserId,
        key: &[u8; 32],
        name: Option<&str>,
        now_ms: u64,
        valid_until: u64,
    ) -> Result<bool, StoreError> {
        let saved = sqlx::query(
            "INSERT INTO devices (device_id, user_id, public_key, paired_ms, valid_until, name)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT (device_id) DO UPDATE SET public_key = excluded.public_key,
                 paired_ms = excluded.paired_ms, valid_until = excluded.valid_until,
                 name = COALESCE(excluded.name, devices.name), revoked_ms = NULL
             WHERE devices.user_id = excluded.user_id",
        )
        .bind(device.ulid().to_string())
        .bind(user.as_str())
        .bind(key.as_slice())
        .bind(ms(now_ms))
        .bind(ms(valid_until))
        .bind(name)
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

    /// When each device was paired (ms), and its name (ORG-8).
    pub async fn load_paired(&self) -> Result<Vec<(DeviceId, u64, Option<String>)>, StoreError> {
        let rows =
            sqlx::query("SELECT device_id, paired_ms, name FROM devices WHERE revoked_ms IS NULL")
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
                    row.get("name"),
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

    /// The ORDO user of a login at the identity provider, if they ever logged in.
    pub async fn user_of_identity(
        &self,
        issuer: &str,
        subject: &str,
    ) -> Result<Option<UserId>, StoreError> {
        let user: Option<String> =
            sqlx::query_scalar("SELECT user_id FROM identities WHERE issuer = ? AND subject = ?")
                .bind(issuer)
                .bind(subject)
                .fetch_optional(&self.pool)
                .await?;
        user.map(|u| {
            u.parse()
                .map_err(|e| StoreError::Invalid(format!("user id: {e}")))
        })
        .transpose()
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

/// Where a profile picture comes from (ADR-0047), in the order they are preferred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvatarSource {
    /// Uploaded by the person in ORDO.
    Ordo,
    /// The identity provider's `picture` claim.
    Provider,
    /// Gravatar, by the hash of the e-mail address.
    Gravatar,
}

impl AvatarSource {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ordo => "ordo",
            Self::Provider => "provider",
            Self::Gravatar => "gravatar",
        }
    }
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
    async fn the_schema_is_migrated_once_also_for_databases_from_before_the_first_release() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ordo.db");
        // A database as the server made it before migrations: tables, then added columns.
        {
            let options = SqliteConnectOptions::new()
                .filename(&path)
                .create_if_missing(true);
            let pool = SqlitePoolOptions::new()
                .connect_with(options)
                .await
                .unwrap();
            for statement in [
                "CREATE TABLE events (incident TEXT NOT NULL, compartment TEXT NOT NULL, seq INTEGER NOT NULL, event BLOB NOT NULL, PRIMARY KEY (incident, compartment, seq)) STRICT",
                "CREATE TABLE devices (device_id TEXT PRIMARY KEY, user_id TEXT NOT NULL, public_key BLOB NOT NULL, paired_ms INTEGER NOT NULL) STRICT",
                "ALTER TABLE devices ADD COLUMN valid_until INTEGER",
                "ALTER TABLE devices ADD COLUMN revoked_ms INTEGER",
                "INSERT INTO events VALUES ('i', 'incident', 1, x'00')",
            ] {
                sqlx::query(statement).execute(&pool).await.unwrap();
            }
            pool.close().await;
        }
        for _ in 0..2 {
            let store = Store::open(&path).await.unwrap();
            let applied: Vec<i64> = sqlx::query_scalar("SELECT version FROM _sqlx_migrations")
                .fetch_all(&store.pool)
                .await
                .unwrap();
            assert_eq!(applied, [1, 2, 3, 4, 5], "each migration, applied once");
            let events: i64 = sqlx::query_scalar("SELECT count(*) FROM events")
                .fetch_one(&store.pool)
                .await
                .unwrap();
            assert_eq!(events, 1, "stored events stay");
            store.pool.close().await;
        }
    }

    // [utest->dsn~organisation-roles~1]
    #[tokio::test]
    async fn organisation_roles_are_stored_replaced_and_withdrawn() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("ordo.db")).await.unwrap();
        let (ben, dora) = (user("ben"), user("dora"));
        store
            .grant_organisation_role(&ben, "planner", Some(5_000), &dora, 1_000)
            .await
            .unwrap();
        store
            .grant_organisation_role(&ben, "planner", None, &dora, 2_000)
            .await
            .unwrap();
        assert_eq!(
            store.load_organisation_roles().await.unwrap(),
            [(ben.clone(), "planner".to_owned(), None)],
            "a new grant replaces the earlier one"
        );
        store
            .withdraw_organisation_role(&ben, "planner")
            .await
            .unwrap();
        assert!(store.load_organisation_roles().await.unwrap().is_empty());
    }

    // [utest->dsn~vehicle-master-data~1]
    #[tokio::test]
    async fn vehicles_are_added_renamed_and_removed_with_unique_call_signs() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("ordo.db")).await.unwrap();
        let dora = user("dora");
        let vehicle = |unit: &str, strength| ordo_sync::message::Vehicle {
            unit: unit.into(),
            unit_type: "RTW".into(),
            organisation: "DRK".into(),
            service: Some("rescue".into()),
            strength,
            fixed: false,
        };
        assert!(
            store
                .save_vehicle(None, &vehicle("RTW 1", Some([0, 1, 1])), &dora, 1)
                .await
                .unwrap()
        );
        assert!(
            store
                .save_vehicle(None, &vehicle("RTW 2", None), &dora, 1)
                .await
                .unwrap()
        );
        assert!(
            !store
                .save_vehicle(None, &vehicle("RTW 1", None), &dora, 2)
                .await
                .unwrap(),
            "a call sign is taken once"
        );
        assert!(
            !store
                .save_vehicle(Some("RTW 1"), &vehicle("RTW 2", None), &dora, 2)
                .await
                .unwrap(),
            "nor renamed onto another"
        );
        assert!(
            !store
                .save_vehicle(Some("RTW 9"), &vehicle("RTW 9", None), &dora, 2)
                .await
                .unwrap(),
            "only an existing one is changed"
        );
        assert!(
            store
                .save_vehicle(Some("RTW 1"), &vehicle("RTW 3", Some([1, 0, 2])), &dora, 3)
                .await
                .unwrap()
        );
        assert_eq!(
            store.load_vehicles().await.unwrap(),
            [vehicle("RTW 2", None), vehicle("RTW 3", Some([1, 0, 2]))]
        );
        assert!(store.remove_vehicle("RTW 2").await.unwrap());
        assert!(!store.remove_vehicle("RTW 2").await.unwrap());
        assert_eq!(store.load_vehicles().await.unwrap().len(), 1);
    }

    // [utest->dsn~display-names~2]
    #[tokio::test]
    async fn an_account_name_fills_in_only_where_there_is_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("ordo.db")).await.unwrap();
        let (anna, ben) = (user("anna"), user("ben"));
        assert!(
            store
                .fill_provider_name(&anna, "Anna Schmidt")
                .await
                .unwrap()
        );
        assert_eq!(
            store.display_name(&anna).await.unwrap().as_deref(),
            Some("Anna Schmidt"),
            "before the first login"
        );
        assert!(!store.fill_provider_name(&anna, "Anna S.").await.unwrap());
        store.set_provider_name(&ben, "Ben Weber").await.unwrap();
        assert!(
            !store.fill_provider_name(&ben, "Benjamin").await.unwrap(),
            "a name from a login wins"
        );
        assert_eq!(
            store.display_name(&ben).await.unwrap().as_deref(),
            Some("Ben Weber")
        );
        assert!(!store.has_identity(&anna).await.unwrap());
    }

    // [utest->dsn~display-names~2]
    #[tokio::test]
    async fn a_chosen_display_name_wins_over_the_providers_while_set() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("ordo.db")).await.unwrap();
        let anna = user("anna");
        assert_eq!(store.display_name(&anna).await.unwrap(), None);
        store
            .set_provider_name(&anna, "Anna Schmidt")
            .await
            .unwrap();
        assert_eq!(
            store.display_name(&anna).await.unwrap().as_deref(),
            Some("Anna Schmidt")
        );
        store.set_chosen_name(&anna, Some("Anna S.")).await.unwrap();
        store
            .set_provider_name(&anna, "Anna Schmidt")
            .await
            .unwrap();
        assert_eq!(
            store.display_name(&anna).await.unwrap().as_deref(),
            Some("Anna S."),
            "a later login keeps the chosen name"
        );
        store.set_chosen_name(&anna, None).await.unwrap();
        assert_eq!(
            store.display_name(&anna).await.unwrap().as_deref(),
            Some("Anna Schmidt"),
            "cleared: the provider's name shows again"
        );
        assert!(
            !store
                .set_provider_name(&anna, "Anna Schmidt")
                .await
                .unwrap(),
            "the same name again is no change"
        );
        assert!(store.set_provider_name(&anna, "Anna Weber").await.unwrap());
    }

    #[tokio::test]
    async fn a_paired_device_cannot_be_taken_over_by_another_person() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("ordo.db")).await.unwrap();
        let device = DeviceId(Ulid::from_parts(1, 1).unwrap());
        assert!(
            store
                .save_device(
                    &device,
                    &user("anna"),
                    &[1; 32],
                    Some("Firefox auf Android"),
                    1,
                    100
                )
                .await
                .unwrap()
        );
        assert!(
            store
                .save_device(&device, &user("anna"), &[2; 32], None, 2, 200)
                .await
                .unwrap(),
            "the same person may pair the device again with a new key"
        );
        assert!(
            !store
                .save_device(&device, &user("mallory"), &[3; 32], Some("Mallory"), 3, 300)
                .await
                .unwrap(),
            "another person may not take the device ID over"
        );
        let devices = store.load_devices().await.unwrap();
        assert_eq!(devices, vec![(device, user("anna"), [2; 32], Some(200))]);
        // [utest->dsn~device-names~1]
        assert_eq!(
            store.load_paired().await.unwrap(),
            vec![(device, 2, Some("Firefox auf Android".to_owned()))],
            "logging in again without a name keeps it"
        );
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
                .save_device(
                    &device,
                    &user("anna"),
                    &[1; 32],
                    Some("Firefox auf Android"),
                    1,
                    100,
                )
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
    async fn the_own_picture_wins_over_provider_and_gravatar() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(&dir.path().join("ordo.db")).await.unwrap();
        let anna = user("anna");
        assert_eq!(store.avatar(&anna).await.unwrap(), None);
        store
            .set_avatar(&anna, AvatarSource::Gravatar, Some(b"g"), 1)
            .await
            .unwrap();
        assert_eq!(
            store.avatar(&anna).await.unwrap().as_deref(),
            Some(&b"g"[..])
        );
        store
            .set_avatar(&anna, AvatarSource::Provider, Some(b"p"), 2)
            .await
            .unwrap();
        assert_eq!(
            store.avatar(&anna).await.unwrap().as_deref(),
            Some(&b"p"[..])
        );
        store
            .set_avatar(&anna, AvatarSource::Ordo, Some(b"o"), 3)
            .await
            .unwrap();
        assert_eq!(
            store.avatar(&anna).await.unwrap().as_deref(),
            Some(&b"o"[..])
        );
        store
            .set_avatar(&anna, AvatarSource::Ordo, None, 4)
            .await
            .unwrap();
        assert_eq!(
            store.avatar(&anna).await.unwrap().as_deref(),
            Some(&b"p"[..]),
            "removing the own picture falls back to the provider's"
        );
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
