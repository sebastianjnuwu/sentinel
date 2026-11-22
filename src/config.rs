use std::fs;
use std::path::Path;

#[derive(Debug, Default)]
pub struct SentinelConfig {
    pub ignore_packages: Vec<String>,
    pub ignore_dirs: Vec<String>,
    pub deny_licenses: Vec<String>,
}

fn parse_str_array(raw: &str) -> Vec<String> {
    raw.split('"')
        .skip(1)
        .step_by(2)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn load(root: &Path) -> SentinelConfig {
    let mut cfg = SentinelConfig::default();
    let Ok(text) = fs::read_to_string(root.join("sentinel.toml")) else {
        return cfg;
    };
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        let Some((k, v)) = t.split_once('=') else { continue };
        match k.trim() {
            "ignore_packages" => cfg.ignore_packages = parse_str_array(v),
            "ignore_dirs" => cfg.ignore_dirs = parse_str_array(v),
            "deny_licenses" => cfg.deny_licenses = parse_str_array(v),
            _ => {}
        }
    }
    cfg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_toml_arrays() {
        let dir = std::env::temp_dir().join(format!("sentinel-cfg-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let sample = "ignore_packages = [\"lodash\", \"ureq\"]\nignore_dirs = [\"examples\"]\ndeny_licenses = [\"GPL\"]\n";
        let _ = fs::write(dir.join("sentinel.toml"), sample);
        let cfg = load(&dir);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(cfg.ignore_packages, vec!["lodash", "ureq"]);
        assert_eq!(cfg.ignore_dirs, vec!["examples"]);
        assert_eq!(cfg.deny_licenses, vec!["GPL"]);
    }
}
