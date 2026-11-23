mod common;

use common::{TempDir, json, run};

#[test]
fn lockfile_only_npm_v2_reports_all_cves_untruncated() {
    let tmp = TempDir::new("npm-v2");
    tmp.write("package-lock.json", r#"{"lockfileVersion":2,"packages":{"node_modules/lodash":{"version":"4.17.0"}}}"#);
    let out = run(&["--fail-on-cve", "--json", tmp.s()]);
    assert_eq!(out.status.code(), Some(1));
    let v = json(&out);
    let ids = v["lockfile_vulnerabilities"].as_object().unwrap().values().next().unwrap()[0]["ids"].as_array().unwrap();
    assert!(ids.len() > 2, "expected >2 CVEs, got {}", ids.len());
}

#[test]
fn npm_v1_and_yarn_berry_v2_report_vulns() {
    let npm1 = TempDir::new("npm-v1");
    npm1.write("package-lock.json", r#"{"lockfileVersion":1,"dependencies":{"lodash":{"version":"4.17.0"}}}"#);
    assert_eq!(run(&["--fail-on-cve", "--json", npm1.s()]).status.code(), Some(1));

    let yarn2 = TempDir::new("yarn-v2");
    yarn2.write("yarn.lock", "__metadata:\n  version: 8\n\"lodash@npm:^4.17.0\":\n  version: 4.17.0\n");
    assert_eq!(run(&["--fail-on-cve", "--json", yarn2.s()]).status.code(), Some(1));
}

#[test]
fn go_sum_and_replace_directive() {
    let tmp = TempDir::new("go-sum");
    tmp.write("go.mod", "module t\ngo 1.16\nrequire github.com/gorilla/mux v1.8.0\nreplace github.com/gin-gonic/gin v1.6.0 => ./local\n");
    tmp.write("go.sum", "github.com/gin-gonic/gin v1.7.0 h1:a\ngithub.com/gin-gonic/gin v1.6.0 h1:b\n");
    let out = run(&["--fail-on-cve", "--json", tmp.s()]);
    assert_eq!(out.status.code(), Some(1));
    let v = json(&out);
    let list = v["lockfile_vulnerabilities"].as_object().unwrap().values().next().unwrap().as_array().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["version"], "1.7.0");
}

#[test]
fn cargo_lock_ignores_git_and_parent_lockfile_isolated() {
    let rs = TempDir::new("cargo-git");
    rs.write("Cargo.lock", "[[package]]\nname = \"time\"\nversion = \"0.1.44\"\nsource = \"git+https://github.com/example/time#abc\"\n");
    let rs_out = run(&["--fail-on-cve", "--json", rs.s()]);
    assert!(rs_out.status.success());

    let parent = TempDir::new("parent-iso");
    parent.write("package-lock.json", r#"{"lockfileVersion":2,"packages":{"node_modules/lodash":{"version":"4.17.0"}}}"#);
    parent.write("sub/package.json", r#"{"name":"sub","dependencies":{"left-pad":"1.3.0"}}"#);
    let sub_out = run(&["--fail-on-cve", "--json", parent.path().join("sub").to_str().unwrap()]);
    assert!(sub_out.status.success());
}
