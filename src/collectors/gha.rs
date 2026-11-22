use crate::types::OutdatedPkg;
use crate::version::{classify_update, is_numeric_ver};
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;

fn extract_actions(dir: &Path) -> Vec<(String, String)> {
    let Ok(entries) = fs::read_dir(dir.join(".github").join("workflows")) else {
        return Vec::new();
    };
    let (mut seen, mut out) = (HashSet::new(), Vec::new());
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "yml" && ext != "yaml" { continue; }
        let Ok(text) = fs::read_to_string(&path) else { continue };
        for line in text.lines() {
            let Some(rest) = line.trim().trim_start_matches('-').trim().strip_prefix("uses:") else { continue };
            let val = rest.split('#').next().unwrap_or("").trim().trim_matches(|c| c == '"' || c == '\'');
            if val.starts_with("./") || val.starts_with("docker://") { continue; }
            let Some((act, ver)) = val.split_once('@') else { continue };
            let parts: Vec<&str> = act.split('/').collect();
            let vc = ver.trim().trim_start_matches('v');
            if parts.len() >= 2 && is_numeric_ver(vc) {
                let repo = format!("{}/{}", parts[0], parts[1]);
                if seen.insert(format!("{repo}@{ver}")) { out.push((repo, ver.to_string())); }
            }
        }
    }
    out
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let actions = extract_actions(dir);
    let token = env::var("GITHUB_TOKEN").ok();
    let mut out = Vec::new();
    thread::scope(|s| {
        let handles: Vec<_> = actions.iter().map(|(repo, cur_tag)| {
            let tok = token.as_deref();
            s.spawn(move || {
                let key = format!("gha:{repo}");
                let lat_tag = if let Some((t, _)) = crate::cache::get(&key) { t } else {
                    let mut req = crate::http::agent().get(&format!("https://api.github.com/repos/{repo}/releases/latest")).set("User-Agent", "sentinel-deps-cli/0.1");
                    if let Some(t) = tok { req = req.set("Authorization", &format!("Bearer {t}")); }
                    let data: Value = req.call().ok()?.into_json().ok()?;
                    let t = data.get("tag_name")?.as_str()?.to_string();
                    crate::cache::set(key, t.clone(), "-".into());
                    t
                };
                let (cur, lat) = (cur_tag.trim_start_matches('v'), lat_tag.trim_start_matches('v'));
                if !is_numeric_ver(lat) { return None; }
                let cm = cur.split('.').next().unwrap_or("0").parse::<u64>().unwrap_or(0);
                let lm = lat.split('.').next().unwrap_or("0").parse::<u64>().unwrap_or(0);
                let behind = if cur.contains('.') { cur != lat && lm >= cm } else { lm > cm };
                let kind = classify_update(cur, lat);
                behind.then(|| OutdatedPkg {
                    manager: "actions".into(), name: repo.clone(), kind,
                    current: cur_tag.clone(), latest: lat_tag, license: "-".into(), cves: Vec::new(),
                })
            })
        }).collect();
        for h in handles { if let Ok(Some(item)) = h.join() { out.push(item); } }
    });
    out
}
