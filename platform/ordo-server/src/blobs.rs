//! Attachments (ADR-0033): encrypted on the device, named by the BLAKE3 hash of their
//! ciphertext, stored in an object store; the server never sees plaintext.
//!
//! A device asks over its authenticated sync connection for a ticket (`blobTicket`); the
//! server grants it by the need-to-know rules of the stream the attachment belongs to.
//! The bytes then travel over HTTP: `PUT /blobs/<hash>` and `GET /blobs/<hash>` with
//! `Authorization: Bearer <ticket>`. An upload must match its hash and size.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use object_store::path::Path as StorePath;
use object_store::{ObjectStore, ObjectStoreExt};
use ordo_sync::StreamKey;

use crate::app::{App, now_ms};
use crate::config::S3Config;

/// How long a ticket is valid.
const TICKET_TTL: Duration = Duration::from_secs(10 * 60);
/// Most tickets kept at once (old ones are dropped first).
const MAX_TICKETS: usize = 10_000;

/// A permission to upload or download one attachment.
#[derive(Clone)]
struct Ticket {
    hash: [u8; 32],
    size: u64,
    upload: bool,
    stream: StreamKey,
    expires: Instant,
}

/// The attachment store and the tickets issued for it.
pub struct Blobs {
    store: Arc<dyn ObjectStore>,
    /// Largest attachment in bytes.
    pub max_bytes: u64,
    tickets: Mutex<HashMap<String, Ticket>>,
}

/// Hex form of a hash.
pub fn hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

fn location(hash: &[u8; 32]) -> StorePath {
    let name = hex(hash);
    StorePath::from(format!("blobs/{}/{name}", &name[..2]))
}

impl Blobs {
    /// A store in the local directory `dir` (created if needed).
    pub fn local(dir: &std::path::Path, max_bytes: u64) -> Result<Self, std::io::Error> {
        std::fs::create_dir_all(dir)?;
        let store = LocalFileSystem::new_with_prefix(dir).map_err(std::io::Error::other)?;
        Ok(Self::with_store(Arc::new(store), max_bytes))
    }

    /// A store in an S3-compatible bucket, e.g. Garage. Checks once that the bucket answers.
    pub async fn s3(config: &S3Config, max_bytes: u64) -> Result<Self, std::io::Error> {
        // TLS for the S3 client uses ring, like the rest of the server (no aws-lc).
        let _ = rustls::crypto::ring::default_provider().install_default();
        let store = AmazonS3Builder::new()
            .with_endpoint(&config.endpoint)
            .with_allow_http(config.endpoint.starts_with("http://"))
            .with_bucket_name(&config.bucket)
            .with_region(&config.region)
            .with_access_key_id(&config.access_key_id)
            .with_secret_access_key(&config.secret_access_key)
            .build()
            .map_err(std::io::Error::other)?;
        match store.head(&StorePath::from("blobs/.check")).await {
            Ok(_) | Err(object_store::Error::NotFound { .. }) => {}
            Err(error) => {
                return Err(std::io::Error::other(format!(
                    "the S3 store {} (bucket {}) does not answer: {error}",
                    config.endpoint, config.bucket
                )));
            }
        }
        Ok(Self::with_store(Arc::new(store), max_bytes))
    }

    fn with_store(store: Arc<dyn ObjectStore>, max_bytes: u64) -> Self {
        Self {
            store,
            max_bytes,
            tickets: Mutex::new(HashMap::new()),
        }
    }

    /// Issues a ticket; the caller has checked the permission.
    pub fn issue(&self, hash: [u8; 32], size: u64, upload: bool, stream: StreamKey) -> String {
        let mut token = [0u8; 32];
        getrandom::fill(&mut token).expect("the operating system provides randomness");
        let token = hex(&token);
        let now = Instant::now();
        let mut tickets = self.tickets.lock().expect("ticket lock");
        tickets.retain(|_, t| t.expires > now);
        if tickets.len() >= MAX_TICKETS
            && let Some(oldest) = tickets
                .iter()
                .min_by_key(|(_, t)| t.expires)
                .map(|(k, _)| k.clone())
        {
            tickets.remove(&oldest);
        }
        tickets.insert(
            token.clone(),
            Ticket {
                hash,
                size,
                upload,
                stream,
                expires: now + TICKET_TTL,
            },
        );
        token
    }

