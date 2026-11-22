use crate::lockfiles::LockEntry;
use std::fs;
use std::path::Path;

fn parse_toml_packages(path: &Path, eco: &'static str, out: &mut Vec<LockEntry>) {
    let Ok(text) = fs::read_to_string(path) else {
        return;
    };
    let (mut name, mut ver) = (String::new(), String::new());
    for line in text.lines() {
        let t = line.trim();
        if t == "[[package]]" {
            if !name.is_empty() && !ver.is_empty() {
                out.push((eco, name.clone(), ver.clone()));
            }
            name.clear();
            ver.clear();
        } else if let Some(rest) = t.strip_prefix("name = ") {
            name = rest.trim_matches('"').to_string();
        } else if let Some(rest) = t.strip_prefix("version = ") {
            ver = rest.trim_matches('"').to_string();
        }
    }
    if !name.is_empty() && !ver.is_empty() {
        out.push((eco, name, ver));
    }
}

pub fn parse_cargo_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    parse_toml_packages(&dir.join("Cargo.lock"), "crates.io", out);
}

pub fn parse_uv_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    parse_toml_packages(&dir.join("uv.lock"), "PyPI", out);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_lock_entries() {
        let dir = std::env::temp_dir().join(format!("sentinel-test-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let sample = "[[package]]\nname = \"serde\"\nversion = \"1.0.200\"\n\n[[package]]\nname = \"ureq\"\nversion = \"2.12.1\"\n";
        let _ = fs::write(dir.join("Cargo.lock"), sample);
        let mut out = Vec::new();
        parse_cargo_lock(&dir, &mut out);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0], ("crates.io", "serde".into(), "1.0.200".into()));
        assert_eq!(out[1], ("crates.io", "ureq".into(), "2.12.1".into()));
    }
}
