use super::*;
use std::{collections::BTreeMap, fs};

#[test]
fn parses_npm_v1_yarn_berry_and_go_sum_replace() {
    let dir = std::env::temp_dir().join(format!("sentinel-locks-{}", std::process::id()));
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(dir.join("package-lock.json"), r#"{"lockfileVersion":1,"dependencies":{"lodash":{"version":"4.17.0","dependencies":{"sub":{"version":"1.0.0"}}}}}"#);
    let _ = fs::write(dir.join("yarn.lock"), "__metadata:\n  version: 8\n\"@scope/pkg@npm:^1.0.0\":\n  version: 1.2.3\n\"ws@workspace:.\":\n  resolution: \"ws@workspace:.\"\n  version: 0.0.0-use.local\n");
    let _ = fs::write(dir.join("go.mod"), "module t\ngo 1.16\nrequire github.com/gorilla/mux v1.8.0\nreplace github.com/a/b => ./local\nreplace github.com/c/d v1.0.0 => github.com/c/d v1.2.0\n");
    let _ = fs::write(dir.join("go.sum"), "github.com/gin-gonic/gin v1.7.0 h1:abc\ngithub.com/gin-gonic/gin v1.7.0/go.mod h1:abc\ngithub.com/a/b v0.1.0 h1:abc\ngithub.com/c/d v1.0.0 h1:abc\n");
    let entries = collect_lock_entries(&dir);
    let _ = fs::remove_dir_all(&dir);
    assert!(entries.contains(&("npm", "lodash".into(), "4.17.0".into())));
    assert!(entries.contains(&("npm", "sub".into(), "1.0.0".into())));
    assert!(entries.contains(&("npm", "@scope/pkg".into(), "1.2.3".into())));
    assert!(!entries.iter().any(|(_, n, _)| n == "ws"));
    assert!(entries.contains(&("Go", "github.com/gin-gonic/gin".into(), "1.7.0".into())));
    assert!(!entries.iter().any(|(_, n, _)| n == "github.com/a/b"));
    assert!(entries.contains(&("Go", "github.com/c/d".into(), "1.2.0".into())));
}

#[test]
fn resolve_ver_does_not_pick_arbitrary_mismatch() {
    let mut locked = BTreeMap::new();
    locked.insert("foo".into(), vec!["1.0.0".into(), "1.5.0".into()]);
    assert_eq!(resolve_ver(&locked, "foo", "2.0.0"), "2.0.0");
    assert_eq!(resolve_ver(&locked, "foo", "1.5"), "1.5.0");
}
