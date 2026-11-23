use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::{fs, path::Path};

fn escape_mod(m: &str) -> String {
    m.chars().flat_map(|c| if c.is_ascii_uppercase() { vec!['!', c.to_ascii_lowercase()] } else { vec![c] }).collect()
}

pub fn parse_go_direct(dir: &Path) -> Vec<(String, String)> {
    let Ok(text) = fs::read_to_string(dir.join("go.mod")) else { return Vec::new() };
    let (mut reqs, mut reps, mut mode) = (Vec::new(), Vec::new(), "");
    for line in text.lines() {
        let indirect = line.contains("// indirect");
        let t = line.split("//").next().unwrap_or("").trim();
        if t.is_empty() || t.starts_with("module ") || t.starts_with("go ") { continue; }
        if t == ")" { mode = ""; continue; }
        if t.starts_with("require (") { mode = "req"; continue; }
        if t.starts_with("replace (") { mode = "rep"; continue; }
        if t.starts_with("replace ") || mode == "rep" {
            if let Some((l, r)) = t.strip_prefix("replace ").unwrap_or(t).split_once("=>") {
                let (mut lp, mut rp) = (l.split_whitespace(), r.split_whitespace());
                if let (Some(om), Some(nm)) = (lp.next(), rp.next()) {
                    reps.push((om.to_string(), lp.next().map(clean_ver), nm.to_string(), rp.next().map(clean_ver).unwrap_or_default()));
                }
            }
        } else if !indirect && (t.starts_with("require ") || mode == "req") {
            let mut p = t.strip_prefix("require ").unwrap_or(t).split_whitespace();
            if let (Some(m), Some(v)) = (p.next(), p.next()) { reqs.push((m.to_string(), clean_ver(v))); }
        }
    }
    reqs.into_iter().filter_map(|(m, v)| match reps.iter().find(|(om, ov, _, _)| om == &m && ov.as_deref().is_none_or(|x| x == v)) {
        Some((_, _, nm, _)) if nm.starts_with('.') || nm.starts_with('/') => None,
        Some((_, _, nm, nv)) if !nv.is_empty() => Some((nm.clone(), nv.clone())),
        Some((_, _, nm, _)) => Some((nm.clone(), v)),
        None => Some((m, v)),
    }).collect()
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let deps = parse_go_direct(dir);
    crate::pool::map_bounded(&deps, |(name, current)| {
        let key = format!("go:{name}");
        let latest = if let Some((t, _)) = crate::cache::get(&key) { t } else {
            let url = format!("https://proxy.golang.org/{}/@latest", escape_mod(name));
            let data: Value = crate::http::agent().get(&url).call().ok()?.into_body().read_json().ok()?;
            let lat = clean_ver(data.get("Version")?.as_str()?);
            crate::cache::set(key, lat.clone(), "-".into());
            lat
        };
        is_newer(current, &latest).then(|| OutdatedPkg {
            manager: "go".into(), name: name.clone(), kind: classify_update(current, &latest),
            current: current.clone(), latest, license: "-".into(), cves: Vec::new(),
        })
    })
}
