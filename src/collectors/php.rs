use crate::cmd;
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::path::Path;

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let raw = cmd::run("composer", &["outdated", "--direct", "--format", "json"], dir);
    let mut out = Vec::new();
    let Ok(parsed) = serde_json::from_str::<Value>(&raw) else {
        return out;
    };
    let Some(arr) = parsed.get("installed").and_then(Value::as_array) else {
        return out;
    };
    for item in arr {
        let name = item.get("name").and_then(Value::as_str).unwrap_or("");
        let current = clean_ver(item.get("version").and_then(Value::as_str).unwrap_or(""));
        let latest = clean_ver(item.get("latest").and_then(Value::as_str).unwrap_or(""));
        if !name.is_empty() && current != latest {
            out.push(OutdatedPkg {
                manager: "composer".into(),
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
