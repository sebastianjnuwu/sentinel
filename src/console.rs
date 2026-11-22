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
