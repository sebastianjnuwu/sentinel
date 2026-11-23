pub mod gradle;
pub mod maven;
mod pom_ctx;

use crate::lockfiles::LockEntry;
use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::path::Path;

pub fn collect(dir: &Path) -> (Vec<OutdatedPkg>, Vec<LockEntry>) {
    let (mgr, deps) = if dir.join("pom.xml").exists() {
        ("maven", maven::parse_pom(dir))
    } else {
        ("gradle", gradle::parse_gradle(dir))
    };
    let direct = deps
        .iter()
        .map(|(g, a, v)| ("Maven", format!("{g}:{a}"), clean_ver(v)))
        .filter(|(_, _, v)| !v.is_empty())
        .collect();
    let out = crate::pool::map_bounded(&deps, |(g, a, v)| {
        let key = format!("maven:{g}:{a}");
        let latest = if let Some((t, _)) = crate::cache::get(&key) { t } else {
            let url = format!("https://search.maven.org/solrsearch/select?q=g:{g}+AND+a:{a}&rows=1&wt=json");
            let data: Value = crate::http::agent().get(&url).call().ok()?.into_body().read_json().ok()?;
            let l = data.get("response")?.get("docs")?.as_array()?.first()?.get("latestVersion")?.as_str()?.to_string();
            crate::cache::set(key, l.clone(), "-".into());
            l
        };
        let cur = clean_ver(v);
        is_newer(&cur, &latest).then(|| OutdatedPkg {
            manager: mgr.into(), name: format!("{g}:{a}"), kind: classify_update(&cur, &latest),
            current: cur, latest, license: "-".into(), cves: Vec::new(),
        })
    });
    (out, direct)
}
