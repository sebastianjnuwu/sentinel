use crate::cmd;
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::path::Path;

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let (manager, raw) = if dir.join("uv.lock").exists() {
        ("uv", cmd::run("uv", &["pip", "list", "--outdated", "--format", "json"], dir))
    } else {
        ("pip", cmd::run("pip", &["list", "--outdated", "--format", "json"], dir))
    };
    let mut out = Vec::new();
    let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(&raw) else {
        return out;
    };
    for item in arr {
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        let current = clean_ver(item.get("version").and_then(Value::as_str).unwrap_or(""));
        let latest = clean_ver(item.get("latest_version").and_then(Value::as_str).unwrap_or(""));
        if !name.is_empty() && !latest.is_empty() && current != latest {
            out.push(OutdatedPkg {
                manager: manager.into(),
                name: name.into(),
                kind: classify_update(&current, &latest),
                current,
                latest,
                license: "-".into(),
                cves: Vec::new(),
            });
        }
    }
    out
}
