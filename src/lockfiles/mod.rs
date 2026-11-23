pub mod extra;
pub mod go;
pub mod misc;
pub mod npm;
#[cfg(test)] mod tests;
pub mod toml_locks;

use crate::types::TransitiveVuln;
use std::{collections::BTreeMap, collections::HashSet, path::Path};

pub type LockEntry = (&'static str, String, String);

pub fn collect_lock_entries(dir: &Path) -> Vec<LockEntry> {
    let mut raw = Vec::new();
    npm::parse_package_lock(dir, &mut raw); npm::parse_pnpm_lock(dir, &mut raw);
    extra::parse_yarn_lock(dir, &mut raw); toml_locks::parse_cargo_lock(dir, &mut raw);
    toml_locks::parse_uv_lock(dir, &mut raw); extra::parse_poetry_lock(dir, &mut raw);
    extra::parse_nuget_lock(dir, &mut raw); go::parse_go_lock(dir, &mut raw);
    misc::parse_composer_lock(dir, &mut raw); misc::parse_gemfile_lock(dir, &mut raw);
    misc::parse_pubspec_lock(dir, &mut raw); misc::parse_mix_lock(dir, &mut raw);
    raw
}

pub fn locked_versions(dir: &Path, eco: &str) -> BTreeMap<String, Vec<String>> {
    let mut map = BTreeMap::<String, Vec<String>>::new();
    for (e, name, ver) in collect_lock_entries(dir) {
        if e == eco { map.entry(name).or_default().push(ver); }
    }
    map
}

pub fn resolve_ver(locked: &BTreeMap<String, Vec<String>>, name: &str, declared: &str) -> String {
    let Some(vers) = locked.get(name) else { return declared.to_string() };
    if declared.is_empty() {
        return if vers.len() == 1 { vers[0].clone() } else { String::new() };
    }
    vers.iter().find(|lv| lv.starts_with(declared)).cloned().unwrap_or_else(|| declared.to_string())
}

pub fn scan_lockfile_vulns(dir: &Path, extra_deps: &[LockEntry]) -> Result<Vec<TransitiveVuln>, String> {
    let (mut seen, mut vulns, mut uncached) = (HashSet::new(), Vec::new(), Vec::new());
    for (eco, name, ver) in collect_lock_entries(dir).into_iter().chain(extra_deps.iter().cloned()) {
        let key = format!("osv:{eco}:{name}@{ver}");
        if !seen.insert(key.clone()) { continue; }
        if eco == "GitHub Actions" {
            let ids = crate::osv::gha::query_action(&name, &ver)?;
            if !ids.is_empty() { vulns.push(TransitiveVuln { ecosystem: eco.into(), package: name, version: ver, ids }); }
            continue;
        }
        if let Some((cached, _)) = crate::cache::get(&key) {
            if !cached.is_empty() {
                vulns.push(TransitiveVuln { ecosystem: eco.into(), package: name, version: ver, ids: cached.split(',').map(String::from).collect() });
            }
        } else { uncached.push((eco.to_string(), name, ver, key)); }
    }
    let queries: Vec<_> = uncached.iter().map(|(e, n, v, _)| (e.clone(), n.clone(), v.clone())).collect();
    if !queries.is_empty() {
        for ((eco, name, ver, key), ids) in uncached.into_iter().zip(crate::osv::query_batch(&queries)?) {
            crate::cache::set(key, ids.join(","), String::new());
            if !ids.is_empty() { vulns.push(TransitiveVuln { ecosystem: eco, package: name, version: ver, ids }); }
        }
    }
    Ok(vulns)
}
