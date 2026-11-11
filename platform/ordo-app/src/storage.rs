//! Device storage (ADR-0034): the replica in SQLite on the browser's file system (OPFS),
//! kept by the `ordo-store` web worker. Every value is encrypted with the vault's storage
//! key before it leaves this thread (ADR-0041), so the worker only sees ciphertext.
//!
//! Rows: `state` (the replica without its events) and one row per event,
//! `e/<incident>/<compartment>/<seq>`. After a change only the new events and the state
//! are written. Browsers without OPFS in workers keep using `localStorage` (see `core`).

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};

use js_sys::{Array, Function, Promise, Uint8Array};
use ordo_core::SequencedEvent;
use ordo_sync::message;
use ordo_sync::{Replica, ReplicaState, StoredStateError, StreamKey};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{MessageEvent, Worker, WorkerOptions, WorkerType};

use crate::vault;

/// The worker's script, built by Trunk from `platform/ordo-store`.
const WORKER_SCRIPT: &str = "/ordo-store_loader.js";
const STATE: &str = "state";

thread_local! {
    static WORKER: RefCell<Option<Worker>> = const { RefCell::new(None) };
    static PENDING: RefCell<HashMap<u32, (Function, Function)>> = RefCell::new(HashMap::new());
    static NEXT: Cell<u32> = const { Cell::new(1) };
    /// Last stored sequence number per stream.
    static SAVED: RefCell<HashMap<StreamKey, u64>> = RefCell::new(HashMap::new());
    static LOADED: RefCell<Result<Option<ReplicaState>, StoredStateError>> = const { RefCell::new(Ok(None)) };
    /// The worker refused a write (storage full or blocked).
    static FAILED: Cell<bool> = const { Cell::new(false) };
    /// Called once the first write succeeded (moving data out of `localStorage`).
    static ON_FIRST_WRITE: RefCell<Option<Box<dyn FnOnce()>>> = const { RefCell::new(None) };
    /// Resolves the wait for the worker's `ready`.
    static READY: RefCell<Option<Function>> = const { RefCell::new(None) };
}

/// How long opening the storage may take before the app falls back to `localStorage`.
const OPEN_TIMEOUT_MS: i32 = 10_000;

/// Whether the worker storage is in use.
pub fn active() -> bool {
    WORKER.with_borrow(Option::is_some)
}

fn event_key(stream: &StreamKey, seq: u64) -> String {
    format!("{}{seq:020}", stream_prefix(stream))
}

fn stream_prefix(stream: &StreamKey) -> String {
    format!("e/{}/{}/", stream.0, stream.1)
}

/// Sends a request to the worker and waits for its answer.
async fn call(op: &str, args: &[JsValue]) -> Result<JsValue, String> {
    let id = NEXT.get();
    NEXT.set(id.wrapping_add(1));
    let message = Array::new();
    message.push(&op.into());
    message.push(&id.into());
    for a in args {
        message.push(a);
    }
    let promise = Promise::new(&mut |resolve, reject| {
        PENDING.with_borrow_mut(|p| p.insert(id, (resolve, reject)));
    });
    WORKER
        .with_borrow(|w| w.as_ref().map(|w| w.post_message(&message)))
        .ok_or("storage worker not running")?
        .map_err(|e| format!("{e:?}"))?;
    JsFuture::from(promise)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))
}

/// Starts the worker, opens the database and loads the stored replica. Fails when the
/// browser cannot run it (then `localStorage` is used).
pub async fn open() -> Result<(), String> {
    let opened = Promise::race(&Array::of2(
        &wasm_bindgen_futures::future_to_promise(async {
            open_worker()
                .await
                .map(|()| JsValue::TRUE)
                .map_err(JsValue::from)
        }),
        &Promise::new(&mut |_, reject| {
            if let Some(window) = web_sys::window() {
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_1(
                    &reject,
                    OPEN_TIMEOUT_MS,
                    &"device storage did not answer".into(),
                );
            }
        }),
    ));
    match JsFuture::from(opened).await {
        Ok(_) => Ok(()),
        Err(e) => {
            stop();
            Err(e.as_string().unwrap_or_else(|| format!("{e:?}")))
        }
    }
}

