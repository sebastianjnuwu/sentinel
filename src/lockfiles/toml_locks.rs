use crate::lockfiles::LockEntry;
use std::{fs, path::Path};

fn parse_toml_packages(path: &Path, eco: &'static str, cargo_mode: bool, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(path) else { return };
    let (mut name, mut ver, mut src) = (String::new(), String::new(), String::new());
    let flush = |n: &str, v: &str, s: &str, out: &mut Vec<LockEntry>| {
        if n.is_empty() || v.is_empty() { return; }
        let ok = if cargo_mode {
            s.starts_with("registry+")
        } else {
            !s.contains("editable") && !s.contains("directory") && !s.contains("virtual") && !s.contains("git")
        };
        if ok { out.push((eco, n.to_string(), v.to_string())); }
    };
    for line in text.lines() {
        let t = line.trim();
        if t == "[[package]]" {
            flush(&name, &ver, &src, out);
            name.clear(); ver.clear(); src.clear();
        } else if let Some(r) = t.strip_prefix("name = ") { name = r.trim_matches('"').to_string(); }
        else if let Some(r) = t.strip_prefix("version = ") { ver = r.trim_matches('"').to_string(); }
        else if let Some(r) = t.strip_prefix("source = ") { src = r.trim().trim_matches('"').to_string(); }
    }
    flush(&name, &ver, &src, out);
}

pub fn parse_cargo_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    parse_toml_packages(&dir.join("Cargo.lock"), "crates.io", true, out);
}

pub fn parse_uv_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    parse_toml_packages(&dir.join("uv.lock"), "PyPI", false, out);
}
