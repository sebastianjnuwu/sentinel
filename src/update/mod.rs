mod install;

use crate::version::{clean_ver, is_newer};
use serde_json::Value;

const RELEASE_URL: &str = "https://api.github.com/repos/sebastianjnuwu/sentinel/releases/latest";

fn fetch_release_json() -> Result<Value, String> {
    let mut resp = crate::http::agent()
        .get(RELEASE_URL)
        .header("User-Agent", crate::http::user_agent())
        .call()
        .map_err(|e| format!("failed to check releases: {e}"))?;
    resp.body_mut().read_json::<Value>().map_err(|e| format!("invalid release JSON: {e}"))
}

pub fn fetch_latest_tag() -> Option<String> {
    let key = "self:latest";
    if let Some((tag, _)) = crate::cache::get(key) {
        return (!tag.is_empty()).then_some(tag);
    }
    let tag = fetch_release_json()
        .ok()
        .and_then(|d| d.get("tag_name").and_then(Value::as_str).map(String::from))?;
    crate::cache::set(key.into(), tag.clone(), String::new());
    Some(tag)
}

pub fn maybe_print_notice(latest: Option<String>) {
    let Some(lat_tag) = latest else { return };
    let cur = env!("CARGO_PKG_VERSION");
    let lat = clean_ver(&lat_tag);
    if is_newer(cur, &lat) {
        let (yel, cyan, rst) = (crate::console::c("\x1b[1;33m"), crate::console::c("\x1b[36m"), crate::console::c("\x1b[0m"));
        println!("⬆️  {yel}New Sentinel version available:{rst} v{cur} → v{lat} {cyan}(run 'sentinel -u' to update){rst}");
    }
}

pub fn run_self_update() -> Result<(), String> {
    let data = fetch_release_json()?;
    let tag = data.get("tag_name").and_then(Value::as_str).ok_or("release missing tag_name")?;
    let cur = env!("CARGO_PKG_VERSION");
    let lat = clean_ver(tag);
    if !is_newer(cur, &lat) {
        println!("✅ Sentinel is already up to date (v{cur}).");
        return Ok(());
    }
    println!("⬇️  Updating Sentinel v{cur} → v{lat}...");
    install::install_from_release(&data, &lat)?;
    println!("✅ Sentinel updated to v{lat}.");
    Ok(())
}
