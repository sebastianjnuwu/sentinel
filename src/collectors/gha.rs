use crate::lockfiles::LockEntry;
use crate::types::OutdatedPkg;
use crate::version::{classify_update, is_newer, is_numeric_ver};
use serde_json::Value;
use std::{collections::HashSet, env, fs, path::Path};

fn extract_actions(dir: &Path) -> Vec<(String, String)> {
    let Ok(entries) = fs::read_dir(dir.join(".github").join("workflows")) else { return Vec::new() };
    let (mut seen, mut out) = (HashSet::new(), Vec::new());
    for path in entries.flatten().map(|e| e.path()) {
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "yml" && ext != "yaml" { continue; }
        let Ok(text) = fs::read_to_string(&path) else { continue };
        for line in text.lines() {
            let Some(rest) = line.trim().trim_start_matches('-').trim().strip_prefix("uses:") else { continue };
            let val = rest.split('#').next().unwrap_or("").trim().trim_matches(['"', '\'']);
            if val.starts_with("./") || val.starts_with("docker://") { continue; }
            let Some((act, ver)) = val.split_once('@') else { continue };
            let parts: Vec<&str> = act.split('/').collect();
            if parts.len() >= 2 && is_numeric_ver(ver.trim().trim_start_matches('v')) {
                let repo = format!("{}/{}", parts[0], parts[1]);
                if seen.insert(format!("{repo}@{ver}")) { out.push((repo, ver.to_string())); }
            }
        }
    }
    out
}

fn fetch_latest_tag(repo: &str, tok: Option<&str>) -> Option<String> {
    let get = |url: &str| -> Option<Value> {
        let mut req = crate::http::agent().get(url).header("User-Agent", crate::http::user_agent());
        if let Some(t) = tok { req = req.header("Authorization", &format!("Bearer {t}")); }
        req.call().ok()?.into_body().read_json().ok()
    };
    if let Some(tag) = get(&format!("https://api.github.com/repos/{repo}/releases/latest"))
        .and_then(|d| d.get("tag_name").and_then(Value::as_str).map(String::from))
    {
        return Some(tag);
    }
    get(&format!("https://api.github.com/repos/{repo}/tags?per_page=10"))?
        .as_array()?.iter().filter_map(|t| t.get("name").and_then(Value::as_str))
        .find(|t| is_numeric_ver(t.trim_start_matches('v'))).map(String::from)
}

pub fn collect(dir: &Path) -> (Vec<OutdatedPkg>, Vec<LockEntry>) {
    let actions = extract_actions(dir);
    let direct = actions.iter().map(|(r, v)| ("GitHub Actions", r.clone(), v.clone())).collect();
    let token = env::var("GITHUB_TOKEN").ok();
    let out = crate::pool::map_bounded(&actions, |(repo, cur_tag)| {
        let key = format!("gha:{repo}");
        let lat_tag = if let Some((t, _)) = crate::cache::get(&key) { t } else {
            let t = fetch_latest_tag(repo, token.as_deref())?;
            crate::cache::set(key, t.clone(), "-".into());
            t
        };
        let (cur, lat) = (cur_tag.trim_start_matches('v'), lat_tag.trim_start_matches('v'));
        if !is_numeric_ver(lat) { return None; }
        let behind = if cur.contains('.') {
            is_newer(cur, lat)
        } else {
            lat.split('.').next().unwrap_or("0").parse::<u64>().unwrap_or(0) > cur.parse::<u64>().unwrap_or(0)
        };
        let kind = classify_update(cur, lat);
        behind.then(|| OutdatedPkg {
            manager: "actions".into(), name: repo.clone(), kind,
            current: cur_tag.clone(), latest: lat_tag, license: "-".into(), cves: Vec::new(),
        })
    });
    (out, direct)
}
