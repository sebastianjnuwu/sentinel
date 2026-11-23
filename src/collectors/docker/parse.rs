use std::{collections::HashSet, fs, path::Path};

pub(super) fn parse_tag(tag: &str) -> Option<(String, String)> {
    let (v, s) = match tag.find('-') { Some(i) => (&tag[..i], &tag[i..]), None => (tag, "") };
    crate::version::is_numeric_ver(v).then(|| (v.to_string(), s.to_string()))
}

fn normalize_repo(raw_repo: &str, raw_img: &str) -> Option<String> {
    let repo = raw_repo.strip_prefix("docker.io/").unwrap_or(raw_repo);
    let host = repo.split('/').next().unwrap_or("");
    if repo.contains('/') && (host.contains('.') || host.contains(':') || host == "localhost") {
        eprintln!("sentinel: skipping unsupported Docker registry image {raw_img}");
        return None;
    }
    Some(repo.to_string())
}

pub(super) fn extract_images(dir: &Path) -> Vec<(String, String)> {
    let (mut seen, mut out) = (HashSet::new(), Vec::new());
    for file in ["Dockerfile", "docker-compose.yml", "docker-compose.yaml"] {
        let Ok(text) = fs::read_to_string(dir.join(file)) else { continue };
        for line in text.lines() {
            let t = line.trim();
            let raw = if file == "Dockerfile" && t.to_ascii_uppercase().starts_with("FROM ") {
                t.split_whitespace().skip(1).find(|tok| !tok.starts_with("--"))
            } else {
                t.strip_prefix("image:").map(|r| r.trim().trim_matches(['"', '\'']))
            };
            let Some(img) = raw.map(|s| s.split('@').next().unwrap_or(s)) else { continue };
            if img.contains('$') || img == "scratch" { continue; }
            let Some((raw_repo, tag)) = img.split_once(':') else { continue };
            let Some(repo) = normalize_repo(raw_repo, img) else { continue };
            if tag == "latest" || parse_tag(tag).is_none() {
                eprintln!("sentinel: skipping non-semver Docker tag {img}");
            } else if seen.insert(format!("{repo}:{tag}")) {
                out.push((repo, tag.to_string()));
            }
        }
    }
    out
}
