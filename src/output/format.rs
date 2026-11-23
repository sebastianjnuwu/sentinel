use crate::console::c;

pub fn safe(s: &str) -> String {
    s.chars().filter(|ch| !ch.is_control()).collect()
}

pub fn trunc(s: &str, max: usize) -> String {
    let clean = safe(s);
    if clean.chars().count() <= max {
        clean
    } else {
        format!("{}…", clean.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}

pub fn rule(widths: &[usize], l: &str, m: &str, r: &str) -> String {
    let segs: Vec<String> = widths.iter().map(|w| "─".repeat(*w + 2)).collect();
    format!("  {}{l}{}{r}{}", c("\x1b[90m"), segs.join(m), c("\x1b[0m"))
}

pub fn format_cve(cves: &[String]) -> (String, &'static str) {
    if cves.is_empty() {
        ("-".into(), c("\x1b[90m"))
    } else if cves.len() == 1 {
        (safe(&cves[0]), c("\x1b[1;31m"))
    } else {
        (format!("{} (+{})", safe(&cves[0]), cves.len() - 1), c("\x1b[1;31m"))
    }
}
