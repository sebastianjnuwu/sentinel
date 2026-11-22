pub fn trunc(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}

pub fn rule(widths: &[usize], l: &str, m: &str, r: &str) -> String {
    let segs: Vec<String> = widths.iter().map(|w| "─".repeat(*w + 2)).collect();
    format!("  \x1b[90m{l}{}{r}\x1b[0m", segs.join(m))
}

pub fn format_cve(cves: &[String]) -> (String, &'static str) {
    if cves.is_empty() {
        ("-".into(), "\x1b[90m")
    } else if cves.len() == 1 {
        (cves[0].clone(), "\x1b[1;31m")
    } else {
        (format!("{} (+{})", cves[0], cves.len() - 1), "\x1b[1;31m")
    }
}
