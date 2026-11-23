mod parse;
#[cfg(test)]
mod tests;

use crate::lockfiles::{LockEntry, locked_versions};
use crate::types::OutdatedPkg;
use crate::version::{classify_update, is_newer};
use serde_json::Value;
use std::{fs, path::Path};

pub fn collect(dir: &Path) -> (Vec<OutdatedPkg>, Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("Cargo.toml")) else { return (Vec::new(), Vec::new()) };
    let locked = locked_versions(dir, "crates.io");
    let deps = parse::parse_cargo_deps(&text, &locked);
    let direct = parse::to_direct(&deps);
    let out = crate::pool::map_bounded(&deps, |(name, current)| {
        let key = format!("cargo:{name}");
        let (latest, license) = if let Some(hit) = crate::cache::get(&key) { hit } else {
            let url = format!("https://crates.io/api/v1/crates/{name}");
            let mut resp = crate::http::agent().get(&url).header("User-Agent", crate::http::user_agent()).call().ok()?;
            let data: Value = resp.body_mut().read_json().ok()?;
            let lat = data.get("crate")?.get("max_stable_version")?.as_str()?.to_string();
            let lic = data.get("versions").and_then(Value::as_array)
                .and_then(|a| a.first()).and_then(|v| v.get("license"))
                .and_then(Value::as_str).unwrap_or("-").to_string();
            crate::cache::set(key, lat.clone(), lic.clone());
            (lat, lic)
        };
        is_newer(current, &latest).then(|| OutdatedPkg {
            manager: "cargo".into(), name: name.clone(), kind: classify_update(current, &latest),
            current: current.clone(), latest, license, cves: Vec::new(),
        })
    });
    (out, direct)
}
