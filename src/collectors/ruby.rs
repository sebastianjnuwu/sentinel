use crate::lockfiles::{locked_versions, resolve_ver};
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::{fs, path::Path, thread};

fn parse_gemfile(dir: &Path) -> Vec<(String, String)> {
    let Ok(text) = fs::read_to_string(dir.join("Gemfile")) else { return Vec::new() };
    let locked = locked_versions(dir, "RubyGems");
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if !t.starts_with("gem ") { continue; }
        let parts: Vec<&str> = t.split(['\'', '"']).collect();
        if parts.len() >= 4 {
            let cur = clean_ver(parts[3]);
            if !cur.is_empty() {
                out.push((parts[1].to_string(), resolve_ver(&locked, parts[1], &cur)));
            }
        } else if parts.len() >= 2 {
            let resolved = resolve_ver(&locked, parts[1], "");
            if !resolved.is_empty() {
                out.push((parts[1].to_string(), resolved));
            }
        }
    }
    out
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let gems = parse_gemfile(dir);
    let mut out = Vec::new();
    thread::scope(|s| {
        let handles: Vec<_> = gems.iter().map(|(name, cur)| {
            s.spawn(move || {
                let key = format!("ruby:{name}");
                let (latest, license) = if let Some(hit) = crate::cache::get(&key) { hit } else {
                    let data: Value = crate::http::agent().get(&format!("https://rubygems.org/api/v1/gems/{name}.json")).call().ok()?.into_json().ok()?;
                    let lat = data.get("version")?.as_str()?.to_string();
                    let lic = data.get("licenses").and_then(Value::as_array).and_then(|a| a.first()).and_then(Value::as_str).unwrap_or("-").to_string();
                    crate::cache::set(key, lat.clone(), lic.clone());
                    (lat, lic)
                };
                (cur != &latest).then(|| OutdatedPkg {
                    manager: "bundler".into(), name: name.clone(), kind: classify_update(cur, &latest),
                    current: cur.clone(), latest, license, cves: Vec::new(),
                })
            })
        }).collect();
        for h in handles { if let Ok(Some(item)) = h.join() { out.push(item); } }
    });
    out
}
