mod common;

use common::{TempDir, run};
use sentinel_scan::{output::{table, vulns}, types::{OutdatedPkg, TransitiveVuln}};
use std::collections::BTreeMap;

#[test]
fn fail_on_major_gate_exits_one_when_major_update_exists() {
    let tmp = TempDir::new("gate-maj");
    tmp.write("Cargo.toml", "[package]\nname = \"t\"\n[dependencies]\nureq = \"2.10.0\"\n");
    let out = run(&["--fail-on-major", "--json", tmp.s()]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn depth_zero_does_not_descend_into_subdirectories() {
    let tmp = TempDir::new("gate-depth");
    tmp.write("sub/package-lock.json", r#"{"lockfileVersion":2,"packages":{"node_modules/lodash":{"version":"4.17.0"}}}"#);
    let d0 = run(&["--depth", "0", "--json", tmp.s()]);
    assert!(d0.status.success());
    assert!(String::from_utf8_lossy(&d0.stdout).trim().is_empty());

    let d1 = run(&["--depth", "1", "--fail-on-cve", "--json", tmp.s()]);
    assert_eq!(d1.status.code(), Some(1));
}

#[test]
fn renders_table_and_lockfile_vulns_without_panicking() {
    let mut rep = BTreeMap::new();
    rep.insert("demo".into(), vec![OutdatedPkg {
        manager: "npm".into(), name: "vite".into(), current: "6.0.0".into(),
        latest: "8.0.0".into(), kind: "MAJOR".into(), license: "MIT".into(),
        cves: vec!["GHSA-1".into(), "GHSA-2".into()],
    }]);
    table::print_table(&rep, 1, 0.12);
    let mut lv = BTreeMap::new();
    lv.insert("demo".into(), vec![TransitiveVuln {
        ecosystem: "npm".into(), package: "lodash".into(),
        version: "4.17.0".into(), ids: vec!["GHSA-3".into()],
    }]);
    vulns::print_lockfile_vulns(&lv);
}
