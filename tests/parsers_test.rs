mod common;

use common::TempDir;
use sentinel_scan::collectors::{docker, go, java, python, rust, swift, terraform};
use std::collections::BTreeMap;

#[test]
fn parses_cargo_table_and_target_sections() {
    let toml = "[dependencies]\na = \"1.0.0\"\n[dependencies.serde]\nversion = \"1.0.1\"\n[target.'cfg(unix)'.dependencies]\nlibc = \"0.2.0\"\n";
    let deps = rust::parse::parse_cargo_deps(toml, &BTreeMap::new());
    assert!(deps.contains(&("a".into(), "1.0.0".into())));
    assert!(deps.contains(&("serde".into(), "1.0.1".into())));
    assert!(deps.contains(&("libc".into(), "0.2.0".into())));
}

#[test]
fn parses_maven_props_bom_and_gradle_map_catalog() {
    let tmp = TempDir::new("java-unit");
    tmp.write("pom.xml", "<project><properties><c.ver>3.12.0</c.ver></properties><dependencyManagement><dependencies><dependency><groupId>g</groupId><artifactId>b</artifactId><version>2.0.0</version></dependency></dependencies></dependencyManagement><dependencies><dependency><groupId>g</groupId><artifactId>a</artifactId><version>${c.ver}</version></dependency><dependency><groupId>g</groupId><artifactId>b</artifactId></dependency></dependencies></project>");
    tmp.write("build.gradle", "dependencies {\n  implementation group: 'org.apache.commons', name: 'commons-lang3', version: '3.12.0'\n}\n");
    tmp.write("gradle/libs.versions.toml", "[versions]\ngson = \"2.10.1\"\n[libraries]\ngson = { module = \"com.google.code.gson:gson\", version.ref = \"gson\" }\n");
    assert_eq!(java::maven::parse_pom(tmp.path()), vec![("g".into(), "a".into(), "3.12.0".into()), ("g".into(), "b".into(), "2.0.0".into())]);
    assert_eq!(java::gradle::parse_gradle(tmp.path()), vec![("org.apache.commons".into(), "commons-lang3".into(), "3.12.0".into()), ("com.google.code.gson".into(), "gson".into(), "2.10.1".into())]);
}

#[test]
fn parses_python_go_swift_terraform_and_docker() {
    let tmp = TempDir::new("misc-unit");
    tmp.write("requirements.txt", "# comment\n-r base.txt\nfastapi[all]>=0.110.0; python_version>'3.8'\n");
    tmp.write("pyproject.toml", "[tool.poetry.dependencies]\npython = \"^3.11\"\npydantic = \"^2.5.0\"\n");
    tmp.write("go.mod", "module t\nrequire example.com/a v2.0.0\nreplace example.com/a v1.0.0 => ./local\n");
    tmp.write("Package.resolved", r#"{"pins":[{"identity":"nio","location":"https://github.com/apple/swift-nio.git","state":{"version":"2.1.0"}}]}"#);
    tmp.write(".terraform.lock.hcl", "provider \"registry.terraform.io/hashicorp/aws\" {\n  version = \"5.1.0\"\n}\n");
    tmp.write("Dockerfile", "FROM --platform=linux/amd64 docker.io/redis:7.2-alpine@sha256:abc\nFROM scratch\n");
    assert_eq!(python::parse::parse_python_deps(tmp.path(), &BTreeMap::new()), vec![("fastapi".into(), "0.110.0".into()), ("pydantic".into(), "2.5.0".into())]);
    assert_eq!(go::parse_go_direct(tmp.path()), vec![("example.com/a".into(), "2.0.0".into())]);
    assert_eq!(swift::parse_swift_resolved(tmp.path()), vec![("nio".into(), "github.com/apple/swift-nio".into(), "2.1.0".into())]);
    assert_eq!(terraform::parse_tf_lock(tmp.path()), vec![("hashicorp/aws".into(), "5.1.0".into())]);
    assert_eq!(docker::parse::parse_tag("16.2-alpine"), Some(("16.2".into(), "-alpine".into())));
    assert_eq!(docker::parse::extract_images(tmp.path()), vec![("redis".into(), "7.2-alpine".into())]);
}
