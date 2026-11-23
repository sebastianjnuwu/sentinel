use std::{fs, path::Path, process::Command};

fn sha256_file(path: &Path) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    let out = Command::new("powershell")
        .args(["-NoProfile", "-Command", "(Get-FileHash -LiteralPath $env:F -Algorithm SHA256).Hash"])
        .env("F", path).output().map_err(|e| format!("sha256 failed: {e}"))?;
    #[cfg(not(target_os = "windows"))]
    let out = Command::new("sh")
        .args(["-c", "sha256sum \"$F\" 2>/dev/null || shasum -a 256 \"$F\""])
        .env("F", path).output().map_err(|e| format!("sha256 failed: {e}"))?;
    if !out.status.success() { return Err("sha256 command failed".into()); }
    let hex = String::from_utf8_lossy(&out.stdout).split_whitespace().next().unwrap_or("").to_ascii_lowercase();
    if hex.len() != 64 { return Err("invalid sha256 output".into()); }
    Ok(hex)
}

pub(super) fn verify_checksum(tag: &str, asset: &str, file: &Path) -> Result<(), String> {
    let url = format!("https://github.com/sebastianjnuwu/sentinel/releases/download/{tag}/checksums.txt");
    let text = crate::http::agent().get(&url).header("User-Agent", crate::http::user_agent())
        .call().map_err(|e| format!("checksums download failed: {e}"))?
        .into_body().read_to_string().map_err(|e| format!("checksums read failed: {e}"))?;
    let expected = text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let (sum, name) = (parts.next()?, parts.next()?);
        (name.trim_start_matches('*') == asset).then(|| sum.to_ascii_lowercase())
    }).ok_or_else(|| format!("checksum entry for '{asset}' not found"))?;
    let actual = sha256_file(file)?;
    if actual != expected {
        return Err(format!("checksum mismatch for {asset}: expected {expected}, got {actual}"));
    }
    let _ = fs::metadata(file);
    Ok(())
}
