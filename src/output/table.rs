use crate::console::c;
use crate::output::format::{format_cve, rule, safe, trunc};
use crate::output::icons::{manager_icon, severity_style, update_cmd};
use crate::types::OutdatedPkg;
use std::collections::BTreeMap;

pub fn print_table(report: &BTreeMap<String, Vec<OutdatedPkg>>, extra_cves: usize, elapsed_secs: f64) {
    let (mut maj, mut min, mut pat, mut cves) = (0usize, 0usize, 0usize, extra_cves);
    let (dim, bold, cyan, red, yel, grn, rst) = (
        c("\x1b[90m"), c("\x1b[1m"), c("\x1b[1;36m"), c("\x1b[1;31m"), c("\x1b[1;33m"), c("\x1b[1;32m"), c("\x1b[0m"),
    );
    for (proj, items) in report {
        let rows: Vec<_> = items.iter().map(|p| {
            let (cve_s, cve_c) = format_cve(&p.cves);
            (p, trunc(&p.name, 22), format!("{} → {}", p.current, p.latest), trunc(&p.license, 12), cve_s, cve_c)
        }).collect();
        let wp = rows.iter().map(|r| r.1.chars().count()).max().unwrap_or(7).max(7);
        let wm = items.iter().map(|p| p.manager.len()).max().unwrap_or(7).max(7);
        let wv = rows.iter().map(|r| r.2.chars().count()).max().unwrap_or(7).max(7);
        let wlic = rows.iter().map(|r| r.3.chars().count()).max().unwrap_or(7).max(7);
        let wcve = rows.iter().map(|r| r.4.len()).max().unwrap_or(3).max(3);
        let widths = [wp, wm + 3, wv, 9, wlic, wcve];
        println!("\n📦 {cyan}{}{rst} {dim}({} outdated){rst}", safe(proj), items.len());
        println!("{}", rule(&widths, "┌", "┬", "┐"));
        println!("  {dim}│{rst} {bold}{:<wp$}{rst} {dim}│{rst} {bold}{:<w_m$}{rst} {dim}│{rst} {bold}{:<wv$}{rst} {dim}│{rst} {bold}{:<9}{rst} {dim}│{rst} {bold}{:<wlic$}{rst} {dim}│{rst} {bold}{:<wcve$}{rst} {dim}│{rst}", "PACKAGE", "MANAGER", "VERSION", "TYPE", "LICENSE", "CVE", w_m = wm + 3);
        println!("{}", rule(&widths, "├", "┼", "┤"));
        let mut mgrs: Vec<(String, bool)> = Vec::new();
        for (p, pkg, ver, lic, cve_s, cve_c) in rows {
            let is_maj = p.kind == "MAJOR";
            if let Some(m) = mgrs.iter_mut().find(|(n, _)| n == &p.manager) { m.1 |= is_maj; } else { mgrs.push((p.manager.clone(), is_maj)); }
            match p.kind.as_str() { "MAJOR" => maj += 1, "MINOR" => min += 1, "PATCH" => pat += 1, _ => {} }
            cves += p.cves.len();
            let (icon, col) = severity_style(&p.kind);
            let lic_c = if p.license.contains("GPL") { yel } else { dim };
            println!("  {dim}│{rst} {:<wp$} {dim}│{rst} {} {dim}{:<wm$}{rst} {dim}│{rst} {bold}{:<wv$}{rst} {dim}│{rst} {icon} {col}{:<6}{rst} {dim}│{rst} {lic_c}{:<wlic$}{rst} {dim}│{rst} {cve_c}{:<wcve$}{rst} {dim}│{rst}", pkg, manager_icon(&p.manager), safe(&p.manager), safe(&ver), p.kind, lic, cve_s);
        }
        println!("{}", rule(&widths, "└", "┴", "┘"));
        for (m, hm) in mgrs {
            let cmd = update_cmd(&m, hm);
            if !cmd.is_empty() { println!("  ⚡ {dim}update ({m}):{rst} {}{cmd}{rst}", c("\x1b[36m")); }
        }
    }
    let badge = if cves > 0 { format!(" · ⚠️  {red}{cves} CVE(s){rst}") } else { format!(" · 🛡️  {}{cves} CVEs{rst}", c("\x1b[32m")) };
    println!("\n📊 {bold}Summary:{rst} 🔴 {red}{maj} major{rst} · 🟡 {yel}{min} minor{rst} · 🟢 {grn}{pat} patch{rst}{badge} {dim}({elapsed_secs:.2}s){rst}\n");
}
