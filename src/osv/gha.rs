use crate::osv::post_osv;
use serde_json::{json, Value};
use std::cmp::Ordering;

fn cmp_ver(a: &str, b: &str) -> Ordering {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v').split(['-', '+']).next().unwrap_or("")
            .split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let (va, vb) = (parse(a), parse(b));
    for i in 0..va.len().max(vb.len()) {
        match va.get(i).copied().unwrap_or(0).cmp(&vb.get(i).copied().unwrap_or(0)) {
            Ordering::Equal => continue,
            ord => return ord,
        }
    }
    Ordering::Equal
}

fn range_matches(cur: &str, range: &Value) -> bool {
    let Some(events) = range.get("events").and_then(Value::as_array) else { return false };
    let mut active = false;
    for ev in events {
        if let Some(intro) = ev.get("introduced").and_then(Value::as_str) {
            active = intro == "0" || cmp_ver(cur, intro) != Ordering::Less;
        } else if let Some(fixed) = ev.get("fixed").and_then(Value::as_str) {
            if active && cmp_ver(cur, fixed) == Ordering::Less { return true; }
            active = false;
        } else if let Some(last) = ev.get("last_affected").and_then(Value::as_str) {
            if active && cmp_ver(cur, last) != Ordering::Greater { return true; }
            active = false;
        }
    }
    active
}

fn affected_matches(repo: &str, cur: &str, aff: &Value) -> bool {
    let pkg_name = aff.get("package").and_then(|p| p.get("name")).and_then(Value::as_str);
    if pkg_name != Some(repo) { return false; }
    let norm = cur.trim_start_matches('v');
    if let Some(vers) = aff.get("versions").and_then(Value::as_array)
        && vers.iter().filter_map(Value::as_str).any(|v| v.trim_start_matches('v') == norm)
    {
        return true;
    }
    aff.get("ranges").and_then(Value::as_array)
        .is_some_and(|rs| rs.iter().any(|r| range_matches(norm, r)))
}

pub fn query_action(repo: &str, cur_tag: &str) -> Result<Vec<String>, String> {
    let key = format!("osv-gha:{repo}");
    let raw = if let Some((cached, _)) = crate::cache::get(&key) {
        cached
    } else {
        let data = post_osv("https://api.osv.dev/v1/query", &json!({ "package": { "name": repo, "ecosystem": "GitHub Actions" } }))?;
        let text = data.to_string();
        crate::cache::set(key, text.clone(), String::new());
        text
    };
    let data: Value = serde_json::from_str(&raw).map_err(|e| format!("OSV cache JSON: {e}"))?;
    let mut ids = Vec::new();
    for v in data.get("vulns").and_then(Value::as_array).into_iter().flatten() {
        let Some(id) = v.get("id").and_then(Value::as_str) else { continue };
        if v.get("affected").and_then(Value::as_array).is_some_and(|a| a.iter().any(|af| affected_matches(repo, cur_tag, af))) {
            ids.push(id.to_string());
        }
    }
    Ok(ids)
}
