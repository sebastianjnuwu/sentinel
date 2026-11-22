use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn sentinel_bin() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_BIN_EXE_sentinel"));
    if p.exists() {
        p
    } else {
        std::env::current_dir().unwrap().join("target").join("debug").join(if cfg!(windows) { "sentinel.exe" } else { "sentinel" })
    }
}

#[test]
fn prints_version() {
    let out = Command::new(sentinel_bin()).arg("--version").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("sentinel 1.0.0"));
}

#[test]
fn prints_help() {
    let out = Command::new(sentinel_bin()).arg("--help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("--json"));
    assert!(stdout.contains("--sarif"));
    assert!(stdout.contains("--fail-on-cve"));
    assert!(stdout.contains("--no-cache"));
}

#[test]
fn handles_empty_directory() {
    let dir = std::env::temp_dir().join(format!("sentinel-empty-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let out = Command::new(sentinel_bin()).arg(&dir).output().unwrap();
    let _ = fs::remove_dir_all(&dir);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("No supported projects found"));
}

#[test]
fn json_mode_on_empty_dir_is_silent() {
    let dir = std::env::temp_dir().join(format!("sentinel-empty-json-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let out = Command::new(sentinel_bin())
        .args(["--json", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&dir);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).trim().is_empty());
}
