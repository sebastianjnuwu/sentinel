mod common;

use common::{TempDir, json, run};

#[test]
fn sarif_emits_rules_level_and_no_fake_start_line() {
    let tmp = TempDir::new("sarif");
    tmp.write("package-lock.json", r#"{"lockfileVersion":1,"dependencies":{"lodash":{"version":"4.17.0"}}}"#);
    let out = run(&["--sarif", tmp.s()]);
    assert!(out.status.success());
    let v = json(&out);
    assert_eq!(v["version"], "2.1.0");
    let rules = v["runs"][0]["tool"]["driver"]["rules"].as_array().unwrap();
    let results = v["runs"][0]["results"].as_array().unwrap();
    assert!(rules.len() > 2);
    assert_eq!(results[0]["level"], "error");
    assert_eq!(results[0]["locations"][0]["physicalLocation"]["artifactLocation"]["uri"], "package.json");
    assert!(results[0]["locations"][0]["physicalLocation"]["region"].is_null());
}

#[test]
fn up_to_date_vulnerable_direct_dep_reported() {
    let tmp = TempDir::new("uptodate-vuln");
    tmp.write("package.json", r#"{"name":"t","dependencies":{"request":"2.88.2"}}"#);
    let out = run(&["--fail-on-cve", "--json", tmp.s()]);
    assert_eq!(out.status.code(), Some(1));
    let v = json(&out);
    let vulns = v["lockfile_vulnerabilities"].as_object().unwrap().values().next().unwrap();
    assert_eq!(vulns[0]["package"], "request");
    assert!(!vulns[0]["ids"].as_array().unwrap().is_empty());
}
