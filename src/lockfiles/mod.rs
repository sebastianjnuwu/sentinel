pub mod extra;
pub mod go;
pub mod misc;
pub mod npm;
#[cfg(test)] mod tests;
pub mod toml_locks;

use crate::types::TransitiveVuln;
use std::{collections::{BTreeMap, HashMap, HashSet}, path::Path};

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
    if declared.is_empty() { return if vers.len() == 1 { vers[0].clone() } else { String::new() }; }
    vers.iter().find(|lv| lv.starts_with(declared)).cloned().unwrap_or_else(|| declared.to_string())
}

pub fn scan_batch(per_proj: Vec<(String, Vec<LockEntry>)>) -> Result<BTreeMap<String, Vec<TransitiveVuln>>, String> {
    let (mut resolved, mut uncached) = (HashMap::new(), Vec::new());
    for (_, entries) in &per_proj {
        for (eco, name, ver) in entries {
            let k = ((*eco).to_string(), name.clone(), ver.clone());
            if resolved.contains_key(&k) { continue; }
            if *eco == "GitHub Actions" {
                resolved.insert(k, crate::osv::gha::query_action(name, ver)?);
            } else if let Some((c, _)) = crate::cache::get(&format!("osv:{eco}:{name}@{ver}")) {
                resolved.insert(k, if c.is_empty() { Vec::new() } else { c.split(',').map(String::from).collect() });
            } else { resolved.insert(k.clone(), Vec::new()); uncached.push(k); }
        }
    }
    if !uncached.is_empty() {
        for (k, ids) in uncached.iter().cloned().zip(crate::osv::query_batch(&uncached)?) {
            crate::cache::set(format!("osv:{}:{}@{}", k.0, k.1, k.2), ids.join(","), String::new());
            resolved.insert(k, ids);
        }
    }
    let mut out = BTreeMap::<String, Vec<TransitiveVuln>>::new();
    for (proj, entries) in per_proj {
        let mut seen = HashSet::new();
        let list: Vec<_> = entries.into_iter().filter_map(|(e, n, v)| {
            let k = (e.to_string(), n, v);
            let ids = resolved.get(&k).filter(|i| !i.is_empty())?;
            seen.insert(k.clone()).then(|| TransitiveVuln { ecosystem: k.0, package: k.1, version: k.2, ids: ids.clone() })
        }).collect();
        if !list.is_empty() { out.insert(proj, list); }
    }
    Ok(out)
}
