use crate::lockfiles::{locked_versions, resolve_ver};
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path, thread};

fn parse_cargo_deps(text: &str, locked: &BTreeMap<String, Vec<String>>) -> Vec<(String, String)> {
    let (mut deps, mut in_deps) = (Vec::new(), false);
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_deps = matches!(
                t,
                "[dependencies]" | "[dev-dependencies]" | "[build-dependencies]" | "[workspace.dependencies]"
            );
            continue;
        }
        if !in_deps || t.is_empty() || t.starts_with('#') { continue; }
        let Some((k, v)) = t.split_once('=') else { continue };
        let (name, v) = (k.trim(), v.trim());
        let raw = if v.starts_with('"') {
            v.trim_matches('"')
        } else if let Some(i) = v.find("version") {
            v[i..].split('"').nth(1).unwrap_or("")
        } else { "" };
        let cleaned = clean_ver(raw);
        if !cleaned.is_empty() {
            deps.push((name.to_string(), resolve_ver(locked, name, &cleaned)));
        }
    }
    deps
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let Ok(text) = fs::read_to_string(dir.join("Cargo.toml")) else { return Vec::new() };
    let locked = locked_versions(dir, "crates.io");
    let deps = parse_cargo_deps(&text, &locked);
    let mut out = Vec::new();
    thread::scope(|s| {
        let handles: Vec<_> = deps.iter().map(|(name, current)| {
            s.spawn(move || {
                let key = format!("cargo:{name}");
                let (latest, license) = if let Some(hit) = crate::cache::get(&key) {
                    hit
                } else {
                    let url = format!("https://crates.io/api/v1/crates/{name}");
                    let resp = crate::http::agent().get(&url).set("User-Agent", "sentinel-deps-cli/1.0").call().ok()?;
                    let data: Value = resp.into_json().ok()?;
                    let lat = data.get("crate")?.get("max_stable_version")?.as_str()?.to_string();
                    let lic = data.get("versions").and_then(Value::as_array)
                        .and_then(|a| a.first()).and_then(|v| v.get("license"))
                        .and_then(Value::as_str).unwrap_or("-").to_string();
                    crate::cache::set(key, lat.clone(), lic.clone());
                    (lat, lic)
                };
                (!current.is_empty() && current != &latest).then(|| OutdatedPkg {
                    manager: "cargo".into(), name: name.clone(),
                    kind: classify_update(current, &latest),
                    current: current.clone(), latest, license, cves: Vec::new(),
                })
            })
        }).collect();
        for h in handles { if let Ok(Some(item)) = h.join() { out.push(item); } }
    });
    out
}
