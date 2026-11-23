use std::{collections::BTreeMap, fs, path::Path};

pub(super) fn tag_val<'a>(block: &'a str, tag: &str) -> Option<&'a str> {
    let open = format!("<{tag}>");
    let start = block.find(&open)? + open.len();
    let end = block[start..].find(&format!("</{tag}>"))?;
    Some(block[start..start + end].trim())
}

fn collect_props(text: &str, props: &mut BTreeMap<String, String>) {
    if let Some(blk) = tag_val(text, "properties") {
        for chunk in blk.split('<').skip(1) {
            if chunk.starts_with('/') || chunk.starts_with('!') { continue; }
            if let Some((tag, rest)) = chunk.split_once('>')
                && let Some(val) = rest.split('<').next()
            {
                props.entry(tag.trim().into()).or_insert_with(|| val.trim().into());
            }
        }
    }
}

pub(super) fn load_ctx(
    path: &Path,
    props: &mut BTreeMap<String, String>,
    mgmt: &mut BTreeMap<(String, String), String>,
    depth: u8,
) -> Option<(String, String)> {
    let text = fs::read_to_string(path).ok()?;
    collect_props(&text, props);
    if let Some(dm) = tag_val(&text, "dependencyManagement") {
        for blk in dm.split("<dependency>").skip(1).filter_map(|c| c.split("</dependency>").next()) {
            if let (Some(g), Some(a), Some(v)) = (tag_val(blk, "groupId"), tag_val(blk, "artifactId"), tag_val(blk, "version")) {
                mgmt.entry((g.into(), a.into())).or_insert_with(|| v.into());
            }
        }
    }
    let parent_gv = tag_val(&text, "parent").and_then(|p| {
        if depth < 2 {
            let rel = tag_val(p, "relativePath").unwrap_or("../pom.xml");
            if let Some(pp) = path.parent().map(|d| d.join(rel)).filter(|p| p.is_file()) {
                load_ctx(&pp, props, mgmt, depth + 1);
            }
        }
        Some((tag_val(p, "groupId")?.to_string(), tag_val(p, "version")?.to_string()))
    });
    if let Some(pv) = tag_val(&text, "version") { props.entry("project.version".into()).or_insert_with(|| pv.into()); }
    parent_gv
}
