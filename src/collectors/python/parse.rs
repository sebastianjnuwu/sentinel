use crate::lockfiles::resolve_ver;
use crate::version::clean_ver;
use std::{collections::BTreeMap, fs, path::Path};

fn parse_req_spec(raw: &str, locked: &BTreeMap<String, Vec<String>>) -> Option<(String, String)> {
    let clean = raw.split(['#', ';']).next()?.trim();
    if clean.is_empty() || clean.starts_with('-') { return None; }
    let ops = ["==", ">=", "<=", "~=", "!=", ">", "<"];
    let pos = ops.iter().filter_map(|op| clean.find(op).map(|i| (i, op.len()))).min_by_key(|x| x.0);
    let (name_raw, ver_raw) = match pos {
        Some((i, len)) => (&clean[..i], &clean[i + len..]),
        None => (clean, ""),
    };
    let name = name_raw.split('[').next()?.trim().to_string();
    if name.is_empty() || name.eq_ignore_ascii_case("python") { return None; }
    let ver = resolve_ver(locked, &name, &clean_ver(ver_raw));
    (!ver.is_empty()).then_some((name, ver))
}

fn parse_pyproject(text: &str, locked: &BTreeMap<String, Vec<String>>, out: &mut Vec<(String, String)>) {
    let (mut in_arr, mut in_poetry) = (false, false);
    for line in text.lines() {
        let t = line.split('#').next().unwrap_or("").trim();
        if t.starts_with('[') { in_poetry = t == "[tool.poetry.dependencies]"; in_arr = false; continue; }
        if t.starts_with("dependencies") && t.contains('[') { in_arr = true; }
        if in_arr {
            for s in t.split(['"', '\'']).skip(1).step_by(2) {
                if let Some(d) = parse_req_spec(s, locked) { out.push(d); }
            }
            if t.contains(']') { in_arr = false; }
        } else if in_poetry && let Some((k, v)) = t.split_once('=') {
            let ver = v.trim().trim_matches(['"', '\'']);
            let c = if ver.contains("version") { ver.split(['"', '\'']).nth(3).unwrap_or("") } else { ver };
            let resolved = resolve_ver(locked, k.trim(), &clean_ver(c));
            if k.trim() != "python" && !resolved.is_empty() { out.push((k.trim().to_string(), resolved)); }
        }
    }
}

pub fn parse_python_deps(dir: &Path, locked: &BTreeMap<String, Vec<String>>) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Ok(req) = fs::read_to_string(dir.join("requirements.txt")) {
        for line in req.lines() {
            if let Some(d) = parse_req_spec(line, locked) { out.push(d); }
        }
    }
    if let Ok(py) = fs::read_to_string(dir.join("pyproject.toml")) {
        parse_pyproject(&py, locked, &mut out);
    }
    out
}
