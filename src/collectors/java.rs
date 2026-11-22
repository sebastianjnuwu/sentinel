use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::thread;

fn tag_val<'a>(block: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = block.find(&open)? + open.len();
    let end = block[start..].find(&close)?;
    Some(block[start..start + end].trim())
}

fn parse_pom(dir: &Path) -> Vec<(String, String, String)> {
    let Ok(text) = fs::read_to_string(dir.join("pom.xml")) else { return Vec::new() };
    let mut out = Vec::new();
    for chunk in text.split("<dependency>").skip(1) {
        let Some(block) = chunk.split("</dependency>").next() else { continue };
        let (Some(g), Some(a), Some(v)) = (tag_val(block, "groupId"), tag_val(block, "artifactId"), tag_val(block, "version")) else { continue };
        if !v.starts_with('$') { out.push((g.to_string(), a.to_string(), v.to_string())); }
    }
    out
}

fn parse_gradle(dir: &Path) -> Vec<(String, String, String)> {
    let text = fs::read_to_string(dir.join("build.gradle.kts"))
        .or_else(|_| fs::read_to_string(dir.join("build.gradle")))
        .unwrap_or_default();
    let mut out = Vec::new();
    for line in text.lines() {
        for quote in ['"', '\''] {
            for part in line.split(quote).skip(1).step_by(2) {
                let segs: Vec<&str> = part.split(':').collect();
                if segs.len() == 3 && segs[2].chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    out.push((segs[0].to_string(), segs[1].to_string(), segs[2].to_string()));
                }
            }
        }
    }
    out
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let (mgr, deps) = if dir.join("pom.xml").exists() { ("maven", parse_pom(dir)) } else { ("gradle", parse_gradle(dir)) };
    let mut out = Vec::new();
    thread::scope(|s| {
        let handles: Vec<_> = deps.iter().map(|(g, a, v)| {
            s.spawn(move || {
                let key = format!("maven:{g}:{a}");
                let latest = if let Some((t, _)) = crate::cache::get(&key) { t } else {
                    let url = format!("https://search.maven.org/solrsearch/select?q=g:{g}+AND+a:{a}&rows=1&wt=json");
                    let data: Value = crate::http::agent().get(&url).call().ok()?.into_json().ok()?;
                    let l = data.get("response")?.get("docs")?.as_array()?.first()?.get("latestVersion")?.as_str()?.to_string();
                    crate::cache::set(key, l.clone(), "-".into());
                    l
                };
                let cur = clean_ver(v);
                is_newer(&cur, &latest).then(|| OutdatedPkg {
                    manager: mgr.into(), name: format!("{g}:{a}"), kind: classify_update(&cur, &latest),
                    current: cur, latest, license: "-".into(), cves: Vec::new(),
                })
            })
        }).collect();
        for h in handles { if let Ok(Some(item)) = h.join() { out.push(item); } }
    });
    out
}
