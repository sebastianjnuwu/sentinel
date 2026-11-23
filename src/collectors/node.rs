use crate::lockfiles::{locked_versions, resolve_ver};
use crate::types::{OutdatedPkg, ProjectScan};
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::{fs, path::Path};

fn detect_manager(dir: &Path) -> &'static str {
    if dir.join("bun.lock").exists() || dir.join("bun.lockb").exists() { "bun" }
    else if dir.join("pnpm-lock.yaml").exists() { "pnpm" }
    else if dir.join("yarn.lock").exists() { "yarn" }
    else { "npm" }
}

pub fn collect(dir: &Path) -> ProjectScan {
    let manager = detect_manager(dir);
    let Ok(text) = fs::read_to_string(dir.join("package.json")) else { return ProjectScan::default() };
    let Ok(pkg) = serde_json::from_str::<Value>(&text) else { return ProjectScan::default() };
    let locked = locked_versions(dir, "npm");
    let mut deps = Vec::new();
    for section in ["dependencies", "devDependencies"] {
        if let Some(obj) = pkg.get(section).and_then(Value::as_object) {
            for (name, ver) in obj {
                if let Some(v) = ver.as_str()
                    && !v.starts_with("workspace:") && !v.starts_with("file:") && !v.starts_with("http")
                {
                    let c = clean_ver(v);
                    if !c.is_empty() { deps.push((name.clone(), resolve_ver(&locked, name, &c))); }
                }
            }
        }
    }
    let direct = deps.iter().map(|(n, v)| ("npm", n.clone(), v.clone())).collect();
    let pairs = crate::pool::map_bounded(&deps, |(name, current)| {
        let key = format!("npm:{name}");
        let (latest, license) = if let Some(hit) = crate::cache::get(&key) { hit } else {
            let url = format!("https://registry.npmjs.org/{name}/latest");
            let data: Value = crate::http::agent().get(&url).call().ok()?.into_body().read_json().ok()?;
            let lat = data.get("version").and_then(Value::as_str)?.to_string();
            let lic = data.get("license").and_then(|l| l.as_str().or_else(|| l.get("type").and_then(Value::as_str))).unwrap_or("-").to_string();
            crate::cache::set(key, lat.clone(), lic.clone());
            (lat, lic)
        };
        let item = is_newer(current, &latest).then(|| OutdatedPkg {
            manager: manager.into(), name: name.clone(), kind: classify_update(current, &latest),
            current: current.clone(), latest, license: license.clone(), cves: Vec::new(),
        });
        Some((item, (name.clone(), license)))
    });
    let (mut outdated, mut licenses) = (Vec::new(), Vec::new());
    for (o, l) in pairs { if let Some(p) = o { outdated.push(p); } licenses.push(l); }
    ProjectScan { outdated, direct, licenses }
}
