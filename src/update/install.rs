use std::{env, fs, io, path::Path, process::Command};

fn asset_name() -> Option<&'static str> {
    match (env::consts::OS, env::consts::ARCH) {
        ("windows", "x86_64") => Some("sentinel-windows-amd64.zip"),
        ("windows", "aarch64") => Some("sentinel-windows-arm64.zip"),
        ("windows", "x86") => Some("sentinel-windows-386.zip"),
        ("linux", "x86_64") => Some("sentinel-linux-amd64.tar.gz"),
        ("linux", "aarch64") => Some("sentinel-linux-arm64.tar.gz"),
        ("linux", "x86") => Some("sentinel-linux-386.tar.gz"),
        ("linux", "arm") => Some("sentinel-linux-armv7.tar.gz"),
        ("macos", "aarch64") => Some("sentinel-macos-arm64.tar.gz"),
        ("macos", "x86_64") => Some("sentinel-macos-amd64.tar.gz"),
        _ => None,
    }
}

fn extract_archive(archive: &Path, dest: &Path, is_zip: bool) -> Result<(), String> {
    let ok = if is_zip {
        Command::new("powershell").args(["-NoProfile", "-Command", &format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", archive.display(), dest.display())]).status()
    } else {
        Command::new("tar").args(["-xzf", &archive.to_string_lossy(), "-C", &dest.to_string_lossy()]).status()
    }.map_err(|e| format!("extract failed: {e}"))?.success();
    ok.then_some(()).ok_or_else(|| "archive extraction failed".into())
}

fn replace_exe(new_bin: &Path, exe: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")] {
        let old = exe.with_file_name("sentinel.old.exe");
        let _ = fs::remove_file(&old);
        fs::rename(exe, &old).map_err(|e| format!("rename running exe failed: {e}"))?;
        if let Err(e) = fs::copy(new_bin, exe) {
            let _ = fs::rename(&old, exe);
            return Err(format!("copy new exe failed: {e}"));
        }
    }
    #[cfg(not(target_os = "windows"))] {
        let staged = exe.with_file_name("sentinel.new");
        fs::copy(new_bin, &staged).map_err(|e| format!("copy failed: {e}"))?;
        fs::rename(&staged, exe).map_err(|e| format!("replace failed: {e}"))?;
    }
    Ok(())
}

pub(super) fn install_from_release(tag: &str, lat: &str) -> Result<(), String> {
    let name = asset_name().ok_or_else(|| format!("unsupported platform {}/{}", env::consts::OS, env::consts::ARCH))?;
    let url = format!("https://github.com/sebastianjnuwu/sentinel/releases/download/{tag}/{name}");
    let dir = env::temp_dir().join(format!("sentinel-upd-{lat}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).map_err(|e| format!("temp dir failed: {e}"))?;
    let archive = dir.join(name);
    let mut resp = crate::http::agent().get(&url).header("User-Agent", crate::http::user_agent()).call().map_err(|e| format!("download failed: {e}"))?;
    let mut file = fs::File::create(&archive).map_err(|e| format!("create archive failed: {e}"))?;
    io::copy(&mut resp.body_mut().as_reader(), &mut file).map_err(|e| format!("write archive failed: {e}"))?;
    extract_archive(&archive, &dir, name.ends_with(".zip"))?;
    let exe = env::current_exe().map_err(|e| format!("current_exe failed: {e}"))?;
    let res = replace_exe(&dir.join(if cfg!(windows) { "sentinel.exe" } else { "sentinel" }), &exe);
    let _ = fs::remove_dir_all(&dir);
    res
}
