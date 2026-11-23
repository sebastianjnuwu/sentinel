pub mod license;
#[cfg(test)]
mod tests;

pub use license::is_denied_license;
use std::{fs, path::Path};

#[derive(Debug)]
pub struct SentinelConfig {
    pub ignore_packages: Vec<String>,
    pub ignore_dirs: Vec<String>,
    pub deny_licenses: Vec<String>,
    pub check_updates: bool,
}

impl Default for SentinelConfig {
    fn default() -> Self {
        Self { ignore_packages: Vec::new(), ignore_dirs: Vec::new(), deny_licenses: Vec::new(), check_updates: true }
    }
}

fn parse_str_array(raw: &str) -> Vec<String> {
    let (mut out, mut chars) = (Vec::new(), raw.chars().peekable());
    while let Some(ch) = chars.next() {
        match ch {
            '#' => while chars.next().is_some_and(|c| c != '\n') {},
            '"' | '\'' => {
                let val: String = chars.by_ref().take_while(|&c| c != ch).collect();
                if !val.trim().is_empty() { out.push(val.trim().to_string()); }
            }
            ']' => break,
            _ => {}
        }
    }
    out
}

fn next_assignment<'a>(lines: &mut impl Iterator<Item = &'a str>) -> Option<(String, String)> {
    while let Some(line) = lines.next() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') { continue; }
        let Some((k, v)) = t.split_once('=') else { continue };
        let mut val = v.trim().to_string();
        if val.starts_with('[') && !val.contains(']') {
            for next in lines.by_ref() {
                val.push('\n'); val.push_str(next);
                if next.contains(']') { break; }
            }
        }
        return Some((k.trim().to_string(), val));
    }
    None
}

pub fn load(root: &Path) -> SentinelConfig {
    let mut cfg = SentinelConfig::default();
    let Ok(text) = fs::read_to_string(root.join("sentinel.toml")) else { return cfg };
    let mut lines = text.lines();
    while let Some((k, v)) = next_assignment(&mut lines) {
        match k.as_str() {
            "ignore_packages" => cfg.ignore_packages = parse_str_array(&v),
            "ignore_dirs" => cfg.ignore_dirs = parse_str_array(&v),
            "deny_licenses" => cfg.deny_licenses = parse_str_array(&v),
            "check_updates" => cfg.check_updates = v.split('#').next().unwrap_or("").trim() != "false",
            _ => {}
        }
    }
    cfg
}
