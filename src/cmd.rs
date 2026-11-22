use std::path::Path;
use std::process::Command;

pub fn run(program: &str, args: &[&str], dir: &Path) -> String {
    #[cfg(target_os = "windows")]
    let output = Command::new("cmd")
        .arg("/C")
        .arg(program)
        .args(args)
        .current_dir(dir)
        .output();

    #[cfg(not(target_os = "windows"))]
    let output = Command::new(program).args(args).current_dir(dir).output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_string(),
        Err(_) => String::new(),
    }
}
