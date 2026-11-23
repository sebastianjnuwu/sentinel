use crate::types::OutdatedPkg;
use crate::version::{classify_update, clean_ver, is_newer};
use serde_json::Value;
use std::{fs, path::Path};

pub fn parse_tf_lock(dir: &Path) -> Vec<(String, String)> {
    let Ok(text) = fs::read_to_string(dir.join(".terraform.lock.hcl")) else { return Vec::new() };
    let (mut out, mut provider) = (Vec::new(), String::new());
    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("provider \"") {
            provider = rest.split('"').next().unwrap_or("").trim_start_matches("registry.terraform.io/").to_string();
        } else if let Some(rest) = t.strip_prefix("version")
            && let Some(v) = rest.split('"').nth(1)
        {
            let cur = clean_ver(v);
            if !provider.is_empty() && !cur.is_empty() {
                out.push((provider.clone(), cur));
                provider.clear();
            }
        }
    }
    out
}

pub fn collect(dir: &Path) -> Vec<OutdatedPkg> {
    let providers = parse_tf_lock(dir);
    crate::pool::map_bounded(&providers, |(prov, cur)| {
        let key = format!("tf:{prov}");
        let latest = if let Some((t, _)) = crate::cache::get(&key) { t } else {
            let data: Value = crate::http::agent().get(&format!("https://registry.terraform.io/v1/providers/{prov}/versions"))
                .call().ok()?.into_body().read_json().ok()?;
            let lat = data.get("versions")?.as_array()?.last()?.get("version")?.as_str()?.to_string();
            crate::cache::set(key, lat.clone(), "-".into());
            lat
        };
        is_newer(cur, &latest).then(|| OutdatedPkg {
            manager: "terraform".into(), name: prov.clone(), kind: classify_update(cur, &latest),
            current: cur.clone(), latest, license: "-".into(), cves: Vec::new(),
        })
    })
}
