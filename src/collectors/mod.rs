pub mod dart;
pub mod docker;
pub mod dotnet;
pub mod elixir;
pub mod gha;
pub mod go;
pub mod java;
pub mod node;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
pub mod swift;
pub mod terraform;

use crate::discover::has_extension;
use crate::types::{LockEntry, OutdatedPkg, ProjectScan};
use std::path::Path;

pub fn project_label(root: &Path, dir: &Path) -> String {
    dir.strip_prefix(root)
        .ok()
        .map(|p| p.display().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| dir.file_name().and_then(|n| n.to_str()).unwrap_or(".").to_string())
}

fn push_pair(s: &mut ProjectScan, (o, d): (Vec<OutdatedPkg>, Vec<LockEntry>)) {
    s.outdated.extend(o);
    s.direct.extend(d);
}

fn push_scan(s: &mut ProjectScan, res: ProjectScan) {
    s.outdated.extend(res.outdated);
    s.direct.extend(res.direct);
    s.licenses.extend(res.licenses);
}

pub fn scan_project(dir: &Path) -> ProjectScan {
    let mut s = ProjectScan::default();
    if dir.join("package.json").exists() { push_scan(&mut s, node::collect(dir)); }
    if dir.join("go.mod").exists() { s.outdated.extend(go::collect(dir)); }
    if dir.join("Cargo.toml").exists() { push_scan(&mut s, rust::collect(dir)); }
    if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() { push_scan(&mut s, python::collect(dir)); }
    if dir.join("composer.json").exists() { s.outdated.extend(php::collect(dir)); }
    if has_extension(dir, "csproj") || has_extension(dir, "fsproj") { s.outdated.extend(dotnet::collect(dir)); }
    if dir.join("pom.xml").exists() || dir.join("build.gradle").exists() || dir.join("build.gradle.kts").exists() { push_pair(&mut s, java::collect(dir)); }
    if dir.join("Gemfile").exists() { push_scan(&mut s, ruby::collect(dir)); }
    if dir.join("pubspec.yaml").exists() { push_pair(&mut s, dart::collect(dir)); }
    if dir.join("mix.exs").exists() { push_scan(&mut s, elixir::collect(dir)); }
    if dir.join("Package.resolved").exists() { push_pair(&mut s, swift::collect(dir)); }
    if dir.join(".terraform.lock.hcl").exists() { s.outdated.extend(terraform::collect(dir)); }
    if dir.join("Dockerfile").exists() || dir.join("docker-compose.yml").exists() || dir.join("docker-compose.yaml").exists() { s.outdated.extend(docker::collect(dir)); }
    if dir.join(".github").join("workflows").exists() { push_pair(&mut s, gha::collect(dir)); }
    s.direct.retain(|(_, n, v)| !s.outdated.iter().any(|p| p.name == *n && p.current == *v));
    s.outdated.sort_by(|a, b| a.name.cmp(&b.name));
    s
}
