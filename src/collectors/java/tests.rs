use super::{gradle::parse_gradle, maven::parse_pom};
use std::fs;

#[test]
fn parses_maven_props_bom_and_gradle_map() {
    let dir = std::env::temp_dir().join(format!("sentinel-java-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let pom = "<project><properties><c.ver>3.12.0</c.ver></properties><dependencyManagement><dependencies><dependency><groupId>g</groupId><artifactId>b</artifactId><version>2.0.0</version></dependency></dependencies></dependencyManagement><dependencies><dependency><groupId>g</groupId><artifactId>a</artifactId><version>${c.ver}</version></dependency><dependency><groupId>g</groupId><artifactId>b</artifactId></dependency></dependencies></project>";
    let gradle = "dependencies {\n  implementation group: 'org.apache.commons', name: 'commons-lang3', version: '3.12.0'\n}\n";
    let _ = fs::write(dir.join("pom.xml"), pom);
    let _ = fs::write(dir.join("build.gradle"), gradle);
    let mp = parse_pom(&dir);
    let gp = parse_gradle(&dir);
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(mp, vec![("g".into(), "a".into(), "3.12.0".into()), ("g".into(), "b".into(), "2.0.0".into())]);
    assert_eq!(gp, vec![("org.apache.commons".into(), "commons-lang3".into(), "3.12.0".into())]);
}
