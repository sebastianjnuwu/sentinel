fn is_spdx_sep(c: char) -> bool {
    c.is_whitespace() || matches!(c, '(' | ')' | '/' | '+')
}

pub fn is_denied_license(license: &str, denied: &[String]) -> bool {
    if license == "-" || license.is_empty() || denied.is_empty() {
        return false;
    }
    let tokens: Vec<String> = license
        .split(is_spdx_sep)
        .filter(|s| !s.is_empty() && !matches!(*s, "OR" | "AND" | "WITH"))
        .map(|s| s.to_ascii_uppercase())
        .collect();
    denied.iter().any(|d| {
        let dn = d.trim().to_ascii_uppercase();
        !dn.is_empty()
            && tokens
                .iter()
                .any(|tok| tok == &dn || tok.starts_with(&format!("{dn}-")))
    })
}
