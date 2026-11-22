use crate::lockfiles::LockEntry;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn parse_yarn_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("yarn.lock")) else { return };
    let mut name = String::new();
    for line in text.lines() {
        if !line.starts_with(' ') && line.ends_with(':') && !line.starts_with('#') {
            let key = line.trim_end_matches(':').split(',').next().unwrap_or("").trim().trim_matches('"');
            if let Some(i) = key.rfind('@').filter(|&i| i > 0) {
                name = key[..i].to_string();
            }
        } else if let Some(rest) = line.trim().strip_prefix("version ") {
            let ver = rest.trim().trim_matches('"');
            if !name.is_empty() && !ver.is_empty() {
                out.push(("npm", name.clone(), ver.to_string()));
                name.clear();
            }
        }
    }
}

pub fn parse_poetry_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("poetry.lock")) else { return };
    let (mut name, mut ver) = (String::new(), String::new());
    for line in text.lines() {
        let t = line.trim();
        if t == "[[package]]" {
            if !name.is_empty() && !ver.is_empty() { out.push(("PyPI", name.clone(), ver.clone())); }
            name.clear();
            ver.clear();
        } else if let Some(r) = t.strip_prefix("name = ") {
            name = r.trim_matches('"').to_string();
        } else if let Some(r) = t.strip_prefix("version = ") {
            ver = r.trim_matches('"').to_string();
        }
    }
    if !name.is_empty() && !ver.is_empty() { out.push(("PyPI", name, ver)); }
}

pub fn parse_nuget_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("packages.lock.json")) else { return };
    let Ok(data) = serde_json::from_str::<Value>(&text) else { return };
    let Some(deps) = data.get("dependencies").and_then(Value::as_object) else { return };
    for fw in deps.values().filter_map(Value::as_object) {
        for (name, info) in fw {
            if let Some(ver) = info.get("resolved").and_then(Value::as_str) {
                out.push(("NuGet", name.clone(), ver.to_string()));
            }
        }
    }
}
