use crate::types::OutdatedPkg;
use crate::version::{classify_update, is_numeric_ver, parse_semver_tuple};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::thread;
fn parse_tag(tag: &str) -> Option<(String, String)> {
    let (v, s) = match tag.find('-') { Some(i) => (&tag[..i], &tag[i..]), None => (tag, "") };
    is_numeric_ver(v).then(|| (v.to_string(), s.to_string()))
}

fn extract_images(dir: &Path) -> Vec<(String, String)> {
    let (mut seen, mut out) = (HashSet::new(), Vec::new());
    for file in ["Dockerfile", "docker-compose.yml", "docker-compose.yaml"] {
        let Ok(text) = fs::read_to_string(dir.join(file)) else { continue };
        for line in text.lines() {
            let t = line.trim();
            let raw = if file == "Dockerfile" && t.to_ascii_uppercase().starts_with("FROM ") {
                t.split_whitespace().skip(1).find(|tok| !tok.starts_with("--"))
            } else {
                t.strip_prefix("image:").map(|r| r.trim().trim_matches(|c| c == '"' || c == '\''))
            };
            let Some(img) = raw else { continue };
            if img.contains('$') || img == "scratch" { continue; }
            let Some((repo, tag)) = img.split_once(':') else { continue };
            if !repo.contains('.') && tag != "latest" && parse_tag(tag).is_some() && seen.insert(format!("{repo}:{tag}")) {
                out.push((repo.to_string(), tag.to_string()));
            }
        }
    }
    out
}

fn latest_hub_tag(repo: &str, cur_ver: &str, sfx: &str) -> Option<String> {
    let key = format!("docker:{repo}:{cur_ver}:{sfx}");
    if let Some((c, _)) = crate::cache::get(&key) { return (!c.is_empty()).then_some(c); }
    let hub = if repo.contains('/') { repo.to_string() } else { format!("library/{repo}") };
    let url = format!("https://hub.docker.com/v2/repositories/{hub}/tags?page_size=50&ordering=last_updated");
    let data: Value = crate::http::agent().get(&url).call().ok()?.into_json().ok()?;
    let (cur_n, cur_t) = (cur_ver.split('.').count(), parse_semver_tuple(cur_ver));
    let mut best: Option<((u64, u64, u64), String)> = None;
    for item in data.get("results")?.as_array()? {
        let Some(name) = item.get("name").and_then(Value::as_str) else { continue };
        let Some((v, s)) = parse_tag(name) else { continue };
        let tup = parse_semver_tuple(&v);
        if s == sfx && v.split('.').count() == cur_n && tup > cur_t && best.as_ref().map(|(b, _)| tup > *b).unwrap_or(true) {
            best = Some((tup, name.to_string()));
        }
    }
    let found = best.map(|(_, t)| t);
    crate::cache::set(key, found.clone().unwrap_or_default(), String::new());
    found
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let images = extract_images(dir);
    let mut out = Vec::new();
    thread::scope(|s| {
        let handles: Vec<_> = images.iter().map(|(repo, tag)| s.spawn(move || {
            let (cv, sfx) = parse_tag(tag)?;
            let lat_tag = latest_hub_tag(repo, &cv, &sfx)?;
            let (lv, _) = parse_tag(&lat_tag)?;
            Some(OutdatedPkg { manager: "docker".into(), name: repo.clone(), kind: classify_update(&cv, &lv), current: tag.clone(), latest: lat_tag, license: "-".into(), cves: Vec::new() })
        })).collect();
        for h in handles { if let Ok(Some(item)) = h.join() { out.push(item); } }
    });
    out
}
