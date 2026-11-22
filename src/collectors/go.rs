use crate::cmd;
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::path::Path;

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let raw = cmd::run("go", &["list", "-m", "-u", "-json", "all"], dir);
    if raw.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for item in serde_json::Deserializer::from_str(&raw)
        .into_iter::<Value>()
        .flatten()
    {
        if item.get("Indirect").and_then(Value::as_bool).unwrap_or(false) {
            continue;
        }
        let Some(latest) = item
            .get("Update")
            .and_then(|u| u.get("Version"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let name = item.get("Path").and_then(Value::as_str).unwrap_or("");
        let current = clean_ver(item.get("Version").and_then(Value::as_str).unwrap_or(""));
        let latest_clean = clean_ver(latest);
        out.push(OutdatedPkg {
            manager: "go".into(),
            name: name.into(),
            kind: classify_update(&current, &latest_clean),
            current,
            latest: latest_clean,
            license: "-".into(),
            cves: Vec::new(),
        });
    }
    out
}
