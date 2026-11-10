//! Attachments on the device (ADR-0033): the encrypted bytes are kept in the device
//! storage (`b/<hash>`), uploads wait in a queue (`uploads`) until the device is
//! connected, and attachments of other devices are downloaded when they are shown.
//!
//! Tickets for `PUT`/`GET /blobs/<hash>` are asked for over the sync connection; the
//! server grants them by the need-to-know rules of the attachment's stream. Without the
//! device database (no OPFS in workers) attachments are kept in memory only.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use js_sys::Uint8Array;
use leptos::prelude::*;
use ordo_core::codec::{self, fixed_bytes};
use ordo_core::{AttachmentRef, CompartmentId, IncidentId};
use ordo_crypto::attachment;
use ordo_sync::message::{BlobRequest, BlobTicket, ClientMessage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{RequestInit, Response};

use crate::core::Handle;
use crate::storage;

/// Storage row of the upload queue.
const QUEUE_KEY: &str = "uploads";
/// Wait before asking again after a failed transfer or a missing attachment.
const RETRY_MS: i32 = 10_000;

/// An attachment waiting for its upload.
#[derive(Clone, Serialize, Deserialize)]
struct Upload {
    incident: IncidentId,
    compartment: CompartmentId,
    #[serde(with = "fixed_bytes")]
    hash: [u8; 32],
    size: u64,
}

thread_local! {
    /// Ciphertext by hash, as far as it was used in this session.
    static CACHE: RefCell<HashMap<[u8; 32], Vec<u8>>> = RefCell::new(HashMap::new());
    static QUEUE: RefCell<Vec<Upload>> = const { RefCell::new(Vec::new()) };
    /// Tickets asked for and not answered yet (hash, upload).
    static ASKED: RefCell<HashSet<([u8; 32], bool)>> = RefCell::new(HashSet::new());
    /// Attachments looked for in the device storage.
    static LOOKED_UP: RefCell<HashSet<[u8; 32]>> = RefCell::new(HashSet::new());
    /// Attachments the server does not have (yet) or could not send.
    static UNAVAILABLE: RefCell<HashSet<[u8; 32]>> = RefCell::new(HashSet::new());
}

fn row(hash: &[u8; 32]) -> String {
    format!("b/{}", hex(hash))
}

fn hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

/// Loads the upload queue from the device storage (after it was opened).
pub async fn load() {
    if let Some(bytes) = storage::get_sealed(QUEUE_KEY).await
        && let Ok(queue) = codec::from_cbor::<Vec<Upload>>(&bytes)
    {
        QUEUE.set(queue);
    }
}

fn save_queue() {
    QUEUE.with_borrow(|queue| {
        if let Ok(bytes) = codec::to_cbor(queue) {
            storage::put_sealed(QUEUE_KEY, &bytes);
        }
    });
}

/// Attachments not uploaded yet.
pub fn queued() -> usize {
    QUEUE.with_borrow(Vec::len)
}

// [impl->dsn~attachment-transfer~1]
/// Keeps a new attachment on this device and queues its upload.
pub fn add(
    core: Handle,
    incident: IncidentId,
    compartment: CompartmentId,
    reference: &AttachmentRef,
    data: Vec<u8>,
) {
    storage::put_raw(&row(&reference.hash), &data);
    CACHE.with_borrow_mut(|c| c.insert(reference.hash, data));
    QUEUE.with_borrow_mut(|q| {
        q.push(Upload {
            incident,
            compartment,
            hash: reference.hash,
            size: reference.size,
        });
    });
    save_queue();
    pump(core);
}

/// Asks for upload tickets of the queued attachments. Call outside `core` updates.
pub fn pump(core: Handle) {
    if !core.with_value(|c| c.replica.is_ready()) {
        return;
    }
    let requests: Vec<BlobRequest> = QUEUE.with_borrow(|queue| {
        queue
            .iter()
            .filter(|u| ASKED.with_borrow_mut(|a| a.insert((u.hash, true))))
            .map(|u| BlobRequest {
                incident: u.incident.clone(),
                compartment: u.compartment.clone(),
                hash: u.hash,
                size: u.size,
                upload: true,
            })
            .collect()
    });
    core.with_value(|c| {
        for request in requests {
            c.send(&ClientMessage::BlobTicket(request));
        }
    });
}

/// A new connection: questions of the old one are asked again.
pub fn on_welcome(core: Handle, revision: RwSignal<u64>) {
    ASKED.with_borrow_mut(HashSet::clear);
    UNAVAILABLE.with_borrow_mut(HashSet::clear);
    pump(core);
    revision.update(|r| *r += 1);
}

/// Runs `f` after `ms` milliseconds.
fn later(ms: i32, f: impl FnOnce() + 'static) {
    let callback = Closure::once_into_js(f);
    if let Some(window) = web_sys::window() {
        let _ = window
            .set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), ms);
    }
}

