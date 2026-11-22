use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;

fn parse_swift_resolved(dir: &Path) -> Vec<(String, String, String)> {
    let Ok(text) = fs::read_to_string(dir.join("Package.resolved")) else { return Vec::new() };
    let Ok(data) = serde_json::from_str::<Value>(&text) else { return Vec::new() };
    let Some(pins) = data.get("pins").and_then(Value::as_array) else { return Vec::new() };
    let mut out = Vec::new();
    for pin in pins {
        let Some(loc) = pin.get("location").and_then(Value::as_str) else { continue };
        let Some(ver) = pin.get("state").and_then(|s| s.get("version")).and_then(Value::as_str) else { continue };
        let Some(gh) = loc.strip_prefix("https://github.com/") else { continue };
        let repo = gh.trim_end_matches(".git").to_string();
        let name = pin.get("identity").and_then(Value::as_str).unwrap_or(&repo).to_string();
        out.push((name, repo, clean_ver(ver)));
    }
    out
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let pins = parse_swift_resolved(dir);
    let token = env::var("GITHUB_TOKEN").ok();
    let mut out = Vec::new();
    thread::scope(|s| {
        let handles: Vec<_> = pins.iter().map(|(name, repo, cur)| {
            let tok = token.as_deref();
            s.spawn(move || {
                let key = format!("swift:{repo}");
                let latest = if let Some((t, _)) = crate::cache::get(&key) { t } else {
                    let mut req = crate::http::agent().get(&format!("https://api.github.com/repos/{repo}/releases/latest")).set("User-Agent", "sentinel-deps-cli/0.1");
                    if let Some(t) = tok { req = req.set("Authorization", &format!("Bearer {t}")); }
                    let data: Value = req.call().ok()?.into_json().ok()?;
                    let lat = clean_ver(data.get("tag_name")?.as_str()?);
                    crate::cache::set(key, lat.clone(), "-".into());
                    lat
                };
                is_newer(cur, &latest).then(|| OutdatedPkg {
                    manager: "swiftpm".into(), name: name.clone(), kind: classify_update(cur, &latest),
                    current: cur.clone(), latest, license: "-".into(), cves: Vec::new(),
                })
            })
        }).collect();
        for h in handles { if let Ok(Some(item)) = h.join() { out.push(item); } }
    });
    out
}
