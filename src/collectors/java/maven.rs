use super::pom_ctx::{load_ctx, tag_val};
use std::{collections::BTreeMap, fs, path::Path};

fn resolve_prop(val: &str, props: &BTreeMap<String, String>) -> String {
    let mut s = val.to_string();
    for _ in 0..3 {
        let Some(a) = s.find("${") else { break };
        let Some(b) = s[a + 2..].find('}') else { break };
        let key = &s[a + 2..a + 2 + b];
        let Some(rep) = props.get(key) else { break };
        s = format!("{}{}{}", &s[..a], rep, &s[a + 3 + b..]);
    }
    s
}

pub fn parse_pom(dir: &Path) -> Vec<(String, String, String)> {
    let pom = dir.join("pom.xml");
    let Ok(text) = fs::read_to_string(&pom) else { return Vec::new() };
    let (mut props, mut mgmt) = (BTreeMap::new(), BTreeMap::new());
    let parent_gv = load_ctx(&pom, &mut props, &mut mgmt, 0);
    let stripped = match (text.find("<dependencyManagement>"), text.find("</dependencyManagement>")) {
        (Some(a), Some(b)) if a < b => format!("{}{}", &text[..a], &text[b + 23..]),
        _ => text,
    };
    let mut out = Vec::new();
    for blk in stripped.split("<dependency>").skip(1).filter_map(|c| c.split("</dependency>").next()) {
        let (Some(g), Some(a)) = (tag_val(blk, "groupId"), tag_val(blk, "artifactId")) else { continue };
        let raw_v = tag_val(blk, "version").map(str::to_string)
            .or_else(|| mgmt.get(&(g.into(), a.into())).cloned())
            .or_else(|| parent_gv.as_ref().filter(|(pg, _)| pg == g).map(|(_, pv)| pv.clone()));
        let resolved = raw_v.map(|v| resolve_prop(&v, &props)).unwrap_or_default();
        if resolved.is_empty() || resolved.contains("${") {
            eprintln!("sentinel: maven skipped {g}:{a} (unresolved version)");
        } else {
            out.push((g.to_string(), a.to_string(), resolved));
        }
    }
    out
}
