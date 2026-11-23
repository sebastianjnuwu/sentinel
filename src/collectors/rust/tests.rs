use super::parse::parse_cargo_deps;
use std::collections::BTreeMap;

#[test]
fn parses_table_and_target_deps() {
    let toml = "[dependencies]\na = \"1.0.0\"\n[dependencies.serde]\nversion = \"1.0.1\"\n[target.'cfg(unix)'.dependencies]\nlibc = \"0.2.0\"\n";
    let deps = parse_cargo_deps(toml, &BTreeMap::new());
    assert!(deps.contains(&("a".into(), "1.0.0".into())));
    assert!(deps.contains(&("serde".into(), "1.0.1".into())));
    assert!(deps.contains(&("libc".into(), "0.2.0".into())));
}
