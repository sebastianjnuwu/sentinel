use std::{collections::BTreeMap, fs, path::Path};

fn map_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let idx = line.find(&format!("{key}:"))?;
    let rest = line[idx + key.len() + 1..].trim_start();
    let q = rest.chars().next().filter(|c| *c == '\'' || *c == '"')?;
    rest[1..].split(q).next()
}

fn parse_catalog(dir: &Path, out: &mut Vec<(String, String, String)>) {
    let Ok(text) = fs::read_to_string(dir.join("gradle").join("libs.versions.toml")) else { return };
    let (mut vers, mut libs, mut sec) = (BTreeMap::new(), Vec::new(), "");
    for line in text.lines() {
        let t = line.split('#').next().unwrap_or("").trim();
        if t.starts_with('[') { sec = t; continue; }
        let Some((k, v)) = t.split_once('=') else { continue };
        if sec == "[versions]" { vers.insert(k.trim().to_string(), v.trim().trim_matches(['"', '\'']).to_string()); }
        else if sec == "[libraries]" { libs.push(v.trim().to_string()); }
    }
    for v in libs {
        let ga = v.split("module").nth(1).and_then(|r| r.split(['"', '\'']).nth(1))
            .and_then(|s| s.split_once(':'));
        let ver = v.split("version.ref").nth(1).and_then(|r| r.split(['"', '\'']).nth(1)).and_then(|r| vers.get(r).cloned())
            .or_else(|| v.split("version").nth(1).and_then(|r| r.split(['"', '\'']).nth(1)).map(String::from));
        if let (Some((g, a)), Some(ver)) = (ga, ver) { out.push((g.into(), a.into(), ver)); }
    }
}

pub fn parse_gradle(dir: &Path) -> Vec<(String, String, String)> {
    let text = fs::read_to_string(dir.join("build.gradle.kts"))
        .or_else(|_| fs::read_to_string(dir.join("build.gradle"))).unwrap_or_default();
    let mut out = Vec::new();
    for line in text.lines() {
        if let (Some(g), Some(a), Some(v)) = (map_field(line, "group"), map_field(line, "name"), map_field(line, "version")) {
            out.push((g.to_string(), a.to_string(), v.to_string()));
        }
        for quote in ['"', '\''] {
            for part in line.split(quote).skip(1).step_by(2) {
                let segs: Vec<&str> = part.split(':').collect();
                if segs.len() == 3 && segs[2].chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    out.push((segs[0].into(), segs[1].into(), segs[2].into()));
                }
            }
        }
    }
    parse_catalog(dir, &mut out);
    out
}
