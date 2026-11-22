pub mod extra;
pub mod misc;
pub mod npm;
pub mod toml_locks;

use crate::types::TransitiveVuln;
use serde_json::{json, Value};
use std::{collections::BTreeMap, collections::HashSet, path::Path};

pub type LockEntry = (&'static str, String, String);

pub fn collect_lock_entries(dir: &Path) -> Vec<LockEntry> {
    let (mut raw, has_js, has_rs) = (Vec::new(), dir.join("package.json").exists(), dir.join("Cargo.toml").exists());
    for a in dir.ancestors().take(4) {
        if has_js { npm::parse_package_lock(a, &mut raw); npm::parse_pnpm_lock(a, &mut raw); extra::parse_yarn_lock(a, &mut raw); }
        if has_rs { toml_locks::parse_cargo_lock(a, &mut raw); }
    }
    toml_locks::parse_uv_lock(dir, &mut raw);
    extra::parse_poetry_lock(dir, &mut raw);
    extra::parse_nuget_lock(dir, &mut raw);
    misc::parse_go_lock(dir, &mut raw);
    misc::parse_composer_lock(dir, &mut raw);
    misc::parse_gemfile_lock(dir, &mut raw);
    misc::parse_pubspec_lock(dir, &mut raw);
    misc::parse_mix_lock(dir, &mut raw);
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
    locked.get(name)
        .and_then(|vers| vers.iter().find(|lv| lv.starts_with(declared)).or_else(|| vers.first()))
        .cloned()
        .unwrap_or_else(|| declared.to_string())
}

pub fn scan_lockfile_vulns(dir: &Path) -> Vec<TransitiveVuln> {
    let (mut seen, mut vulns, mut uncached) = (HashSet::new(), Vec::new(), Vec::new());
    for (eco, name, ver) in collect_lock_entries(dir) {
        let key = format!("osv:{eco}:{name}@{ver}");
        if !seen.insert(key.clone()) { continue; }
        if let Some((cached, _)) = crate::cache::get(&key) {
            if !cached.is_empty() {
                vulns.push(TransitiveVuln { ecosystem: eco.into(), package: name, version: ver, ids: cached.split(',').map(String::from).collect() });
            }
        } else { uncached.push((eco, name, ver, key)); }
    }
    for chunk in uncached.chunks(500) {
        let queries: Vec<Value> = chunk.iter().map(|(e, n, v, _)| json!({ "package": { "name": n, "ecosystem": e }, "version": v })).collect();
        let Ok(resp) = crate::http::agent().post("https://api.osv.dev/v1/querybatch").send_json(json!({ "queries": queries })) else { continue };
        let Ok(data) = resp.into_json::<Value>() else { continue };
        let Some(results) = data.get("results").and_then(Value::as_array) else { continue };
        for ((eco, name, ver, key), res) in chunk.iter().zip(results.iter()) {
            let ids: Vec<String> = res.get("vulns").and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.get("id").and_then(Value::as_str).map(String::from)).take(2).collect()).unwrap_or_default();
            crate::cache::set(key.clone(), ids.join(","), String::new());
            if !ids.is_empty() { vulns.push(TransitiveVuln { ecosystem: (*eco).into(), package: name.clone(), version: ver.clone(), ids }); }
        }
    }
    vulns
}
