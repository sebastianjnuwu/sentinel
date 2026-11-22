use crate::output::icons::{manager_icon, severity_style, update_cmd};
use crate::types::{OutdatedPkg, TransitiveVuln};
use std::collections::BTreeMap;

fn trunc(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max.saturating_sub(1)).collect::<String>())
    }
}

fn rule(widths: &[usize], l: &str, m: &str, r: &str) -> String {
    let segs: Vec<String> = widths.iter().map(|w| "─".repeat(*w + 2)).collect();
    format!("  \x1b[90m{l}{}{r}\x1b[0m", segs.join(m))
}

fn format_cve(cves: &[String]) -> (String, &'static str) {
    if cves.is_empty() {
        ("-".into(), "\x1b[90m")
    } else if cves.len() == 1 {
        (cves[0].clone(), "\x1b[1;31m")
    } else {
        (format!("{} (+{})", cves[0], cves.len() - 1), "\x1b[1;31m")
    }
}

pub fn print_table(report: &BTreeMap<String, Vec<OutdatedPkg>>, elapsed_secs: f64) {
    let (mut maj, mut min, mut pat, mut cves) = (0usize, 0usize, 0usize, 0usize);
    for (proj, items) in report {
        let rows: Vec<_> = items.iter().map(|p| {
            let pkg = trunc(&p.name, 22);
            let ver = format!("{} → {}", p.current, p.latest);
            let lic = trunc(&p.license, 12);
            let (cve_s, cve_c) = format_cve(&p.cves);
            (p, pkg, ver, lic, cve_s, cve_c)
        }).collect();
        let wp = rows.iter().map(|r| r.1.chars().count()).max().unwrap_or(7).max(7);
        let wm = items.iter().map(|p| p.manager.len()).max().unwrap_or(7).max(7);
        let wv = rows.iter().map(|r| r.2.chars().count()).max().unwrap_or(7).max(7);
        let wlic = rows.iter().map(|r| r.3.chars().count()).max().unwrap_or(7).max(7);
        let wcve = rows.iter().map(|r| r.4.len()).max().unwrap_or(3).max(3);
        let widths = [wp, wm + 3, wv, 9, wlic, wcve];

        println!("\n📦 \x1b[1;36m{proj}\x1b[0m \x1b[90m({} outdated)\x1b[0m", items.len());
        println!("{}", rule(&widths, "┌", "┬", "┐"));
        println!("  \x1b[90m│\x1b[0m \x1b[1m{:<wp$}\x1b[0m \x1b[90m│\x1b[0m \x1b[1m{:<w_m$}\x1b[0m \x1b[90m│\x1b[0m \x1b[1m{:<wv$}\x1b[0m \x1b[90m│\x1b[0m \x1b[1m{:<9}\x1b[0m \x1b[90m│\x1b[0m \x1b[1m{:<wlic$}\x1b[0m \x1b[90m│\x1b[0m \x1b[1m{:<wcve$}\x1b[0m \x1b[90m│\x1b[0m", "PACKAGE", "MANAGER", "VERSION", "TYPE", "LICENSE", "CVE", w_m = wm + 3);
        println!("{}", rule(&widths, "├", "┼", "┤"));

        let mut mgrs: Vec<(String, bool)> = Vec::new();
        for (p, pkg, ver, lic, cve_s, cve_c) in rows {
            let is_maj = p.kind == "MAJOR";
            if let Some(m) = mgrs.iter_mut().find(|(n, _)| n == &p.manager) { m.1 |= is_maj; } else { mgrs.push((p.manager.clone(), is_maj)); }
            match p.kind.as_str() { "MAJOR" => maj += 1, "MINOR" => min += 1, "PATCH" => pat += 1, _ => {} }
            cves += p.cves.len();
            let (icon, col) = severity_style(&p.kind);
            let lic_c = if p.license.contains("GPL") { "\x1b[1;33m" } else { "\x1b[90m" };
            println!("  \x1b[90m│\x1b[0m {:<wp$} \x1b[90m│\x1b[0m {} \x1b[90m{:<wm$}\x1b[0m \x1b[90m│\x1b[0m \x1b[1m{:<wv$}\x1b[0m \x1b[90m│\x1b[0m {icon} {col}{:<6}\x1b[0m \x1b[90m│\x1b[0m {lic_c}{:<wlic$}\x1b[0m \x1b[90m│\x1b[0m {cve_c}{:<wcve$}\x1b[0m \x1b[90m│\x1b[0m", pkg, manager_icon(&p.manager), p.manager, ver, p.kind, lic, cve_s);
        }
        println!("{}", rule(&widths, "└", "┴", "┘"));
        for (m, hm) in mgrs {
            let cmd = update_cmd(&m, hm);
            if !cmd.is_empty() { println!("  ⚡ \x1b[90mupdate ({m}):\x1b[0m \x1b[36m{cmd}\x1b[0m"); }
        }
    }
    let badge = if cves > 0 { format!(" · ⚠️  \x1b[1;31m{cves} CVE(s)\x1b[0m") } else { " · 🛡️  \x1b[32m0 CVEs\x1b[0m".into() };
    println!("\n📊 \x1b[1mSummary:\x1b[0m 🔴 \x1b[1;31m{maj} major\x1b[0m · 🟡 \x1b[1;33m{min} minor\x1b[0m · 🟢 \x1b[1;32m{pat} patch\x1b[0m{badge} \x1b[90m({elapsed_secs:.2}s)\x1b[0m\n");
}

pub fn print_lockfile_vulns(vulns: &BTreeMap<String, Vec<TransitiveVuln>>) {
    println!("🔒 \x1b[1;31mLockfile / Transitive Vulnerabilities (OSV):\x1b[0m");
    for (proj, list) in vulns {
        println!("  📦 \x1b[1;36m{proj}\x1b[0m \x1b[90m({} vulnerable package(s))\x1b[0m", list.len());
        for v in list { println!("    ⚠️  \x1b[1m{}@{}\x1b[0m \x1b[90m[{}]\x1b[0m → \x1b[1;31m{}\x1b[0m", v.package, v.version, v.ecosystem, v.ids.join(", ")); }
    }
    println!();
}
