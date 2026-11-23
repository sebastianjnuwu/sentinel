mod install;
mod verify;

use crate::version::{clean_ver, is_newer};
use std::{io::{self, Write}, time::Duration};
use ureq::Agent;

const LATEST_URL: &str = "https://github.com/sebastianjnuwu/sentinel/releases/latest";

pub fn cleanup_old_exe() {
    #[cfg(target_os = "windows")]
    if let Ok(exe) = std::env::current_exe() {
        let old = exe.with_file_name("sentinel.old.exe");
        if old.exists() { let _ = std::fs::remove_file(old); }
    }
}

fn resolve_latest_tag() -> Result<String, String> {
    let agent: Agent = Agent::config_builder().max_redirects(0).timeout_global(Some(Duration::from_secs(10))).build().into();
    let resp = agent.get(LATEST_URL).header("User-Agent", crate::http::user_agent()).call()
        .map_err(|e| format!("failed to check releases: {e}"))?;
    let loc = resp.headers().get("location").and_then(|v| v.to_str().ok()).ok_or("no releases found yet")?;
    loc.rsplit('/').next().filter(|t| !t.is_empty() && *t != "releases").map(String::from).ok_or_else(|| "no releases found yet".into())
}

pub fn fetch_latest_tag() -> Option<String> {
    let key = "self:latest";
    if let Some((tag, _)) = crate::cache::get(key) { return (!tag.is_empty()).then_some(tag); }
    let tag = resolve_latest_tag().ok()?;
    crate::cache::set(key.into(), tag.clone(), String::new());
    Some(tag)
}

pub fn maybe_print_notice(latest: Option<String>) {
    let Some(lat_tag) = latest else { return };
    let (cur, lat) = (env!("CARGO_PKG_VERSION"), clean_ver(&lat_tag));
    if is_newer(cur, &lat) {
        let (yel, cyan, rst) = (crate::console::c("\x1b[1;33m"), crate::console::c("\x1b[36m"), crate::console::c("\x1b[0m"));
        println!("⬆️  {yel}New Sentinel version available:{rst} v{cur} → v{lat} {cyan}(run 'sentinel -u' to update){rst}");
    }
}

fn confirm(cur: &str, lat: &str) -> bool {
    print!("Update Sentinel v{cur} → v{lat}? [y/N]: ");
    let _ = io::stdout().flush();
    let mut ans = String::new();
    io::stdin().read_line(&mut ans).is_ok() && matches!(ans.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

pub fn run_self_update() -> Result<(), String> {
    let tag = resolve_latest_tag()?;
    let (cur, lat) = (env!("CARGO_PKG_VERSION"), clean_ver(&tag));
    if !is_newer(cur, &lat) { println!("✅ Sentinel is already up to date (v{cur})."); return Ok(()); }
    if !confirm(cur, &lat) { println!("Update cancelled."); return Ok(()); }
    println!("⬇️  Updating Sentinel v{cur} → v{lat}...");
    install::install_from_release(&tag, &lat)?;
    println!("✅ Sentinel updated to v{lat}.");
    Ok(())
}
