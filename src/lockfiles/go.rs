use crate::lockfiles::LockEntry;
use std::{collections::BTreeSet, fs, path::Path};

struct Replace { old_mod: String, old_ver: Option<String>, new_mod: String, new_ver: String }

fn clean_go_ver(v: &str) -> String {
    v.trim_start_matches('v').trim_end_matches("+incompatible").to_string()
}

fn parse_replace_rule(line: &str) -> Option<Replace> {
    let (lhs, rhs) = line.split_once("=>")?;
    let mut lp = lhs.split_whitespace();
    let old_mod = lp.next()?.to_string();
    let old_ver = lp.next().map(clean_go_ver);
    let mut rp = rhs.split_whitespace();
    let new_mod = rp.next()?.to_string();
    let new_ver = rp.next().map(clean_go_ver).unwrap_or_default();
    Some(Replace { old_mod, old_ver, new_mod, new_ver })
}

fn parse_go_mod(dir: &Path) -> (Vec<(String, String)>, Vec<Replace>) {
    let Ok(text) = fs::read_to_string(dir.join("go.mod")) else { return (Vec::new(), Vec::new()) };
    let (mut reqs, mut reps, mut mode) = (Vec::new(), Vec::new(), "");
    for line in text.lines() {
        let t = line.split("//").next().unwrap_or("").trim();
        if t.is_empty() || t.starts_with("module ") || t.starts_with("go ") { continue; }
        if t == ")" { mode = ""; continue; }
        if t.starts_with("require (") { mode = "req"; continue; }
        if t.starts_with("replace (") { mode = "rep"; continue; }
        if t.starts_with("replace ") || mode == "rep" {
            if let Some(r) = parse_replace_rule(t.strip_prefix("replace ").unwrap_or(t)) { reps.push(r); }
        } else if t.starts_with("require ") || mode == "req" {
            let mut p = t.strip_prefix("require ").unwrap_or(t).split_whitespace();
            if let (Some(m), Some(v)) = (p.next(), p.next()) { reqs.push((m.to_string(), clean_go_ver(v))); }
        }
    }
    (reqs, reps)
}

pub fn parse_go_lock(dir: &Path, out: &mut Vec<LockEntry>) {
    let (mut entries, reps) = parse_go_mod(dir);
    if let Ok(sum) = fs::read_to_string(dir.join("go.sum")) {
        for line in sum.lines() {
            let mut p = line.split_whitespace();
            if let (Some(m), Some(v)) = (p.next(), p.next()) {
                entries.push((m.to_string(), clean_go_ver(v.trim_end_matches("/go.mod"))));
            }
        }
    }
    let mut seen = BTreeSet::new();
    for (mut m, mut v) in entries {
        if let Some(r) = reps.iter().find(|r| r.old_mod == m && r.old_ver.as_deref().is_none_or(|ov| ov == v)) {
            if r.new_mod.starts_with('.') || r.new_mod.starts_with('/') { continue; }
            m = r.new_mod.clone();
            if !r.new_ver.is_empty() { v = r.new_ver.clone(); }
        }
        if !m.is_empty() && !v.is_empty() && seen.insert((m.clone(), v.clone())) {
            out.push(("Go", m, v));
        }
    }
}
