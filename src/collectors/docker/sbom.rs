use serde_json::{json, Value};
use std::collections::BTreeSet;

fn get_json(url: &str, tok: Option<&str>, accept: Option<&str>) -> Option<Value> {
    let mut req = crate::http::agent().get(url).header("User-Agent", crate::http::user_agent());
    if let Some(t) = tok { req = req.header("Authorization", &format!("Bearer {t}")); }
    if let Some(a) = accept { req = req.header("Accept", a); }
    req.call().ok()?.into_body().read_json().ok()
}

fn fetch_sbom_purls(hub: &str, tag: &str) -> Option<Vec<String>> {
    let tok_url = format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:{hub}:pull");
    let tok = get_json(&tok_url, None, None)?.get("token")?.as_str()?.to_string();
    let idx_url = format!("https://registry-1.docker.io/v2/{hub}/manifests/{tag}");
    let idx = get_json(&idx_url, Some(&tok), Some("application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json"))?;
    let att_dig = idx.get("manifests")?.as_array()?.iter().find(|m| {
        m.get("annotations").and_then(|a| a.get("vnd.docker.reference.type")).and_then(Value::as_str) == Some("attestation-manifest")
    })?.get("digest")?.as_str()?.to_string();
    let att_url = format!("https://registry-1.docker.io/v2/{hub}/manifests/{att_dig}");
    let att = get_json(&att_url, Some(&tok), Some("application/vnd.oci.image.manifest.v1+json"))?;
    let sbom_dig = att.get("layers")?.as_array()?.iter().find(|l| {
        l.get("annotations").and_then(|a| a.get("in-toto.io/predicate-type")).and_then(Value::as_str).is_some_and(|s| s.contains("spdx"))
    })?.get("digest")?.as_str()?.to_string();
    let blob_url = format!("https://registry-1.docker.io/v2/{hub}/blobs/{sbom_dig}");
    let sbom = get_json(&blob_url, Some(&tok), None)?;
    let mut purls = BTreeSet::new();
    for pkg in sbom.get("predicate")?.get("packages")?.as_array()? {
        for r in pkg.get("externalRefs").and_then(Value::as_array).into_iter().flatten() {
            if r.get("referenceType").and_then(Value::as_str) == Some("purl")
                && let Some(p) = r.get("referenceLocator").and_then(Value::as_str)
            {
                purls.insert(p.to_string());
            }
        }
    }
    Some(purls.into_iter().collect())
}

pub(super) fn image_cves(repo: &str, tag: &str) -> Vec<String> {
    let key = format!("docker-cves:{repo}:{tag}");
    if let Some((cached, _)) = crate::cache::get(&key) {
        return if cached.is_empty() { Vec::new() } else { cached.split(',').map(String::from).collect() };
    }
    let hub = if repo.contains('/') { repo.to_string() } else { format!("library/{repo}") };
    let Some(purls) = fetch_sbom_purls(&hub, tag) else {
        crate::cache::set(key, String::new(), String::new());
        return Vec::new();
    };
    let mut ids = BTreeSet::new();
    for chunk in purls.chunks(500) {
        let queries: Vec<Value> = chunk.iter().map(|p| json!({ "package": { "purl": p } })).collect();
        let Ok(data) = crate::osv::post_osv("https://api.osv.dev/v1/querybatch", &json!({ "queries": queries })) else { return Vec::new() };
        for res in data.get("results").and_then(Value::as_array).into_iter().flatten() {
            for v in res.get("vulns").and_then(Value::as_array).into_iter().flatten() {
                if let Some(id) = v.get("id").and_then(Value::as_str) { ids.insert(id.to_string()); }
            }
        }
    }
    let out: Vec<String> = ids.into_iter().collect();
    crate::cache::set(key, out.join(","), String::new());
    out
}
