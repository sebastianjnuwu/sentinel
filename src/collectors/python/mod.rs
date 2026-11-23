mod parse;

use crate::lockfiles::{LockEntry, locked_versions};
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::path::Path;

pub fn collect(dir: &Path) -> (Vec<OutdatedPkg>, Vec<LockEntry>) {
    let mgr = if dir.join("uv.lock").exists() { "uv" } else { "pip" };
    let locked = locked_versions(dir, "PyPI");
    let deps = parse::parse_python_deps(dir, &locked);
    let direct = deps.iter().map(|(n, v)| ("PyPI", n.clone(), v.clone())).collect();
    let out = crate::pool::map_bounded(&deps, |(name, current)| {
        let key = format!("pypi:{name}");
        let (latest, license) = if let Some(hit) = crate::cache::get(&key) { hit } else {
            let url = format!("https://pypi.org/pypi/{name}/json");
            let data: Value = crate::http::agent().get(&url).call().ok()?.into_body().read_json().ok()?;
            let info = data.get("info")?;
            let lat = clean_ver(info.get("version")?.as_str()?);
            let lic = info.get("license").and_then(Value::as_str).filter(|s| !s.is_empty()).unwrap_or("-").to_string();
            crate::cache::set(key, lat.clone(), lic.clone());
            (lat, lic)
        };
        is_newer(current, &latest).then(|| OutdatedPkg {
            manager: mgr.into(), name: name.clone(), kind: classify_update(current, &latest),
            current: current.clone(), latest, license, cves: Vec::new(),
        })
    });
    (out, direct)
}
