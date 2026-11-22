use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::thread;

fn parse_mix(dir: &Path) -> Vec<(String, String)> {
    let Ok(text) = fs::read_to_string(dir.join("mix.exs")) else { return Vec::new() };
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("{:") {
            let Some((name, after)) = rest.split_once(',') else { continue };
            let Some(ver) = after.split('"').nth(1) else { continue };
            let cur = clean_ver(ver);
            if !cur.is_empty() { out.push((name.trim().to_string(), cur)); }
        }
    }
    out
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let deps = parse_mix(dir);
    let mut out = Vec::new();
    thread::scope(|s| {
        let handles: Vec<_> = deps.iter().map(|(name, cur)| {
            s.spawn(move || {
                let key = format!("hex:{name}");
                let (latest, license) = if let Some(hit) = crate::cache::get(&key) { hit } else {
                    let data: Value = crate::http::agent().get(&format!("https://hex.pm/api/packages/{name}")).set("User-Agent", "sentinel-deps-cli/0.1").call().ok()?.into_json().ok()?;
                    let lat = data.get("latest_stable_version").or_else(|| data.get("latest_version"))?.as_str()?.to_string();
                    let lic = data.get("meta")?.get("licenses").and_then(Value::as_array).and_then(|a| a.first()).and_then(Value::as_str).unwrap_or("-").to_string();
                    crate::cache::set(key, lat.clone(), lic.clone());
                    (lat, lic)
                };
                (cur != &latest).then(|| OutdatedPkg {
                    manager: "hex".into(), name: name.clone(), kind: classify_update(cur, &latest),
                    current: cur.clone(), latest, license, cves: Vec::new(),
                })
            })
        }).collect();
        for h in handles { if let Ok(Some(item)) = h.join() { out.push(item); } }
    });
    out
}
