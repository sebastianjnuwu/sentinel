use super::{Map, TTL_SECS, now_secs};
use std::{env, fs, path::PathBuf};

fn cache_dir() -> PathBuf {
    env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(env::temp_dir)
        .join("sentinel")
}

fn path() -> PathBuf {
    cache_dir().join("registry-cache.json")
}

pub fn load() -> Map {
    fs::read_to_string(path())
        .ok()
        .and_then(|s| serde_json::from_str::<Map>(&s).ok())
        .unwrap_or_default()
}

pub fn save(map: &Map) {
    let dir = cache_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let now = now_secs();
    let active: Map = map
        .iter()
        .filter(|(_, e)| now.saturating_sub(e.ts) < TTL_SECS)
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let Ok(json) = serde_json::to_string(&active) else { return };
    let tmp = dir.join(format!("registry-cache-{}.tmp", std::process::id()));
    if fs::write(&tmp, json).is_ok() {
        let _ = fs::rename(&tmp, path());
    }
}
