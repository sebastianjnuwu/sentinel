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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_exact_spdx_without_false_positives() {
        let deny = vec!["GPL".to_string(), "MIT".to_string()];
        assert!(is_denied_license("GPL-3.0-only", &deny));
        assert!(is_denied_license("MIT OR Apache-2.0", &deny));
        assert!(!is_denied_license("LGPL-2.1", &["GPL".into()]));
        assert!(!is_denied_license("AGPL-3.0", &["GPL".into()]));
        assert!(!is_denied_license("Apache-2.0", &deny));
    }
}
