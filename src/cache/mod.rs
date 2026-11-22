mod disk;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const TTL_SECS: u64 = 900;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    ts: u64,
    latest: String,
    license: String,
}

pub type Map = HashMap<String, Entry>;

static STORE: OnceLock<Mutex<Map>> = OnceLock::new();
static ENABLED: OnceLock<bool> = OnceLock::new();

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn enabled() -> bool {
    ENABLED.get().copied().unwrap_or(false)
}

pub fn init(on: bool) {
    let _ = ENABLED.set(on);
    let map = if on { disk::load() } else { Map::new() };
    let _ = STORE.set(Mutex::new(map));
}

pub fn get(key: &str) -> Option<(String, String)> {
    if !enabled() {
        return None;
    }
    let guard = STORE.get()?.lock().ok()?;
    let e = guard.get(key)?;
    (now_secs().saturating_sub(e.ts) < TTL_SECS).then(|| (e.latest.clone(), e.license.clone()))
}

pub fn set(key: String, latest: String, license: String) {
    if !enabled() {
        return;
    }
    if let Some(Ok(mut guard)) = STORE.get().map(|m| m.lock()) {
        guard.insert(key, Entry { ts: now_secs(), latest, license });
    }
}

pub fn flush() {
    if !enabled() {
        return;
    }
    if let Some(Ok(guard)) = STORE.get().map(|m| m.lock()) {
        disk::save(&guard);
    }
}
