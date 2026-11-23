use crate::lockfiles::LockEntry;
use serde_json::{Map, Value};
use std::{fs, path::Path};

fn walk_v1_deps(deps: &Map<String, Value>, out: &mut Vec<LockEntry>) {
    for (name, val) in deps {
        if let Some(ver) = val.get("version").and_then(Value::as_str)
            && !ver.starts_with("file:") && !ver.starts_with("http")
        {
            out.push(("npm", name.clone(), ver.to_string()));
        }
        if let Some(nested) = val.get("dependencies").and_then(Value::as_object) {
            walk_v1_deps(nested, out);
        }
    }
}

pub fn parse_package_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("package-lock.json")) else { return };
    let Ok(data) = serde_json::from_str::<Value>(&text) else { return };
    if let Some(pkgs) = data.get("packages").and_then(Value::as_object) {
        for (key, val) in pkgs {
            if key.is_empty() { continue; }
            let name = key.rsplit("node_modules/").next().unwrap_or("");
            let Some(ver) = val.get("version").and_then(Value::as_str) else { continue };
            if !name.is_empty() && !ver.is_empty() {
                out.push(("npm", name.to_string(), ver.to_string()));
            }
        }
    } else if let Some(deps) = data.get("dependencies").and_then(Value::as_object) {
        walk_v1_deps(deps, out);
    }
}

pub fn parse_pnpm_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("pnpm-lock.yaml")) else { return };
    let mut in_pkgs = false;
    for line in text.lines() {
        if line.starts_with("packages:") { in_pkgs = true; continue; }
        if in_pkgs && !line.starts_with(' ') && !line.trim().is_empty() { in_pkgs = false; continue; }
        if in_pkgs && line.starts_with("  ") && !line.starts_with("    ") {
            let key = line.trim().trim_end_matches(':').trim_matches(['\'', '"']).trim_start_matches('/');
            let base = key.split('(').next().unwrap_or(key);
            if let Some(idx) = base.rfind('@').filter(|&i| i > 0) {
                let (name, ver) = (&base[..idx], &base[idx + 1..]);
                if !name.is_empty() && !ver.is_empty() {
                    out.push(("npm", name.to_string(), ver.to_string()));
                }
            }
        }
    }
}