fn stop() {
    WORKER.with_borrow_mut(|w| {
        if let Some(w) = w.take() {
            w.terminate();
        }
    });
}

async fn open_worker() -> Result<(), String> {
    let options = WorkerOptions::new();
    options.set_type(WorkerType::Module);
    let worker = Worker::new_with_options(WORKER_SCRIPT, &options).map_err(|e| format!("{e:?}"))?;
    let ready = Promise::new(&mut |resolve, _| READY.set(Some(resolve)));
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(|event: MessageEvent| {
        let answer: Array = event.data().unchecked_into();
        if answer.get(0).as_string().as_deref() == Some("ready") {
            if let Some(resolve) = READY.take() {
                let _ = resolve.call0(&JsValue::NULL);
            }
            return;
        }
        let Some(id) = answer.get(0).as_f64().map(|f| f as u32) else {
            return;
        };
        if let Some((resolve, reject)) = PENDING.with_borrow_mut(|p| p.remove(&id)) {
            if answer.get(1).as_string().as_deref() == Some("ok") {
                let _ = resolve.call1(&JsValue::NULL, &answer.get(2));
            } else {
                let _ = reject.call1(&JsValue::NULL, &answer.get(2));
            }
        }
    });
    worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
    WORKER.set(Some(worker));
    JsFuture::from(ready).await.map_err(|e| format!("{e:?}"))?;
    call("open", &[]).await?;
    // Attachments (`b/…`) are read when needed, not at start.
    let rows: Array = call("load", &[STATE.into()]).await?.unchecked_into();
    let events: Array = call("load", &["e/".into()]).await?.unchecked_into();
    let state = assemble(&rows.concat(&events));
    if let Ok(Some(state)) = &state {
        SAVED.set(
            state
                .streams
                .iter()
                .map(|(k, e)| (k.clone(), e.last().map_or(0, |e| e.seq)))
                .collect(),
        );
    }
    LOADED.set(state);
    Ok(())
}

/// The replica state found when the storage was opened (once).
pub fn take_loaded() -> Result<Option<ReplicaState>, StoredStateError> {
    LOADED.replace(Ok(None))
}

/// Runs `f` after the next successful write (e.g. to remove data that moved here).
pub fn after_first_write(f: impl FnOnce() + 'static) {
    ON_FIRST_WRITE.set(Some(Box::new(f)));
}

fn decrypt(key: &str, value: &JsValue) -> Option<Vec<u8>> {
    let text = String::from_utf8(Uint8Array::new(value).to_vec()).ok()?;
    vault::decrypt(key, &text)
}

fn encrypted(key: &str, bytes: &[u8]) -> Option<JsValue> {
    let text = vault::encrypt(key, bytes)?;
    Some(Uint8Array::from(text.as_bytes()).into())
}

/// The replica state from the stored rows; `None` without a state row. A state that
/// cannot be used (written by a newer version, or damaged) is an error, never "none": the
/// device must not start empty and overwrite it.
fn assemble(rows: &Array) -> Result<Option<ReplicaState>, StoredStateError> {
    let mut state: Option<ReplicaState> = None;
    let mut events: BTreeMap<StreamKey, Vec<SequencedEvent>> = BTreeMap::new();
    for row in rows.iter() {
        let row: Array = row.unchecked_into();
        let Some(key) = row.get(0).as_string() else {
            continue;
        };
        let Some(bytes) = decrypt(&key, &row.get(1)) else {
            continue; // written with an earlier storage key: unreadable on purpose
        };
        if key == STATE {
            state = Some(ReplicaState::decode(&bytes)?);
        } else if key.starts_with("e/") {
            let Ok(event) = message::decode::<SequencedEvent>(&bytes) else {
                continue;
            };
            let Ok(body) = event.event.decode_body() else {
                continue;
            };
            events
                .entry((body.incident, body.compartment))
                .or_default()
                .push(event);
        }
    }
    let Some(mut state) = state else {
        return Ok(None);
    };
    for (stream, list) in &mut state.streams {
        if let Some(stored) = events.remove(stream) {
            *list = stored;
        }
    }
    state.streams.extend(events);
    Ok(Some(state))
}

