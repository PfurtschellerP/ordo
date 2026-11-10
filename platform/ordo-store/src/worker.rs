//! The worker's message loop and SQLite access.

use std::cell::RefCell;

use js_sys::{Array, Uint8Array};
use rusqlite::{Connection, params};
use sqlite_wasm_rs::WasmOsCallback;
use sqlite_wasm_vfs::sahpool::{OpfsSAHPoolCfg, install};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

thread_local! {
    static DB: RefCell<Option<Connection>> = const { RefCell::new(None) };
}

fn scope() -> DedicatedWorkerGlobalScope {
    js_sys::global().unchecked_into()
}

fn reply(id: &JsValue, result: Result<Option<JsValue>, String>) {
    let message = match result {
        Ok(value) => Array::of3(id, &"ok".into(), &value.unwrap_or(JsValue::NULL)),
        Err(error) => Array::of3(id, &"error".into(), &error.into()),
    };
    let _ = scope().post_message(&message);
}

/// Installs the message handler.
pub fn start() {
    let handler = Closure::<dyn FnMut(MessageEvent)>::new(|event: MessageEvent| {
        let message: Array = event.data().unchecked_into();
        let op = message.get(0).as_string().unwrap_or_default();
        let id = message.get(1);
        if op == "open" {
            wasm_bindgen_futures::spawn_local(async move {
                reply(&id, open().await.map(|()| None));
            });
            return;
        }
        let result = DB.with_borrow_mut(|db| {
            let db = db.as_mut().ok_or("database not open")?;
            match op.as_str() {
                "load" => load(db, &message.get(2).as_string().unwrap_or_default()).map(Some),
                "write" => write(db, &message.get(2).into(), &message.get(3).into()).map(|()| None),
                "clear" => db
                    .execute("DELETE FROM kv", [])
                    .map(|_| None)
                    .map_err(|e| e.to_string()),
                other => Err(format!("unknown operation {other}")),
            }
        });
        reply(&id, result);
    });
    scope().set_onmessage(Some(handler.as_ref().unchecked_ref()));
    handler.forget();
    // Messages sent before this point had no listener: tell the app it may start now.
    let _ = scope().post_message(&Array::of1(&"ready".into()));
}

async fn open() -> Result<(), String> {
    install::<WasmOsCallback>(&OpfsSAHPoolCfg::default(), true)
        .await
        .map_err(|e| format!("OPFS: {e:?}"))?;
    let db = Connection::open("ordo.db").map_err(|e| e.to_string())?;
    db.execute_batch(
        "PRAGMA journal_mode = DELETE;
         CREATE TABLE IF NOT EXISTS kv (key TEXT PRIMARY KEY, value BLOB NOT NULL) WITHOUT ROWID;",
    )
    .map_err(|e| e.to_string())?;
    DB.set(Some(db));
    Ok(())
}

/// The first key after every key starting with `prefix` (for range queries).
fn after(prefix: &str) -> String {
    format!("{prefix}\u{10FFFF}")
}

fn load(db: &Connection, prefix: &str) -> Result<JsValue, String> {
    let mut statement = db
        .prepare("SELECT key, value FROM kv WHERE key >= ?1 AND key < ?2 ORDER BY key")
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map(params![prefix, after(prefix)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .map_err(|e| e.to_string())?;
    let entries = Array::new();
    for row in rows {
        let (key, value) = row.map_err(|e| e.to_string())?;
        entries.push(&Array::of2(
            &key.into(),
            &Uint8Array::from(value.as_slice()),
        ));
    }
    Ok(entries.into())
}

fn write(db: &mut Connection, puts: &Array, prefixes: &Array) -> Result<(), String> {
    let tx = db.transaction().map_err(|e| e.to_string())?;
    for prefix in prefixes.iter() {
        let prefix = prefix.as_string().unwrap_or_default();
        tx.execute(
            "DELETE FROM kv WHERE key >= ?1 AND key < ?2",
            params![prefix, after(&prefix)],
        )
        .map_err(|e| e.to_string())?;
    }
    for entry in puts.iter() {
        let entry: Array = entry.unchecked_into();
        let key = entry.get(0).as_string().unwrap_or_default();
        let value = Uint8Array::new(&entry.get(1)).to_vec();
        tx.execute(
            "INSERT INTO kv (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}
