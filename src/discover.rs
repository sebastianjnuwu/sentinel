use std::fs;
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &[
    "node_modules", ".git", "target", "dist", "build", ".next", ".nuxt",
    "vendor", ".venv", "venv", "__pycache__", ".agents", "backup", "bin", "obj",
];
const MARKERS: &[&str] = &[
    "package.json", "go.mod", "Cargo.toml", "pyproject.toml", "requirements.txt",
    "composer.json", "pom.xml", "build.gradle", "build.gradle.kts", "Gemfile",
    "Gemfile.lock", "pubspec.yaml", "pubspec.lock", "mix.exs", "mix.lock",
    "Dockerfile", "docker-compose.yml", "docker-compose.yaml",
    "Package.resolved", ".terraform.lock.hcl",
];

pub fn has_extension(dir: &Path, ext: &str) -> bool {
    let Ok(entries) = fs::read_dir(dir) else { return false };
    entries.flatten().any(|e| e.path().extension().and_then(|s| s.to_str()) == Some(ext))
}

fn is_project_dir(dir: &Path) -> bool {
    MARKERS.iter().any(|m| dir.join(m).exists())
        || dir.join(".github").join("workflows").exists()
        || has_extension(dir, "csproj")
        || has_extension(dir, "fsproj")
}

pub fn discover_projects(root: &Path, max_depth: usize, extra_skip: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, 0, max_depth, extra_skip, &mut out);
    out
}

fn walk(current: &Path, depth: usize, max_depth: usize, extra_skip: &[String], out: &mut Vec<PathBuf>) {
    if is_project_dir(current) { out.push(current.to_path_buf()); }
    if depth >= max_depth { return; }
    let Ok(entries) = fs::read_dir(current) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if SKIP_DIRS.contains(&name) || name.starts_with('.') || extra_skip.iter().any(|s| s == name) { continue; }
        walk(&path, depth + 1, max_depth, extra_skip, out);
    }
}
