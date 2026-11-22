use super::Map;
use std::fs;
use std::path::PathBuf;

fn path() -> PathBuf {
    std::env::temp_dir().join("sentinel-registry-cache.json")
}

pub fn load() -> Map {
    fs::read_to_string(path())
        .ok()
        .and_then(|s| serde_json::from_str::<Map>(&s).ok())
        .unwrap_or_default()
}

pub fn save(map: &Map) {
    let Ok(json) = serde_json::to_string(map) else { return };
    let tmp = std::env::temp_dir().join(format!("sentinel-cache-{}.tmp", std::process::id()));
    if fs::write(&tmp, json).is_ok() {
        let _ = fs::rename(&tmp, path());
    }
}