    /// The valid ticket from the request for this attachment and direction. Upload tickets
    /// are used once.
    fn ticket(&self, headers: &HeaderMap, hash: &[u8; 32], upload: bool) -> Option<Ticket> {
        let token = headers
            .get(header::AUTHORIZATION)?
            .to_str()
            .ok()?
            .strip_prefix("Bearer ")?;
        let mut tickets = self.tickets.lock().expect("ticket lock");
        let ticket = tickets.get(token)?.clone();
        if ticket.hash != *hash || ticket.upload != upload || ticket.expires <= Instant::now() {
            return None;
        }
        if upload {
            tickets.remove(token);
        }
        Some(ticket)
    }
}

fn parse_hash(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut hash = [0u8; 32];
    for (i, byte) in hash.iter_mut().enumerate() {
        *byte = u8::from_str_radix(text.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(hash)
}

// [impl->dsn~attachments~1]
/// `PUT /blobs/<hash>`: stores an attachment whose content matches its name.
pub async fn put(
    State(app): State<Arc<App>>,
    Path(name): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(hash) = parse_hash(&name) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(ticket) = app.blobs.ticket(&headers, &hash, true) else {
        return StatusCode::FORBIDDEN.into_response();
    };
    if body.len() as u64 != ticket.size || *blake3::hash(&body).as_bytes() != hash {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            "content does not match its hash\n",
        )
            .into_response();
    }
    if let Err(error) = app.blobs.store.put(&location(&hash), body.into()).await {
        tracing::error!(%error, "storing an attachment failed");
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    match app
        .store
        .add_blob(&hex(&hash), &ticket.stream, ticket.size, now_ms())
        .await
    {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(error) => {
            tracing::error!(%error, "recording an attachment failed");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

// [impl->dsn~attachments~1]
/// `GET /blobs/<hash>`: the encrypted attachment.
pub async fn get(
    State(app): State<Arc<App>>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Response {
    let Some(hash) = parse_hash(&name) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if app.blobs.ticket(&headers, &hash, false).is_none() {
        return StatusCode::FORBIDDEN.into_response();
    }
    let bytes = match app.blobs.store.get(&location(&hash)).await {
        Ok(result) => result.bytes().await,
        Err(object_store::Error::NotFound { .. }) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => Err(e),
    };
    match bytes {
        Ok(bytes) => (
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/octet-stream"),
                ),
                (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
            ],
            bytes,
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "reading an attachment failed");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_are_64_hex_characters() {
        let hash = [0xab; 32];
        assert_eq!(parse_hash(&hex(&hash)), Some(hash));
        assert_eq!(parse_hash("ab"), None);
        assert_eq!(parse_hash(&"zz".repeat(32)), None);
        assert_eq!(location(&hash).as_ref(), format!("blobs/ab/{}", hex(&hash)));
    }

    // [utest->dsn~attachments~1]
    /// Against a real S3-compatible store (Garage in CI, `deploy/garage`): runs when
    /// `ORDO_TEST_S3_ENDPOINT`, `…_BUCKET`, `…_ACCESS_KEY_ID` and `…_SECRET_ACCESS_KEY`
    /// are set, otherwise it only says that it was skipped.
    #[tokio::test]
    async fn an_s3_store_keeps_attachments() {
        let var = |name: &str| std::env::var(format!("ORDO_TEST_S3_{name}")).ok();
        let (Some(endpoint), Some(bucket), Some(access_key_id), Some(secret_access_key)) = (
            var("ENDPOINT"),
            var("BUCKET"),
            var("ACCESS_KEY_ID"),
            var("SECRET_ACCESS_KEY"),
        ) else {
            eprintln!("skipped: ORDO_TEST_S3_* not set");
            return;
        };
        let config = S3Config {
            endpoint,
            bucket,
            region: "garage".into(),
            access_key_id,
            secret_access_key,
        };
        let blobs = Blobs::s3(&config, 1024).await.expect("the store answers");
        let mut data = [0u8; 600];
        getrandom::fill(&mut data).unwrap();
        let path = location(blake3::hash(&data).as_bytes());
        blobs
            .store
            .put(&path, data.to_vec().into())
            .await
            .expect("stored");
        let back = blobs.store.get(&path).await.expect("found");
        assert_eq!(back.bytes().await.unwrap().as_ref(), data);
        let missing = location(&[0; 32]);
        assert!(matches!(
            blobs.store.get(&missing).await,
            Err(object_store::Error::NotFound { .. })
        ));
        let wrong = S3Config {
            secret_access_key: "wrong".into(),
            ..config
        };
        assert!(Blobs::s3(&wrong, 1024).await.is_err(), "wrong secret");
    }
}
