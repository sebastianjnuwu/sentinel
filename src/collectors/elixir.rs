use crate::lockfiles::{LockEntry, locked_versions, resolve_ver};
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::{fs, path::Path};

fn parse_mix(dir: &Path) -> Vec<(String, String)> {
    let Ok(text) = fs::read_to_string(dir.join("mix.exs")) else { return Vec::new() };
    let locked = locked_versions(dir, "Hex");
    let mut out = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("{:") {
            let Some((name, after)) = rest.split_once(',') else { continue };
            let Some(ver) = after.split('"').nth(1) else { continue };
            let cur = clean_ver(ver);
            let n = name.trim();
            if !cur.is_empty() { out.push((n.to_string(), resolve_ver(&locked, n, &cur))); }
        }
    }
    out
}

pub fn collect(dir: &Path) -> (Vec<OutdatedPkg>, Vec<LockEntry>) {
    let deps = parse_mix(dir);
    let direct = deps.iter().map(|(n, v)| ("Hex", n.clone(), v.clone())).collect();
    let out = crate::pool::map_bounded(&deps, |(name, cur)| {
        let key = format!("hex:{name}");
        let (latest, license) = if let Some(hit) = crate::cache::get(&key) { hit } else {
            let data: Value = crate::http::agent().get(&format!("https://hex.pm/api/packages/{name}"))
                .header("User-Agent", crate::http::user_agent()).call().ok()?.into_body().read_json().ok()?;
            let lat = data.get("latest_stable_version").or_else(|| data.get("latest_version"))?.as_str()?.to_string();
            let lic = data.get("meta")?.get("licenses").and_then(Value::as_array).and_then(|a| a.first()).and_then(Value::as_str).unwrap_or("-").to_string();
            crate::cache::set(key, lat.clone(), lic.clone());
            (lat, lic)
        };
        is_newer(cur, &latest).then(|| OutdatedPkg {
            manager: "hex".into(), name: name.clone(), kind: classify_update(cur, &latest),
            current: cur.clone(), latest, license, cves: Vec::new(),
        })
    });
    (out, direct)
}
