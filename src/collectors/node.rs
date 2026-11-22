use crate::lockfiles::{locked_versions, resolve_ver};
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::{fs, path::Path};

fn detect_manager(dir: &Path) -> &'static str {
    if dir.join("bun.lock").exists() || dir.join("bun.lockb").exists() { "bun" }
    else if dir.join("pnpm-lock.yaml").exists() { "pnpm" }
    else if dir.join("yarn.lock").exists() { "yarn" }
    else { "npm" }
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let manager = detect_manager(dir);
    let Ok(text) = fs::read_to_string(dir.join("package.json")) else { return Vec::new() };
    let Ok(pkg) = serde_json::from_str::<Value>(&text) else { return Vec::new() };
    let locked = locked_versions(dir, "npm");
    let mut deps: Vec<(String, String)> = Vec::new();
    for section in ["dependencies", "devDependencies"] {
        if let Some(obj) = pkg.get(section).and_then(Value::as_object) {
            for (name, ver) in obj {
                if let Some(v) = ver.as_str()
                    && !v.starts_with("workspace:")
                    && !v.starts_with("file:")
                    && !v.starts_with("http")
                {
                    let cleaned = clean_ver(v);
                    if !cleaned.is_empty() {
                        deps.push((name.clone(), resolve_ver(&locked, name, &cleaned)));
                    }
                }
            }
        }
    }

    crate::pool::map_bounded(&deps, |(name, current)| {
        let key = format!("npm:{name}");
        let (latest, license) = if let Some(hit) = crate::cache::get(&key) {
            hit
        } else {
            let url = format!("https://registry.npmjs.org/{name}/latest");
            let resp = crate::http::agent().get(&url).call().ok()?;
            let data: Value = resp.into_json().ok()?;
            let lat = data.get("version").and_then(Value::as_str)?.to_string();
            let lic = data
                .get("license")
                .and_then(|l| l.as_str().or_else(|| l.get("type").and_then(Value::as_str)))
                .unwrap_or("-")
                .to_string();
            crate::cache::set(key, lat.clone(), lic.clone());
            (lat, lic)
        };
        (!current.is_empty() && current != &latest).then(|| OutdatedPkg {
            manager: manager.into(),
            name: name.clone(),
            kind: classify_update(current, &latest),
            current: current.clone(),
            latest,
            license,
            cves: Vec::new(),
        })
    })
}
