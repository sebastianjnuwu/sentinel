#![allow(dead_code)]

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new(tag: &str) -> Self {
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!("sentinel-{tag}-{}-{id}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        let _ = std::fs::create_dir_all(&p);
        Self(p)
    }
    pub fn write(&self, rel: &str, content: &str) {
        let p = self.0.join(rel);
        if let Some(parent) = p.parent() { let _ = std::fs::create_dir_all(parent); }
        std::fs::write(p, content).unwrap();
    }
    pub fn s(&self) -> &str { self.0.to_str().unwrap() }
    pub fn path(&self) -> &Path { &self.0 }
}

impl Drop for TempDir {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
}

pub fn sentinel_bin() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_BIN_EXE_sentinel"));
    if p.exists() { p } else {
        std::env::current_dir().unwrap().join("target").join("debug")
            .join(if cfg!(windows) { "sentinel.exe" } else { "sentinel" })
    }
}

pub fn run(args: &[&str]) -> Output {
    Command::new(sentinel_bin()).args(args).output().unwrap()
}

pub fn json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout).unwrap()
}
