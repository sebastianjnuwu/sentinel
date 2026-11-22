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
    let core = s.split(['-', '+']).next().unwrap_or(s);
    let mut parts = core.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    (parts.next().unwrap_or(0), parts.next().unwrap_or(0), parts.next().unwrap_or(0))
}

pub fn is_numeric_ver(v: &str) -> bool {
    !v.is_empty() && v.chars().all(|c| c.is_ascii_digit() || c == '.')
}

pub fn is_newer(current: &str, latest: &str) -> bool {
    if current.is_empty() || latest.is_empty() || current == latest { return false; }
    let c: Vec<u64> = current.split(['-', '+']).next().unwrap_or(current).split('.').filter_map(|p| p.parse().ok()).collect();
    let l: Vec<u64> = latest.split(['-', '+']).next().unwrap_or(latest).split('.').filter_map(|p| p.parse().ok()).collect();
    if c.is_empty() || l.is_empty() { return current != latest; }
    for i in 0..c.len().max(l.len()) {
        let (cv, lv) = (*c.get(i).unwrap_or(&0), *l.get(i).unwrap_or(&0));
        if lv != cv { return lv > cv; }
    }
    false
}

pub fn classify_update(current: &str, latest: &str) -> String {
    let (c_maj, c_min, c_pat) = parse_semver_tuple(current);
    let (l_maj, l_min, l_pat) = parse_semver_tuple(latest);
    if l_maj > c_maj || (c_maj == 0 && l_maj == 0 && l_min > c_min) {
        "MAJOR".into()
    } else if l_min > c_min || (c_maj == 0 && l_maj == 0 && l_pat > c_pat) {
        "MINOR".into()
    } else if l_pat > c_pat || is_newer(current, latest) {
        "PATCH".into()
    } else {
        "UPDATE".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_and_classifies() {
        assert_eq!(clean_ver("^1.2.3"), "1.2.3");
        assert_eq!(clean_ver("v2.0.1"), "2.0.1");
        assert_eq!(classify_update("1.2.3", "2.0.0"), "MAJOR");
        assert_eq!(classify_update("0.26.1", "0.36.1"), "MAJOR");
        assert_eq!(classify_update("1.2.3", "1.3.0"), "MINOR");
        assert_eq!(classify_update("1.2.3", "1.2.4"), "PATCH");
        assert!(is_newer("1.2.3", "1.2.4"));
        assert!(!is_newer("3.54.1.0", "3.49.1.0"));
        assert!(!is_newer("20260814", "20250517"));
    }
}
