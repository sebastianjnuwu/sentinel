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
use crate::lockfiles::LockEntry;
use crate::types::OutdatedPkg;
use std::path::Path;

pub fn project_label(root: &Path, dir: &Path) -> String {
    dir.strip_prefix(root)
        .ok()
        .map(|p| p.display().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| dir.file_name().and_then(|n| n.to_str()).unwrap_or(".").to_string())
}

fn push_pair(items: &mut Vec<OutdatedPkg>, direct: &mut Vec<LockEntry>, res: (Vec<OutdatedPkg>, Vec<LockEntry>)) {
    items.extend(res.0);
    direct.extend(res.1);
}

pub fn scan_project(root: &Path, dir: &Path) -> (Option<(String, Vec<OutdatedPkg>)>, Vec<LockEntry>) {
    let (mut items, mut direct) = (Vec::new(), Vec::new());
    if dir.join("package.json").exists() { push_pair(&mut items, &mut direct, node::collect(dir)); }
    if dir.join("go.mod").exists() { items.extend(go::collect(dir)); }
    if dir.join("Cargo.toml").exists() { push_pair(&mut items, &mut direct, rust::collect(dir)); }
    if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() { push_pair(&mut items, &mut direct, python::collect(dir)); }
    if dir.join("composer.json").exists() { items.extend(php::collect(dir)); }
    if has_extension(dir, "csproj") || has_extension(dir, "fsproj") { items.extend(dotnet::collect(dir)); }
    if dir.join("pom.xml").exists() || dir.join("build.gradle").exists() || dir.join("build.gradle.kts").exists() { push_pair(&mut items, &mut direct, java::collect(dir)); }
    if dir.join("Gemfile").exists() { push_pair(&mut items, &mut direct, ruby::collect(dir)); }
    if dir.join("pubspec.yaml").exists() { push_pair(&mut items, &mut direct, dart::collect(dir)); }
    if dir.join("mix.exs").exists() { push_pair(&mut items, &mut direct, elixir::collect(dir)); }
    if dir.join("Package.resolved").exists() { items.extend(swift::collect(dir)); }
    if dir.join(".terraform.lock.hcl").exists() { items.extend(terraform::collect(dir)); }
    if dir.join("Dockerfile").exists() || dir.join("docker-compose.yml").exists() || dir.join("docker-compose.yaml").exists() { items.extend(docker::collect(dir)); }
    if dir.join(".github").join("workflows").exists() { push_pair(&mut items, &mut direct, gha::collect(dir)); }
    direct.retain(|(_, n, v)| !items.iter().any(|p| p.name == *n && p.current == *v));
    if items.is_empty() { return (None, direct); }
    items.sort_by(|a, b| a.name.cmp(&b.name));
    (Some((project_label(root, dir), items)), direct)
}
