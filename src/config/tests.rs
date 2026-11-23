use super::*;
use std::fs;

#[test]
fn parses_multiline_and_single_quoted_toml() {
    let dir = std::env::temp_dir().join(format!("sentinel-cfg-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let sample = "ignore_packages = [\n  'lodash',\n  # comment\n  \"ureq\",\n]\nignore_dirs = ['examples']\ndeny_licenses = [\"GPL\"]\ncheck_updates = false\n";
    let _ = fs::write(dir.join("sentinel.toml"), sample);
    let cfg = load(&dir);
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(cfg.ignore_packages, vec!["lodash", "ureq"]);
    assert_eq!(cfg.ignore_dirs, vec!["examples"]);
    assert_eq!(cfg.deny_licenses, vec!["GPL"]);
    assert!(!cfg.check_updates);
    assert!(SentinelConfig::default().check_updates);
}
