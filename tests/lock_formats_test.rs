mod common;

use common::TempDir;
use sentinel_scan::lockfiles::{collect_lock_entries, resolve_ver};
use std::collections::BTreeMap;

#[test]
fn parses_npm_v1_yarn_berry_go_sum_and_cargo_sources() {
    let tmp = TempDir::new("locks-unit");
    tmp.write("package-lock.json", r#"{"lockfileVersion":1,"dependencies":{"lodash":{"version":"4.17.0","dependencies":{"sub":{"version":"1.0.0"}}}}}"#);
    tmp.write("yarn.lock", "__metadata:\n  version: 8\n\"@scope/pkg@npm:^1.0.0\":\n  version: 1.2.3\n\"ws@workspace:.\":\n  resolution: \"ws@workspace:.\"\n  version: 0.0.0-use.local\n");
    tmp.write("go.mod", "module t\ngo 1.16\nrequire github.com/gorilla/mux v1.8.0\nreplace github.com/a/b => ./local\nreplace github.com/c/d v1.0.0 => github.com/c/d v1.2.0\n");
    tmp.write("go.sum", "github.com/gin-gonic/gin v1.7.0 h1:abc\ngithub.com/gin-gonic/gin v1.7.0/go.mod h1:abc\ngithub.com/a/b v0.1.0 h1:abc\ngithub.com/c/d v1.0.0 h1:abc\n");
    tmp.write("Cargo.lock", "[[package]]\nname = \"serde\"\nversion = \"1.0.200\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n[[package]]\nname = \"time\"\nversion = \"0.1.44\"\nsource = \"git+https://github.com/example/time#abc\"\n[[package]]\nname = \"local\"\nversion = \"0.1.0\"\n");
    let e = collect_lock_entries(tmp.path());
    assert!(e.contains(&("npm", "lodash".into(), "4.17.0".into())));
    assert!(e.contains(&("npm", "sub".into(), "1.0.0".into())));
    assert!(e.contains(&("npm", "@scope/pkg".into(), "1.2.3".into())));
    assert!(!e.iter().any(|(_, n, _)| n == "ws"));
    assert!(e.contains(&("Go", "github.com/gin-gonic/gin".into(), "1.7.0".into())));
    assert!(!e.iter().any(|(_, n, _)| n == "github.com/a/b"));
    assert!(e.contains(&("Go", "github.com/c/d".into(), "1.2.0".into())));
    assert!(e.contains(&("crates.io", "serde".into(), "1.0.200".into())));
    assert!(!e.iter().any(|(_, n, _)| n == "time" || n == "local"));
}

#[test]
fn parses_pnpm_poetry_nuget_composer_gem_pub_mix_and_resolves_ver() {
    let tmp = TempDir::new("locks-extra");
    tmp.write("pnpm-lock.yaml", "packages:\n  /axios@1.6.0:\n    resolution: {}\n");
    tmp.write("poetry.lock", "[[package]]\nname = \"flask\"\nversion = \"3.0.0\"\n");
    tmp.write("packages.lock.json", r#"{"dependencies":{"net8.0":{"Newtonsoft.Json":{"resolved":"13.0.3"}}}}"#);
    tmp.write("composer.lock", r#"{"packages":[{"name":"monolog/monolog","version":"v3.5.0"}]}"#);
    tmp.write("Gemfile.lock", "GEM\n  specs:\n    rails (7.1.0)\n");
    tmp.write("pubspec.lock", "packages:\n  http:\n    version: \"1.2.0\"\n");
    tmp.write("mix.lock", "%{\n  \"jason\": {:hex, :jason, \"1.4.1\", \"abc\"},\n}\n");
    let e = collect_lock_entries(tmp.path());
    assert!(e.contains(&("npm", "axios".into(), "1.6.0".into())));
    assert!(e.contains(&("PyPI", "flask".into(), "3.0.0".into())));
    assert!(e.contains(&("NuGet", "Newtonsoft.Json".into(), "13.0.3".into())));
    assert!(e.contains(&("Packagist", "monolog/monolog".into(), "3.5.0".into())));
    assert!(e.contains(&("RubyGems", "rails".into(), "7.1.0".into())));
    assert!(e.contains(&("Pub", "http".into(), "1.2.0".into())));
    assert!(e.contains(&("Hex", "jason".into(), "1.4.1".into())));

    let mut locked = BTreeMap::new();
    locked.insert("foo".into(), vec!["1.0.0".into(), "1.5.0".into()]);
    assert_eq!(resolve_ver(&locked, "foo", "2.0.0"), "2.0.0");
    assert_eq!(resolve_ver(&locked, "foo", "1.5"), "1.5.0");
}
