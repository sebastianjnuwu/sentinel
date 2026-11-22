pub fn clean_ver(v: &str) -> String {
    v.trim()
        .trim_start_matches('v')
        .trim_start_matches(|c: char| !c.is_ascii_digit())
        .split([' ', ','])
        .next()
        .unwrap_or("")
        .to_string()
}

pub fn parse_semver_tuple(s: &str) -> (u64, u64, u64) {
    let core = s.split('-').next().unwrap_or(s);
    let mut parts = core.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

pub fn is_numeric_ver(v: &str) -> bool {
    !v.is_empty() && v.chars().all(|c| c.is_ascii_digit() || c == '.')
}

pub fn classify_update(current: &str, latest: &str) -> String {
    let (c_maj, c_min, c_pat) = parse_semver_tuple(current);
    let (l_maj, l_min, l_pat) = parse_semver_tuple(latest);
    if l_maj > c_maj {
        "MAJOR".into()
    } else if l_min > c_min {
        "MINOR".into()
    } else if l_pat > c_pat {
        "PATCH".into()
    } else {
        "UPDATE".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_semver_prefixes() {
        assert_eq!(clean_ver("^1.2.3"), "1.2.3");
        assert_eq!(clean_ver("v2.0.1"), "2.0.1");
        assert_eq!(clean_ver(">=3.4.5, <4.0.0"), "3.4.5");
    }

    #[test]
    fn classifies_updates() {
        assert_eq!(classify_update("1.2.3", "2.0.0"), "MAJOR");
        assert_eq!(classify_update("1.2.3", "1.3.0"), "MINOR");
        assert_eq!(classify_update("1.2.3", "1.2.4"), "PATCH");
    }
}