/// The server's answer to a ticket request.
pub fn on_ticket(core: Handle, revision: RwSignal<u64>, answer: BlobTicket) {
    let BlobTicket {
        hash,
        upload,
        ticket,
    } = answer;
    ASKED.with_borrow_mut(|a| a.remove(&(hash, upload)));
    let Some(ticket) = ticket else {
        if !upload {
            // Not uploaded yet (or not allowed): the preview stays; ask again later.
            unavailable(revision, hash);
        }
        return;
    };
    spawn_local(async move {
        if upload {
            if send(&hash, &ticket).await {
                QUEUE.with_borrow_mut(|q| q.retain(|u| u.hash != hash));
                save_queue();
            } else {
                later(RETRY_MS, move || pump(core));
            }
        } else {
            match receive(&hash, &ticket).await {
                Some(data) => {
                    storage::put_raw(&row(&hash), &data);
                    CACHE.with_borrow_mut(|c| c.insert(hash, data));
                }
                None => unavailable(revision, hash),
            }
        }
        revision.update(|r| *r += 1);
    });
}

fn unavailable(revision: RwSignal<u64>, hash: [u8; 32]) {
    UNAVAILABLE.with_borrow_mut(|u| u.insert(hash));
    revision.update(|r| *r += 1);
    later(RETRY_MS, move || {
        UNAVAILABLE.with_borrow_mut(|u| u.remove(&hash));
        revision.update(|r| *r += 1);
    });
}

async fn fetch(hash: &[u8; 32], ticket: &str, body: Option<&[u8]>) -> Option<Response> {
    let headers = crate::core::api_headers()?;
    headers
        .set("Authorization", &format!("Bearer {ticket}"))
        .ok()?;
    let init = RequestInit::new();
    init.set_headers(&headers);
    if let Some(body) = body {
        init.set_method("PUT");
        init.set_body(&Uint8Array::from(body).into());
    }
    let url = format!("/blobs/{}", hex(hash));
    let promise = web_sys::window()?.fetch_with_str_and_init(&url, &init);
    let response: Response = JsFuture::from(promise).await.ok()?.dyn_into().ok()?;
    response.ok().then_some(response)
}

/// Uploads a queued attachment; `true` once the server has it.
async fn send(hash: &[u8; 32], ticket: &str) -> bool {
    let data = match CACHE.with_borrow(|c| c.get(hash).cloned()) {
        Some(data) => Some(data),
        None => storage::get_raw(&row(hash)).await,
    };
    let Some(data) = data else {
        // Lost on this device (e.g. kept in memory only): nothing left to send.
        return true;
    };
    fetch(hash, ticket, Some(&data)).await.is_some()
}

/// Downloads an attachment and checks that it is the one asked for.
async fn receive(hash: &[u8; 32], ticket: &str) -> Option<Vec<u8>> {
    let response = fetch(hash, ticket, None).await?;
    let buffer = JsFuture::from(response.array_buffer().ok()?).await.ok()?;
    let data = Uint8Array::new(&buffer).to_vec();
    (attachment::hash(&data) == *hash).then_some(data)
}

/// What the device knows about an attachment it wants to show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Availability {
    /// Decrypted content.
    Ready(Vec<u8>),
    /// Being looked for or downloaded.
    Loading,
    /// The server does not have it (yet).
    Unavailable,
}

// [impl->dsn~attachment-transfer~1]
/// The attachment's content if this device has it; otherwise starts looking for it (device
/// storage, then the server) and bumps `revision` when something changed.
pub fn get(
    core: Handle,
    revision: RwSignal<u64>,
    compartment: &str,
    reference: &AttachmentRef,
) -> Availability {
    let hash = reference.hash;
    if let Some(data) = CACHE.with_borrow(|c| c.get(&hash).cloned()) {
        return attachment::open(reference, &data)
            .map_or(Availability::Unavailable, Availability::Ready);
    }
    if UNAVAILABLE.with_borrow(|u| u.contains(&hash)) {
        return Availability::Unavailable;
    }
    if LOOKED_UP.with_borrow_mut(|l| l.insert(hash)) {
        spawn_local(async move {
            if let Some(data) = storage::get_raw(&row(&hash)).await {
                CACHE.with_borrow_mut(|c| c.insert(hash, data));
            }
            revision.update(|r| *r += 1);
        });
        return Availability::Loading;
    }
    if !ASKED.with_borrow(|a| a.contains(&(hash, false))) {
        let request = core.with_value(|c| {
            let incident = c.incident.clone()?;
            let compartment = compartment.parse::<CompartmentId>().ok()?;
            c.replica.is_ready().then_some(BlobRequest {
                incident,
                compartment,
                hash,
                size: reference.size,
                upload: false,
            })
        });
        if let Some(request) = request {
            ASKED.with_borrow_mut(|a| a.insert((hash, false)));
            core.with_value(|c| c.send(&ClientMessage::BlobTicket(request)));
        }
    }
    Availability::Loading
}

/// Forgets everything (unpairing, remote wipe; the device storage is cleared separately).
pub fn clear() {
    CACHE.with_borrow_mut(HashMap::clear);
    QUEUE.with_borrow_mut(Vec::clear);
    ASKED.with_borrow_mut(HashSet::clear);
    LOOKED_UP.with_borrow_mut(HashSet::clear);
    UNAVAILABLE.with_borrow_mut(HashSet::clear);
}
