use crate::lockfiles::LockEntry;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn parse_go_sum(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("go.sum")) else { return };
    for line in text.lines() {
        let mut p = line.split_whitespace();
        let (Some(m), Some(v)) = (p.next(), p.next()) else { continue };
        let v = v.trim_end_matches("/go.mod").trim_start_matches('v');
        if !m.is_empty() && !v.is_empty() { out.push(("Go", m.to_string(), v.to_string())); }
    }
}

pub fn parse_composer_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("composer.lock")) else { return };
    let Ok(data) = serde_json::from_str::<Value>(&text) else { return };
    for key in ["packages", "packages-dev"] {
        let Some(arr) = data.get(key).and_then(Value::as_array) else { continue };
        for item in arr {
            let (Some(n), Some(v)) = (item.get("name").and_then(Value::as_str), item.get("version").and_then(Value::as_str)) else { continue };
            out.push(("Packagist", n.to_string(), v.trim_start_matches('v').to_string()));
        }
    }
}

pub fn parse_gemfile_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("Gemfile.lock")) else { return };
    let mut in_specs = false;
    for line in text.lines() {
        if line.trim() == "specs:" { in_specs = true; continue; }
        if in_specs && !line.starts_with(' ') { in_specs = false; continue; }
        if in_specs && line.starts_with("    ") && !line.starts_with("      ") {
            if let Some((n, r)) = line.trim().split_once(" (") {
                let v = r.trim_end_matches(')');
                if !n.is_empty() && !v.is_empty() { out.push(("RubyGems", n.to_string(), v.to_string())); }
            }
        }
    }
}

pub fn parse_pubspec_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("pubspec.lock")) else { return };
    let mut pkg = String::new();
    for line in text.lines() {
        if line.starts_with("  ") && !line.starts_with("    ") && line.trim_end().ends_with(':') {
            pkg = line.trim().trim_end_matches(':').to_string();
        } else if line.starts_with("    version:") && !pkg.is_empty() {
            let v = line.trim().trim_start_matches("version:").trim().trim_matches('"');
            if !v.is_empty() { out.push(("Pub", pkg.clone(), v.to_string())); }
        }
    }
}

pub fn parse_mix_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(dir.join("mix.lock")) else { return };
    for line in text.lines() {
        if let Some((p, r)) = line.trim().split_once(": {:hex,") {
            let n = p.trim().trim_matches('"');
            let parts: Vec<&str> = r.split('"').collect();
            if parts.len() >= 2 && !n.is_empty() { out.push(("Hex", n.to_string(), parts[1].to_string())); }
        }
    }
}
