use crate::types::OutdatedPkg;
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn osv_ecosystem(manager: &str) -> Option<&'static str> {
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
        _ => None,
    }
}

pub fn attach_cves(report: &mut BTreeMap<String, Vec<OutdatedPkg>>) {
    let (mut refs, mut queries, mut keys) = (Vec::new(), Vec::new(), Vec::new());
    for (proj, items) in report.iter_mut() {
        for (idx, pkg) in items.iter_mut().enumerate() {
            let Some(eco) = osv_ecosystem(&pkg.manager) else { continue };
            let key = format!("osv:{eco}:{}@{}", pkg.name, pkg.current);
            if let Some((cached, _)) = crate::cache::get(&key) {
                if !cached.is_empty() {
                    pkg.cves = cached.split(',').map(String::from).collect();
                }
            } else {
                queries.push(json!({ "package": { "name": pkg.name, "ecosystem": eco }, "version": pkg.current }));
                refs.push((proj.clone(), idx));
                keys.push(key);
            }
        }
    }
    if queries.is_empty() { return; }

    let Ok(resp) = crate::http::agent().post("https://api.osv.dev/v1/querybatch").send_json(json!({ "queries": queries })) else { return };
    let Ok(data) = resp.into_json::<Value>() else { return };
    let Some(results) = data.get("results").and_then(Value::as_array) else { return };

    for (((proj, idx), key), res) in refs.into_iter().zip(keys.into_iter()).zip(results.iter()) {
        let ids: Vec<String> = res.get("vulns").and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.get("id").and_then(Value::as_str).map(String::from)).take(2).collect())
            .unwrap_or_default();
        crate::cache::set(key, ids.join(","), String::new());
        if !ids.is_empty() {
            if let Some(pkg) = report.get_mut(&proj).and_then(|v| v.get_mut(idx)) { pkg.cves = ids; }
        }
    }
}
