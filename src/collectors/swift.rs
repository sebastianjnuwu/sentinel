use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::{env, fs, path::Path};

fn extract_pin(pin: &Value) -> Option<(String, String, String)> {
    let loc = pin.get("location").or_else(|| pin.get("repositoryURL")).and_then(Value::as_str)?;
    let ver = pin.get("state").and_then(|s| s.get("version")).and_then(Value::as_str)?;
    let gh = loc.strip_prefix("https://github.com/")?;
    let repo = gh.trim_end_matches(".git").to_string();
    let name = pin.get("identity").or_else(|| pin.get("package")).and_then(Value::as_str).unwrap_or(&repo).to_string();
    Some((name, repo, clean_ver(ver)))
}

pub fn parse_swift_resolved(dir: &Path) -> Vec<(String, String, String)> {
    let Ok(text) = fs::read_to_string(dir.join("Package.resolved")) else { return Vec::new() };
    let Ok(data) = serde_json::from_str::<Value>(&text) else { return Vec::new() };
    let pins = data.get("pins").or_else(|| data.get("object").and_then(|o| o.get("pins"))).and_then(Value::as_array);
    pins.map(|arr| arr.iter().filter_map(extract_pin).collect()).unwrap_or_default()
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let pins = parse_swift_resolved(dir);
    let token = env::var("GITHUB_TOKEN").ok();
    crate::pool::map_bounded(&pins, |(name, repo, cur)| {
        let key = format!("swift:{repo}");
        let latest = if let Some((t, _)) = crate::cache::get(&key) { t } else {
            let mut req = crate::http::agent().get(&format!("https://api.github.com/repos/{repo}/releases/latest"))
                .header("User-Agent", crate::http::user_agent());
            if let Some(t) = token.as_deref() { req = req.header("Authorization", &format!("Bearer {t}")); }
            let data: Value = req.call().ok()?.into_body().read_json().ok()?;
            let lat = clean_ver(data.get("tag_name")?.as_str()?);
            crate::cache::set(key, lat.clone(), "-".into());
            lat
        };
        is_newer(cur, &latest).then(|| OutdatedPkg {
            manager: "swiftpm".into(), name: name.clone(), kind: classify_update(cur, &latest),
            current: cur.clone(), latest, license: "-".into(), cves: Vec::new(),
        })
    })
}
