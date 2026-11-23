use crate::types::{OutdatedPkg, TransitiveVuln};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn manifest_file(manager: &str) -> &'static str {
    match manager {
        "cargo" | "crates.io" => "Cargo.toml",
        "go" | "Go" => "go.mod",
        "maven" | "Maven" => "pom.xml",
        "gradle" => "build.gradle",
        "composer" | "Packagist" => "composer.json",
        "bundler" | "RubyGems" => "Gemfile",
        "pub" | "Pub" => "pubspec.yaml",
        "hex" | "Hex" => "mix.exs",
        "pip" | "uv" | "PyPI" => "pyproject.toml",
        _ => "package.json",
    }
}

fn loc(proj: &str, mgr: &str) -> Value {
    let f = manifest_file(mgr);
    let uri = if proj == "." { f.into() } else { format!("{}/{f}", proj.replace('\\', "/")) };
    json!([{ "physicalLocation": { "artifactLocation": { "uri": uri } } }])
}

fn sarif_result(id: &str, msg: String, locs: Value) -> Value {
    json!({ "ruleId": id, "level": "error", "message": { "text": msg }, "locations": locs })
}

pub fn print_sarif(report: &BTreeMap<String, Vec<OutdatedPkg>>, lock_vulns: &BTreeMap<String, Vec<TransitiveVuln>>) {
    let (mut results, mut rule_ids) = (Vec::new(), BTreeSet::new());
    for (proj, items) in report {
        for p in items {
            let locs = loc(proj, &p.manager);
            for id in &p.cves {
                rule_ids.insert(id.clone());
                let msg = format!("{}@{} ({}) has vulnerability {} — latest: {}", p.name, p.current, p.manager, id, p.latest);
                results.push(sarif_result(id, msg, locs.clone()));
            }
        }
    }
    for (proj, vulns) in lock_vulns {
        for v in vulns {
            let locs = loc(proj, &v.ecosystem);
            for id in &v.ids {
                rule_ids.insert(id.clone());
                let msg = format!("Transitive {}@{} [{}] has vulnerability {}", v.package, v.version, v.ecosystem, id);
                results.push(sarif_result(id, msg, locs.clone()));
            }
        }
    }
    let rules: Vec<Value> = rule_ids.into_iter().map(|id| json!({ "id": id, "shortDescription": { "text": format!("Vulnerability {id}") } })).collect();
    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{ "tool": { "driver": { "name": "Sentinel", "version": env!("CARGO_PKG_VERSION"), "rules": rules } }, "results": results }]
    });
    println!("{}", serde_json::to_string_pretty(&sarif).unwrap_or_default());
}
