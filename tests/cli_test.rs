mod common;

use common::{TempDir, run};
use std::process::Command;

#[test]
fn prints_version_and_help() {
    let v = run(&["--version"]);
    assert!(v.status.success());
    assert!(String::from_utf8_lossy(&v.stdout).contains("sentinel 1.0.0"));

    let h = run(&["--help"]);
    assert!(h.status.success());
    let s = String::from_utf8_lossy(&h.stdout);
    for flag in ["--json", "--sarif", "--fail-on-cve", "--fail-on-major", "--depth", "--no-cache", "--update"] {
        assert!(s.contains(flag), "missing {flag}");
    }
}

#[test]
fn handles_empty_directory_and_json_mode() {
    let tmp = TempDir::new("empty");
    let out = run(&[tmp.s()]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("No supported projects found"));

    let json_out = run(&["--json", tmp.s()]);
    assert!(json_out.status.success());
    assert!(String::from_utf8_lossy(&json_out.stdout).trim().is_empty());
}

#[test]
fn rejects_unknown_flag_invalid_depth_and_bad_paths() {
    let bad_flag = run(&["--fail-on-cves", "."]);
    assert_eq!(bad_flag.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bad_flag.stderr).contains("unknown flag"));

    let bad_depth = run(&["--depth", "abc", "."]);
    assert_eq!(bad_depth.status.code(), Some(2));

    let missing = run(&["nonexistent_sentinel_dir_99999"]);
    assert_eq!(missing.status.code(), Some(2));

    let tmp = TempDir::new("file-arg");
    tmp.write("a.txt", "x");
    let file_arg = run(&[tmp.path().join("a.txt").to_str().unwrap()]);
    assert_eq!(file_arg.status.code(), Some(2));
}

#[test]
fn piped_and_no_color_emit_no_ansi_escapes() {
    let tmp = TempDir::new("noansi");
    tmp.write("package-lock.json", r#"{"lockfileVersion":2,"packages":{"node_modules/lodash":{"version":"4.17.0"}}}"#);
    let out = Command::new(common::sentinel_bin())
        .env("NO_COLOR", "1")
        .arg(tmp.s())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(!stdout.contains('\x1b'));
}
