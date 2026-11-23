use crate::lockfiles::{LockEntry, resolve_ver};
use crate::version::clean_ver;
use std::collections::BTreeMap;

enum Section<'a> { None, Inline, Table(&'a str) }

fn classify_section(hdr: &str) -> Section<'_> {
    let s = hdr.trim_start_matches('[').trim_end_matches(']').trim();
    for base in ["workspace.dependencies", "dependencies", "dev-dependencies", "build-dependencies"] {
        if s == base || s.ends_with(&format!(".{base}")) { return Section::Inline; }
        if let Some((_, pkg)) = s.split_once(&format!("{base}."))
            && !pkg.is_empty() && !pkg.contains('.')
        {
            return Section::Table(pkg.trim_matches(['"', '\'']));
        }
    }
    Section::None
}

fn extract_ver(val: &str) -> &str {
    let v = val.trim();
    if v.starts_with('"') || v.starts_with('\'') {
        return v.trim_matches(['"', '\'']);
    }
    if let Some(i) = v.find("version") {
        let rest = &v[i..];
        let q = if rest.contains('"') { '"' } else { '\'' };
        return rest.split(q).nth(1).unwrap_or("");
    }
    ""
}

pub fn parse_cargo_deps(text: &str, locked: &BTreeMap<String, Vec<String>>) -> Vec<(String, String)> {
    let (mut deps, mut sec) = (Vec::new(), Section::None);
    for line in text.lines() {
        let t = line.split('#').next().unwrap_or("").trim();
        if t.is_empty() { continue; }
        if t.starts_with('[') { sec = classify_section(t); continue; }
        let Some((k, v)) = t.split_once('=') else { continue };
        let (key, val) = (k.trim(), v.trim());
        match sec {
            Section::Inline => {
                let c = clean_ver(extract_ver(val));
                if !c.is_empty() { deps.push((key.to_string(), resolve_ver(locked, key, &c))); }
            }
            Section::Table(pkg) if key == "version" => {
                let c = clean_ver(val.trim_matches(['"', '\'']));
                if !c.is_empty() { deps.push((pkg.to_string(), resolve_ver(locked, pkg, &c))); }
            }
            _ => {}
        }
    }
    deps
}

pub fn to_direct(deps: &[(String, String)]) -> Vec<LockEntry> {
    deps.iter().map(|(n, v)| ("crates.io", n.clone(), v.clone())).collect()
}
