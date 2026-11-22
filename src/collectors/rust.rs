use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::thread;

fn parse_cargo_deps(text: &str) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    let mut in_deps = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_deps = t == "[dependencies]" || t == "[dev-dependencies]";
            continue;
        }
        if !in_deps || t.is_empty() || t.starts_with('#') { continue; }
        let Some((k, v)) = t.split_once('=') else { continue };
        let v = v.trim();
        let ver = if v.starts_with('"') {
            v.trim_matches('"')
        } else if let Some(i) = v.find("version") {
            v[i..].split('"').nth(1).unwrap_or("")
        } else { "" };
        if !ver.is_empty() { deps.push((k.trim().to_string(), ver.to_string())); }
    }
    deps
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let Ok(text) = fs::read_to_string(dir.join("Cargo.toml")) else {
        return Vec::new();
    };
    let deps = parse_cargo_deps(&text);
    let mut out = Vec::new();
    thread::scope(|s| {
        let handles: Vec<_> = deps.iter().map(|(name, declared)| {
            s.spawn(move || {
                let key = format!("cargo:{name}");
                let (latest, license) = if let Some(hit) = crate::cache::get(&key) {
                    hit
                } else {
                    let url = format!("https://crates.io/api/v1/crates/{name}");
                    let resp = crate::http::agent().get(&url).set("User-Agent", "sentinel-deps-cli/0.1").call().ok()?;
                    let data: Value = resp.into_json().ok()?;
                    let lat = data.get("crate")?.get("max_stable_version")?.as_str()?.to_string();
                    let lic = data.get("versions").and_then(Value::as_array)
                        .and_then(|a| a.first()).and_then(|v| v.get("license"))
                        .and_then(Value::as_str).unwrap_or("-").to_string();
                    crate::cache::set(key, lat.clone(), lic.clone());
                    (lat, lic)
                };
                let current = clean_ver(declared);
                let behind = if current.contains('.') { current != latest } else { current.split('.').next() != latest.split('.').next() };
                if !current.is_empty() && behind {
                    Some(OutdatedPkg {
                        manager: "cargo".into(), name: name.clone(),
                        kind: classify_update(&current, &latest),
                        current, latest, license, cves: Vec::new(),
                    })
                } else { None }
            })
        }).collect();
        for h in handles { if let Ok(Some(item)) = h.join() { out.push(item); } }
    });
    out
}
