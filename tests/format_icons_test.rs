mod common;

use common::TempDir;
use sentinel_scan::{config, output::{format, icons}};

#[test]
fn sanitizes_truncates_and_formats_cve_cells() {
    assert_eq!(format::safe("pkg\x1b[31m\nname"), "pkg[31mname");
    assert_eq!(format::trunc("short", 10), "short");
    assert_eq!(format::trunc("abcdefghijk", 5), "abcd…");
    assert!(format::rule(&[3, 4], "┌", "┬", "┐").contains("─────┬──────"));
    assert_eq!(format::format_cve(&[]).0, "-");
    assert_eq!(format::format_cve(&["CVE-1".into()]).0, "CVE-1");
    assert_eq!(format::format_cve(&["CVE-1".into(), "CVE-2".into(), "CVE-3".into()]).0, "CVE-1 (+2)");
}

#[test]
fn maps_all_ecosystem_icons_and_update_hints() {
    for (m, ic) in [("npm", "🟩"), ("go", "🐹"), ("cargo", "🦀"), ("pip", "🐍"), ("composer", "🐘"), ("dotnet", "🟣"), ("maven", "☕"), ("bundler", "💎"), ("pub", "🐦"), ("hex", "⚡"), ("swiftpm", "🍎"), ("terraform", "🧱"), ("docker", "🐳"), ("actions", "🐙"), ("other", "📦")] {
        assert_eq!(icons::manager_icon(m), ic);
    }
    assert_eq!(icons::severity_style("MAJOR").0, "🔴");
    assert_eq!(icons::severity_style("MINOR").0, "🟡");
    assert_eq!(icons::severity_style("PATCH").0, "🟢");
    assert!(icons::update_cmd("npm", true).contains("@latest"));
    assert_eq!(icons::update_cmd("cargo", false), "cargo update");
}

#[test]
fn parses_sentinel_toml_and_checks_spdx_tokens() {
    let tmp = TempDir::new("toml-unit");
    tmp.write("sentinel.toml", "ignore_packages = [\n  'lodash',\n  # comment\n  \"ureq\",\n]\nignore_dirs = ['examples']\ndeny_licenses = [\"GPL\"]\ncheck_updates = false\n");
    let cfg = config::load(tmp.path());
    assert_eq!(cfg.ignore_packages, vec!["lodash", "ureq"]);
    assert_eq!(cfg.ignore_dirs, vec!["examples"]);
    assert_eq!(cfg.deny_licenses, vec!["GPL"]);
    assert!(!cfg.check_updates);
    assert!(config::SentinelConfig::default().check_updates);

    let deny = vec!["GPL".into(), "MIT".into()];
    assert!(config::is_denied_license("GPL-3.0-only", &deny));
    assert!(config::is_denied_license("MIT OR Apache-2.0", &deny));
    assert!(!config::is_denied_license("LGPL-2.1", &["GPL".into()]));
    assert!(!config::is_denied_license("AGPL-3.0", &["GPL".into()]));
    assert!(!config::is_denied_license("Apache-2.0", &deny));
}
