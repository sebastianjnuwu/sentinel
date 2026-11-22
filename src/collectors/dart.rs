use crate::lockfiles::{locked_versions, resolve_ver};
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::{fs, path::Path, thread};

fn parse_pubspec(dir: &Path) -> Vec<(String, String)> {
    let Ok(text) = fs::read_to_string(dir.join("pubspec.yaml")) else { return Vec::new() };
    let locked = locked_versions(dir, "Pub");
    let (mut out, mut in_deps) = (Vec::new(), false);
    for line in text.lines() {
        if line.starts_with("dependencies:") || line.starts_with("dev_dependencies:") {
            in_deps = true;
            continue;
        }
        if !line.starts_with(' ') && !line.trim().is_empty() { in_deps = false; continue; }
        if in_deps && line.starts_with("  ") && !line.starts_with("    ") {
            let Some((k, v)) = line.trim().split_once(':') else { continue };
            let cur = clean_ver(v.trim().trim_matches(['"', '\'']));
            if !cur.is_empty() && k != "flutter" && k != "sdk" {
                out.push((k.to_string(), resolve_ver(&locked, k, &cur)));
            }
        }
    }
    out
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let deps = parse_pubspec(dir);
    let mut out = Vec::new();
    thread::scope(|s| {
        let handles: Vec<_> = deps.iter().map(|(name, cur)| {
            s.spawn(move || {
                let key = format!("pub:{name}");
                let latest = if let Some((t, _)) = crate::cache::get(&key) { t } else {
                    let data: Value = crate::http::agent().get(&format!("https://pub.dev/api/packages/{name}")).call().ok()?.into_json().ok()?;
                    let lat = data.get("latest")?.get("version")?.as_str()?.to_string();
                    crate::cache::set(key, lat.clone(), "-".into());
                    lat
                };
                (cur != &latest).then(|| OutdatedPkg {
                    manager: "pub".into(), name: name.clone(), kind: classify_update(cur, &latest),
                    current: cur.clone(), latest, license: "-".into(), cves: Vec::new(),
                })
            })
        }).collect();
        for h in handles { if let Ok(Some(item)) = h.join() { out.push(item); } }
    });
    out
}
