pub mod extra;
pub mod misc;
pub mod npm;
pub mod toml_locks;

use crate::types::TransitiveVuln;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::path::Path;

pub type LockEntry = (&'static str, String, String);

pub fn scan_lockfile_vulns(dir: &Path) -> Vec<TransitiveVuln> {
    let mut raw = Vec::new();
    npm::parse_package_lock(dir, &mut raw);
    npm::parse_pnpm_lock(dir, &mut raw);
    extra::parse_yarn_lock(dir, &mut raw);
    toml_locks::parse_cargo_lock(dir, &mut raw);
    toml_locks::parse_uv_lock(dir, &mut raw);
    extra::parse_poetry_lock(dir, &mut raw);
    extra::parse_nuget_lock(dir, &mut raw);
    misc::parse_go_sum(dir, &mut raw);
    misc::parse_composer_lock(dir, &mut raw);
    misc::parse_gemfile_lock(dir, &mut raw);
    misc::parse_pubspec_lock(dir, &mut raw);
    misc::parse_mix_lock(dir, &mut raw);

    let (mut seen, mut vulns, mut uncached) = (HashSet::new(), Vec::new(), Vec::new());
    for (eco, name, ver) in raw {
        let key = format!("osv:{eco}:{name}@{ver}");
        if !seen.insert(key.clone()) { continue; }
        if let Some((cached, _)) = crate::cache::get(&key) {
            if !cached.is_empty() {
                vulns.push(TransitiveVuln { ecosystem: eco.into(), package: name, version: ver, ids: cached.split(',').map(String::from).collect() });
            }
        } else {
            uncached.push((eco, name, ver, key));
        }
    }

    for chunk in uncached.chunks(500) {
        let queries: Vec<Value> = chunk.iter().map(|(e, n, v, _)| json!({ "package": { "name": n, "ecosystem": e }, "version": v })).collect();
        let Ok(resp) = crate::http::agent().post("https://api.osv.dev/v1/querybatch").send_json(json!({ "queries": queries })) else { continue };
        let Ok(data) = resp.into_json::<Value>() else { continue };
        let Some(results) = data.get("results").and_then(Value::as_array) else { continue };

        for ((eco, name, ver, key), res) in chunk.iter().zip(results.iter()) {
            let ids: Vec<String> = res.get("vulns").and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.get("id").and_then(Value::as_str).map(String::from)).take(2).collect())
                .unwrap_or_default();
            crate::cache::set(key.clone(), ids.join(","), String::new());
            if !ids.is_empty() {
                vulns.push(TransitiveVuln { ecosystem: (*eco).into(), package: name.clone(), version: ver.clone(), ids });
            }
        }
    }
    vulns
}
