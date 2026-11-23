mod common;

use common::{TempDir, json, run};

#[test]
fn multiline_single_quoted_ignore_packages_filters_lock_vulns() {
    let tmp = TempDir::new("cfg-ignore");
    tmp.write(
        "sentinel.toml",
        "ignore_packages = [\n  # ignore lodash\n  'lodash',\n]\n",
    );
    tmp.write(
        "package-lock.json",
        r#"{"lockfileVersion":2,"packages":{"node_modules/lodash":{"version":"4.17.0"}}}"#,
    );
    let out = run(&["--fail-on-cve", "--json", tmp.s()]);
    assert!(out.status.success());
    let v = json(&out);
    assert!(v["lockfile_vulnerabilities"].as_object().unwrap().is_empty());
}

#[test]
fn ignore_dirs_skips_nested_project() {
    let tmp = TempDir::new("cfg-dirs");
    tmp.write("sentinel.toml", "ignore_dirs = ['fixtures']\n");
    tmp.write(
        "fixtures/package-lock.json",
        r#"{"lockfileVersion":2,"packages":{"node_modules/lodash":{"version":"4.17.0"}}}"#,
    );
    let out = run(&["--json", tmp.s()]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
}

#[test]
fn deny_licenses_gpl_does_not_block_lgpl_or_mit() {
    let tmp = TempDir::new("cfg-lic");
    tmp.write("sentinel.toml", "deny_licenses = ['GPL']\n");
    tmp.write("package.json", r#"{"name":"t","dependencies":{"left-pad":"1.0.0"}}"#);
    let out = run(&["--json", tmp.s()]);
    assert!(out.status.success());
}
