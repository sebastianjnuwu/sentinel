use crate::osv::post_osv;
use serde_json::{json, Value};
use std::cmp::Ordering;

fn cmp_ver(a: &str, b: &str) -> Ordering {
    let parse = |s: &str| -> Vec<u64> {
        s.trim_start_matches('v').split(['-', '+']).next().unwrap_or("")
            .split('.').filter_map(|p| p.parse().ok()).collect()
    };
    let (va, vb) = (parse(a), parse(b));
    (0..va.len().max(vb.len()))
        .map(|i| va.get(i).copied().unwrap_or(0).cmp(&vb.get(i).copied().unwrap_or(0)))
        .find(|o| *o != Ordering::Equal)
        .unwrap_or(Ordering::Equal)
}

fn within_upper(cur: &str, cap: Option<&str>) -> bool {
    let Some(s) = cap else { return true };
    if let Some(v) = s.strip_prefix("<=") { return cmp_ver(cur, v.trim()) != Ordering::Greater; }
    s.strip_prefix('<').is_none_or(|v| cmp_ver(cur, v.trim()) == Ordering::Less)
}

fn range_matches(cur: &str, range: &Value, cap: Option<&str>) -> bool {
    let Some(events) = range.get("events").and_then(Value::as_array) else { return false };
    let (mut active, mut closed) = (false, false);
    for ev in events {
        if let Some(i) = ev.get("introduced").and_then(Value::as_str) {
            active = i == "0" || cmp_ver(cur, i) != Ordering::Less;
        } else if let Some(f) = ev.get("fixed").and_then(Value::as_str) {
            closed = true;
            if active && cmp_ver(cur, f) == Ordering::Less { return true; }
            active = false;
        } else if let Some(l) = ev.get("last_affected").and_then(Value::as_str) {
            closed = true;
            if active && cmp_ver(cur, l) != Ordering::Greater { return true; }
            active = false;
        }
    }
    active && (closed || within_upper(cur, cap))
}

fn affected_matches(repo: &str, cur: &str, aff: &Value) -> bool {
    if aff.get("package").and_then(|p| p.get("name")).and_then(Value::as_str) != Some(repo) { return false; }
    let norm = cur.trim_start_matches('v');
    if aff.get("versions").and_then(Value::as_array).is_some_and(|vs| vs.iter().filter_map(Value::as_str).any(|v| v.trim_start_matches('v') == norm)) {
        return true;
    }
    let cap = aff.get("database_specific").and_then(|d| d.get("last_known_affected_version_range")).and_then(Value::as_str);
    aff.get("ranges").and_then(Value::as_array).is_some_and(|rs| rs.iter().any(|r| range_matches(norm, r, cap)))
}

pub fn query_action(repo: &str, cur_tag: &str) -> Result<Vec<String>, String> {
    let key = format!("osv-gha:{repo}");
    let raw = if let Some((cached, _)) = crate::cache::get(&key) { cached } else {
        let data = post_osv("https://api.osv.dev/v1/query", &json!({ "package": { "name": repo, "ecosystem": "GitHub Actions" } }))?;
        let text = data.to_string();
        crate::cache::set(key, text.clone(), String::new());
        text
    };
    let data: Value = serde_json::from_str(&raw).map_err(|e| format!("OSV cache JSON: {e}"))?;
    Ok(data.get("vulns").and_then(Value::as_array).into_iter().flatten()
        .filter(|v| v.get("affected").and_then(Value::as_array).is_some_and(|a| a.iter().any(|af| affected_matches(repo, cur_tag, af))))
        .filter_map(|v| v.get("id").and_then(Value::as_str).map(String::from))
        .collect())
}
