use crate::lockfiles::{locked_versions, resolve_ver};
use crate::types::{OutdatedPkg, ProjectScan};
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::{fs, path::Path};

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
            if !cur.is_empty() { out.push((parts[1].to_string(), resolve_ver(&locked, parts[1], &cur))); }
        } else if parts.len() >= 2 {
            let resolved = resolve_ver(&locked, parts[1], "");
            if !resolved.is_empty() { out.push((parts[1].to_string(), resolved)); }
        }
    }
    out
}

pub fn collect(dir: &Path) -> ProjectScan {
    let gems = parse_gemfile(dir);
    let direct = gems.iter().map(|(n, v)| ("RubyGems", n.clone(), v.clone())).collect();
    let pairs = crate::pool::map_bounded(&gems, |(name, cur)| {
        let key = format!("ruby:{name}");
        let (latest, license) = if let Some(hit) = crate::cache::get(&key) { hit } else {
            let data: Value = crate::http::agent().get(&format!("https://rubygems.org/api/v1/gems/{name}.json")).call().ok()?.into_body().read_json().ok()?;
            let lat = data.get("version")?.as_str()?.to_string();
            let lic = data.get("licenses").and_then(Value::as_array).and_then(|a| a.first()).and_then(Value::as_str).unwrap_or("-").to_string();
            crate::cache::set(key, lat.clone(), lic.clone());
            (lat, lic)
        };
        let item = is_newer(cur, &latest).then(|| OutdatedPkg {
            manager: "bundler".into(), name: name.clone(), kind: classify_update(cur, &latest),
            current: cur.clone(), latest, license: license.clone(), cves: Vec::new(),
        });
        Some((item, (name.clone(), license)))
    });
    let (mut outdated, mut licenses) = (Vec::new(), Vec::new());
    for (o, l) in pairs { if let Some(p) = o { outdated.push(p); } licenses.push(l); }
    ProjectScan { outdated, direct, licenses }
}