// [impl->dsn~device-storage~1]
/// Writes what changed since the last save: new events and the state; deletes streams the
/// device no longer has (e.g. after a role was withdrawn). Fire and forget: a refused
/// write is reported by the next save (`Err`), which the app shows.
pub fn save(replica: &Replica) -> Result<(), ()> {
    let puts = Array::new();
    let deletes = Array::new();
    let heads = replica.stream_heads();
    let mut saved = SAVED.take();
    for (stream, head) in &heads {
        let from = saved.get(stream).copied().unwrap_or(0);
        for event in replica.events_after(stream, from) {
            let key = event_key(stream, event.seq);
            let bytes = message::encode(event).map_err(|_| ())?;
            puts.push(&Array::of2(
                &key.clone().into(),
                &encrypted(&key, &bytes).ok_or(())?,
            ));
        }
        saved.insert(stream.clone(), *head);
    }
    saved.retain(|stream, _| {
        let keep = heads.iter().any(|(s, _)| s == stream);
        if !keep {
            deletes.push(&stream_prefix(stream).into());
        }
        keep
    });
    SAVED.set(saved);
    let state = message::encode(&replica.snapshot_without_events()).map_err(|_| ())?;
    puts.push(&Array::of2(
        &STATE.into(),
        &encrypted(STATE, &state).ok_or(())?,
    ));
    wasm_bindgen_futures::spawn_local(async move {
        match call("write", &[puts.into(), deletes.into()]).await {
            Ok(_) => {
                FAILED.set(false);
                if let Some(f) = ON_FIRST_WRITE.take() {
                    f();
                }
            }
            Err(error) => {
                FAILED.set(true);
                web_sys::console::error_1(&format!("ORDO storage: {error}").into());
            }
        }
    });
    if FAILED.get() { Err(()) } else { Ok(()) }
}

/// Stores bytes as they are (attachments, which are encrypted already). Fire and forget.
pub fn put_raw(key: &str, bytes: &[u8]) {
    if !active() {
        return;
    }
    let puts = Array::of1(&Array::of2(&key.into(), &Uint8Array::from(bytes).into()));
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = call("write", &[puts.into(), Array::new().into()]).await {
            web_sys::console::error_1(&format!("ORDO storage: {error}").into());
        }
    });
}

/// The bytes stored under `key` with [`put_raw`].
pub async fn get_raw(key: &str) -> Option<Vec<u8>> {
    if !active() {
        return None;
    }
    let rows: Array = call("load", &[key.into()]).await.ok()?.unchecked_into();
    rows.iter().find_map(|row| {
        let row: Array = row.unchecked_into();
        (row.get(0).as_string().as_deref() == Some(key))
            .then(|| Uint8Array::new(&row.get(1)).to_vec())
    })
}

/// Stores bytes encrypted with the storage key. Fire and forget.
pub fn put_sealed(key: &str, bytes: &[u8]) {
    if let Some(value) = encrypted(key, bytes) {
        let text = Uint8Array::new(&value).to_vec();
        put_raw(key, &text);
    }
}

/// The bytes stored under `key` with [`put_sealed`].
pub async fn get_sealed(key: &str) -> Option<Vec<u8>> {
    let value = get_raw(key).await?;
    decrypt(key, &Uint8Array::from(value.as_slice()).into())
}

/// Deletes everything stored by the worker (unpairing, remote wipe).
pub fn clear() {
    SAVED.set(HashMap::new());
    if active() {
        wasm_bindgen_futures::spawn_local(async {
            let _ = call("clear", &[]).await;
        });
    }
}
