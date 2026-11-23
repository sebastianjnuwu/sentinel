mod client;
pub mod gha;

pub(crate) use client::post_osv;
pub use client::query_batch;
use crate::types::OutdatedPkg;
use std::collections::BTreeMap;

pub fn osv_ecosystem(manager: &str) -> Option<&'static str> {
    match manager {
        "npm" | "pnpm" | "yarn" | "bun" => Some("npm"),
        "go" => Some("Go"),
        "cargo" => Some("crates.io"),
        "pip" | "uv" => Some("PyPI"),
        "composer" => Some("Packagist"),
        "dotnet" => Some("NuGet"),
        "maven" | "gradle" => Some("Maven"),
        "bundler" => Some("RubyGems"),
        "pub" => Some("Pub"),
        "hex" => Some("Hex"),
        "actions" => Some("GitHub Actions"),
        _ => None,
    }
}

pub fn attach_cves(report: &mut BTreeMap<String, Vec<OutdatedPkg>>) -> Result<(), String> {
    let (mut refs, mut queries, mut keys) = (Vec::new(), Vec::new(), Vec::new());
    for (proj, items) in report.iter_mut() {
        for (idx, pkg) in items.iter_mut().enumerate() {
            if pkg.manager == "actions" {
                pkg.cves = gha::query_action(&pkg.name, &pkg.current)?;
                continue;
            }
            let Some(eco) = osv_ecosystem(&pkg.manager) else { continue };
            let key = format!("osv:{eco}:{}@{}", pkg.name, pkg.current);
            if let Some((cached, _)) = crate::cache::get(&key) {
                if !cached.is_empty() { pkg.cves = cached.split(',').map(String::from).collect(); }
            } else {
                queries.push((eco.into(), pkg.name.clone(), pkg.current.clone()));
                refs.push((proj.clone(), idx));
                keys.push(key);
            }
        }
    }
    if queries.is_empty() { return Ok(()); }
    for (((proj, idx), key), ids) in refs.into_iter().zip(keys).zip(query_batch(&queries)?) {
        crate::cache::set(key, ids.join(","), String::new());
        if !ids.is_empty() && let Some(pkg) = report.get_mut(&proj).and_then(|v| v.get_mut(idx)) {
            pkg.cves = ids;
        }
    }
    Ok(())
}
