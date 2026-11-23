mod common;

use common::TempDir;
use sentinel_scan::{cache, discover, osv, pool, version};

#[test]
fn cleans_compares_and_classifies_semver() {
    assert_eq!(version::clean_ver("^1.2.3"), "1.2.3");
    assert_eq!(version::clean_ver("v2.0.1"), "2.0.1");
    assert_eq!(version::parse_semver_tuple("1.2.3-alpha+1"), (1, 2, 3));
    assert!(version::is_numeric_ver("1.2.3"));
    assert!(!version::is_numeric_ver("latest"));
    assert_eq!(version::classify_update("1.2.3", "2.0.0"), "MAJOR");
    assert_eq!(version::classify_update("0.26.1", "0.36.1"), "MAJOR");
    assert_eq!(version::classify_update("1.2.3", "1.3.0"), "MINOR");
    assert_eq!(version::classify_update("1.2.3", "1.2.4"), "PATCH");
    assert!(version::is_newer("1.2.3", "1.2.4"));
    assert!(!version::is_newer("3.54.1.0", "3.49.1.0"));
}

#[test]
fn bounded_pool_maps_empty_and_parallel_items() {
    let empty: Vec<i32> = Vec::new();
    assert!(pool::map_bounded(&empty, |x| Some(*x)).is_empty());
    let nums: Vec<usize> = (0..40).collect();
    let mut out = pool::map_bounded(&nums, |&x| (x % 2 == 0).then_some(x * 2));
    out.sort_unstable();
    assert_eq!(out.len(), 20);
    assert_eq!((out[0], out[19]), (0, 76));
}

#[test]
fn discovers_projects_and_respects_skip_dirs() {
    let tmp = TempDir::new("disc");
    tmp.write("a/Cargo.lock", "");
    tmp.write("node_modules/b/package.json", "{}");
    tmp.write("skipme/go.mod", "");
    let found = discover::discover_projects(tmp.path(), 2, &["skipme".into()]);
    assert_eq!(found, vec![tmp.path().join("a")]);
}

#[test]
fn maps_osv_ecosystems_and_memory_cache() {
    for (m, e) in [("npm", "npm"), ("pnpm", "npm"), ("go", "Go"), ("cargo", "crates.io"), ("pip", "PyPI"), ("uv", "PyPI"), ("composer", "Packagist"), ("dotnet", "NuGet"), ("maven", "Maven"), ("bundler", "RubyGems"), ("pub", "Pub"), ("hex", "Hex"), ("actions", "GitHub Actions")] {
        assert_eq!(osv::osv_ecosystem(m), Some(e));
    }
    assert_eq!(osv::osv_ecosystem("unknown"), None);
    cache::init(true);
    cache::set("test:k".into(), "1.0.0".into(), "MIT".into());
    assert_eq!(cache::get("test:k"), Some(("1.0.0".into(), "MIT".into())));
}
