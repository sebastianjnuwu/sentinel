mod cache;
mod cli;
mod cmd;
mod collectors;
mod config;
mod discover;
mod http;
mod lockfiles;
mod osv;
mod output;
mod pool;
mod types;
mod version;

use std::collections::BTreeMap;
use std::thread;
use std::time::Instant;

#[cfg(target_os = "windows")]
fn enable_utf8_console() {
    unsafe extern "system" { fn SetConsoleOutputCP(wCodePageID: u32) -> i32; }
    unsafe { SetConsoleOutputCP(65001); }
}
#[cfg(not(target_os = "windows"))]
fn enable_utf8_console() {}

fn main() {
    enable_utf8_console();
    let started = Instant::now();
    let opts = cli::parse();
    let cfg = config::load(&opts.root);
    cache::init(!opts.no_cache);
    let quiet = opts.json || opts.sarif;

    let projects = discover::discover_projects(&opts.root, opts.max_depth, &cfg.ignore_dirs);
    if projects.is_empty() { if !quiet { println!("🔍 No supported projects found in {}.", opts.root.display()); } return; }
    if !quiet { println!("🛡️  Sentinel: scanning {} project(s) in {}...", projects.len(), opts.root.display()); }

    let (mut report, mut lock_vulns) = (BTreeMap::new(), BTreeMap::new());
    thread::scope(|s| {
        let handles: Vec<_> = projects.iter().map(|dir| {
            let root_ref = &opts.root;
            s.spawn(move || (collectors::scan_project(root_ref, dir), collectors::project_label(root_ref, dir), lockfiles::scan_lockfile_vulns(dir)))
        }).collect();
        for h in handles {
            if let Ok((outdated, label, vulns)) = h.join() {
                if let Some((n, mut items)) = outdated {
                    items.retain(|p| !cfg.ignore_packages.iter().any(|ig| ig == &p.name));
                    if !items.is_empty() { report.insert(n, items); }
                }
                if !vulns.is_empty() { lock_vulns.insert(label, vulns); }
            }
        }
    });

    if !report.is_empty() { osv::attach_cves(&mut report); }
    cache::flush();
    let has_cve = !lock_vulns.is_empty() || report.values().flatten().any(|p| !p.cves.is_empty());
    let has_major = report.values().flatten().any(|p| p.kind == "MAJOR");
    let has_denied_lic = report.values().flatten().any(|p| cfg.deny_licenses.iter().any(|d| p.license.contains(d)));
    if opts.sarif {
        output::sarif::print_sarif(&report, &lock_vulns);
    } else if opts.json {
        let out = serde_json::json!({ "outdated": report, "lockfile_vulnerabilities": lock_vulns, "elapsed_secs": started.elapsed().as_secs_f64() });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else if report.is_empty() && lock_vulns.is_empty() {
        println!("✅ All packages are up to date and no lockfile vulnerabilities found.");
    } else {
        if !report.is_empty() { output::table::print_table(&report, started.elapsed().as_secs_f64()); }
        if !lock_vulns.is_empty() { output::table::print_lockfile_vulns(&lock_vulns); }
    }

    if (opts.fail_on_cve && has_cve) || (opts.fail_on_major && has_major) || has_denied_lic {
        std::process::exit(1);
    }
}
