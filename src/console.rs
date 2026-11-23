use std::io::IsTerminal;
use std::sync::OnceLock;

static COLOR: OnceLock<bool> = OnceLock::new();

#[cfg(target_os = "windows")]
pub fn enable_utf8() {
    unsafe extern "system" {
        fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
    }
    unsafe {
        SetConsoleOutputCP(65001);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn enable_utf8() {}

pub fn use_color() -> bool {
    *COLOR.get_or_init(|| {
        std::io::stdout().is_terminal()
            && std::env::var_os("NO_COLOR").is_none()
            && std::env::var("TERM").map(|t| t != "dumb").unwrap_or(true)
    })
}

pub fn c(code: &'static str) -> &'static str {
    if use_color() { code } else { "" }
}
