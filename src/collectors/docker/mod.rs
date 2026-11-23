mod parse;
mod sbom;

use crate::types::OutdatedPkg;
use crate::version::{classify_update, parse_semver_tuple};
use serde_json::Value;
use std::path::Path;

fn latest_hub_tag(repo: &str, cur_ver: &str, sfx: &str) -> Option<String> {
    let key = format!("docker:{repo}:{cur_ver}:{sfx}");
    if let Some((c, _)) = crate::cache::get(&key) { return (!c.is_empty()).then_some(c); }
    let hub = if repo.contains('/') { repo.to_string() } else { format!("library/{repo}") };
    let url = format!("https://hub.docker.com/v2/repositories/{hub}/tags?page_size=100&ordering=last_updated");
    let data: Value = crate::http::agent().get(&url).call().ok()?.into_body().read_json().ok()?;
    let (cur_n, cur_t) = (cur_ver.split('.').count(), parse_semver_tuple(cur_ver));
    let mut best: Option<((u64, u64, u64), String)> = None;
    for item in data.get("results")?.as_array()? {
        let Some(name) = item.get("name").and_then(Value::as_str) else { continue };
        let Some((v, s)) = parse::parse_tag(name) else { continue };
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
    let images = parse::extract_images(dir);
    crate::pool::map_bounded(&images, |(repo, tag)| {
        let (cv, sfx) = parse::parse_tag(tag)?;
        let lat_tag = latest_hub_tag(repo, &cv, &sfx)?;
        let (lv, _) = parse::parse_tag(&lat_tag)?;
        let cves = sbom::image_cves(repo, tag);
        Some(OutdatedPkg {
            manager: "docker".into(), name: repo.clone(),
            kind: classify_update(&cv, &lv), current: tag.clone(),
            latest: lat_tag, license: "-".into(), cves,
        })
    })
}
