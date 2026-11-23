mod common;

use common::{TempDir, json, run};

#[test]
fn python_scans_project_not_host_env() {
    let tmp = TempDir::new("py-proj");
    tmp.write("pyproject.toml", "[project]\nname = \"x\"\ndependencies = [\"requests==2.0.0\"]\n");
    let out = run(&["--fail-on-cve", "--json", tmp.s()]);
    assert_eq!(out.status.code(), Some(1));
    let v = json(&out);
    let pkgs = v["outdated"].as_object().unwrap().values().next().unwrap().as_array().unwrap();
    assert_eq!(pkgs.len(), 1);
    assert_eq!(pkgs[0]["name"], "requests");
    assert!(!pkgs[0]["cves"].as_array().unwrap().is_empty());
}

#[test]
fn rust_table_and_target_deps_detected() {
    let tmp = TempDir::new("rs-table");
    tmp.write("Cargo.toml", "[package]\nname = \"t\"\n[dependencies.serde]\nversion = \"1.0.0\"\n[target.'cfg(unix)'.dependencies]\nlibc = \"0.2.0\"\n");
    let out = run(&["--json", tmp.s()]);
    assert!(out.status.success());
    let v = json(&out);
    let names: Vec<_> = v["outdated"].as_object().unwrap().values().next().unwrap()
        .as_array().unwrap().iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"serde"));
    assert!(names.contains(&"libc"));
}

#[test]
fn docker_handles_docker_io_digest_and_warns_unsupported() {
    let tmp = TempDir::new("dock");
    tmp.write("Dockerfile", "FROM docker.io/library/nginx:1.25@sha256:deadbeef\nFROM ghcr.io/org/app:1.2.3\nFROM node:latest\n");
    let out = run(&["--json", tmp.s()]);
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("ghcr.io/org/app:1.2.3"));
    assert!(err.contains("node:latest"));
    let v = json(&out);
    let item = &v["outdated"].as_object().unwrap().values().next().unwrap()[0];
    assert_eq!(item["name"], "library/nginx");
    assert!(!item["cves"].as_array().unwrap().is_empty());
}

#[test]
fn gha_vulnerable_action_fails_cve_gate() {
    let tmp = TempDir::new("gha-cve");
    tmp.write(".github/workflows/ci.yml", "on: [push]\njobs:\n  b:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: tj-actions/changed-files@v40\n");
    let out = run(&["--fail-on-cve", "--json", tmp.s()]);
    assert_eq!(out.status.code(), Some(1));
    let v = json(&out);
    let in_out = v["outdated"].as_object().and_then(|m| m.values().next()).and_then(|a| a[0]["cves"].as_array()).is_some_and(|c| !c.is_empty());
    let in_lock = v["lockfile_vulnerabilities"].as_object().and_then(|m| m.values().next()).and_then(|a| a[0]["ids"].as_array()).is_some_and(|c| !c.is_empty());
    assert!(in_out || in_lock);
}

#[test]
fn swift_v1_checks_swifturl_osv() {
    let tmp = TempDir::new("swift-v1");
    tmp.write("Package.resolved", r#"{"object":{"pins":[{"package":"swift-nio","repositoryURL":"https://github.com/apple/swift-nio.git","state":{"version":"2.0.0"}}]},"version":1}"#);
    let out = run(&["--fail-on-cve", "--json", tmp.s()]);
    assert_eq!(out.status.code(), Some(1));
    let v = json(&out);
    let vulns = v["lockfile_vulnerabilities"].as_object().unwrap().values().next().unwrap();
    assert_eq!(vulns[0]["ecosystem"], "SwiftURL");
    assert_eq!(vulns[0]["package"], "github.com/apple/swift-nio");
}
