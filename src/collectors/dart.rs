use crate::lockfiles::{LockEntry, locked_versions, resolve_ver};
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::{fs, path::Path};

fn parse_pubspec(dir: &Path) -> Vec<(String, String)> {
    let Ok(text) = fs::read_to_string(dir.join("pubspec.yaml")) else { return Vec::new() };
    let locked = locked_versions(dir, "Pub");
    let (mut out, mut in_deps) = (Vec::new(), false);
    for line in text.lines() {
        if line.starts_with("dependencies:") || line.starts_with("dev_dependencies:") { in_deps = true; continue; }
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

pub fn collect(dir: &Path) -> (Vec<OutdatedPkg>, Vec<LockEntry>) {
    let deps = parse_pubspec(dir);
    let direct = deps.iter().map(|(n, v)| ("Pub", n.clone(), v.clone())).collect();
    let out = crate::pool::map_bounded(&deps, |(name, cur)| {
        let key = format!("pub:{name}");
        let latest = if let Some((t, _)) = crate::cache::get(&key) { t } else {
            let data: Value = crate::http::agent().get(&format!("https://pub.dev/api/packages/{name}")).call().ok()?.into_body().read_json().ok()?;
            let lat = data.get("latest")?.get("version")?.as_str()?.to_string();
            crate::cache::set(key, lat.clone(), "-".into());
            lat
        };
        is_newer(cur, &latest).then(|| OutdatedPkg {
            manager: "pub".into(), name: name.clone(), kind: classify_update(cur, &latest),
            current: cur.clone(), latest, license: "-".into(), cves: Vec::new(),
        })
    });
    (out, direct)
}
