use crate::types::{OutdatedPkg, TransitiveVuln};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn print_sarif(
    report: &BTreeMap<String, Vec<OutdatedPkg>>,
    lock_vulns: &BTreeMap<String, Vec<TransitiveVuln>>,
) {
    let mut results: Vec<Value> = Vec::new();
    for (proj, items) in report {
        for p in items {
            for id in &p.cves {
                results.push(json!({
                    "ruleId": id,
                    "level": "error",
                    "message": { "text": format!("{}@{} ({}) has vulnerability {} — latest: {}", p.name, p.current, p.manager, id, p.latest) },
                    "locations": [{ "physicalLocation": { "artifactLocation": { "uri": proj } } }]
                }));
            }
        }
    }
    for (proj, vulns) in lock_vulns {
        for v in vulns {
            for id in &v.ids {
                results.push(json!({
                    "ruleId": id,
                    "level": "error",
                    "message": { "text": format!("Transitive {}@{} [{}] has vulnerability {}", v.package, v.version, v.ecosystem, id) },
                    "locations": [{ "physicalLocation": { "artifactLocation": { "uri": proj } } }]
                }));
            }
        }
    }

    let sarif = json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": { "driver": { "name": "Sentinel", "version": env!("CARGO_PKG_VERSION") } },
            "results": results
        }]
    });
    println!("{}", serde_json::to_string_pretty(&sarif).unwrap_or_default());
}
