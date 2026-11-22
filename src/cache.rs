use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const TTL_SECS: u64 = 900;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Entry {
    ts: u64,
    latest: String,
    license: String,
}

static STORE: OnceLock<Mutex<HashMap<String, Entry>>> = OnceLock::new();
static ENABLED: OnceLock<bool> = OnceLock::new();

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn cache_path() -> PathBuf {
    std::env::temp_dir().join("sentinel-registry-cache.json")
}

pub fn init(enabled: bool) {
    let _ = ENABLED.set(enabled);
    let map = if enabled {
        fs::read_to_string(cache_path())
            .ok()
            .and_then(|s| serde_json::from_str::<HashMap<String, Entry>>(&s).ok())
            .unwrap_or_default()
    } else {
        HashMap::new()
    };
    let _ = STORE.set(Mutex::new(map));
}

pub fn get(key: &str) -> Option<(String, String)> {
    if !ENABLED.get().copied().unwrap_or(false) { return None; }
    let guard = STORE.get()?.lock().ok()?;
    let e = guard.get(key)?;
    (now_secs().saturating_sub(e.ts) < TTL_SECS).then(|| (e.latest.clone(), e.license.clone()))
}

pub fn set(key: String, latest: String, license: String) {
    if !ENABLED.get().copied().unwrap_or(false) { return; }
    if let Some(m) = STORE.get() {
        if let Ok(mut guard) = m.lock() {
            guard.insert(key, Entry { ts: now_secs(), latest, license });
        }
    }
}

pub fn flush() {
    if !ENABLED.get().copied().unwrap_or(false) {
        return;
    }
    if let Some(m) = STORE.get() {
        if let Ok(guard) = m.lock() {
            if let Ok(json) = serde_json::to_string(&*guard) {
                let target = cache_path();
                let tmp = std::env::temp_dir()
                    .join(format!("sentinel-cache-{}.tmp", std::process::id()));
                if fs::write(&tmp, json).is_ok() {
                    let _ = fs::rename(&tmp, &target);
                }
            }
        }
    }
}
