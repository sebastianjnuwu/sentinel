use crate::cmd;
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver};
use serde_json::Value;
use std::path::Path;

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let raw = cmd::run("dotnet", &["list", "package", "--outdated", "--format", "json"], dir);
    let mut out = Vec::new();
    let Ok(parsed) = serde_json::from_str::<Value>(&raw) else {
        return out;
    };
    let Some(projects) = parsed.get("projects").and_then(Value::as_array) else {
        return out;
    };
    for proj in projects {
        let Some(fws) = proj.get("frameworks").and_then(Value::as_array) else { continue };
        for fw in fws {
            let Some(pkgs) = fw.get("topLevelPackages").and_then(Value::as_array) else { continue };
            for p in pkgs {
                let name = p.get("id").and_then(Value::as_str).unwrap_or("");
                let current = clean_ver(p.get("resolvedVersion").and_then(Value::as_str).unwrap_or(""));
                let latest = clean_ver(p.get("latestVersion").and_then(Value::as_str).unwrap_or(""));
                if !name.is_empty() && current != latest {
                    out.push(OutdatedPkg {
                        manager: "dotnet".into(),
                        name: name.into(),
                        kind: classify_update(&current, &latest),
                        current,
                        latest,
                        license: "-".into(),
                        cves: Vec::new(),
                    });
                }
            }
        }
    }
    out
}
