mod cache;
mod cli;
mod cmd;
mod collectors;
mod config;
mod console;
mod discover;
mod http;
mod lockfiles;
mod osv;
mod output;
mod pool;
mod scan;
mod types;
mod version;

use std::time::Instant;

fn main() {
    console::enable_utf8();
    let started = Instant::now();
    let opts = cli::parse();
    let cfg = config::load(&opts.root);
    cache::init(!opts.no_cache);
    let quiet = opts.json || opts.sarif;

    let projects = discover::discover_projects(&opts.root, opts.max_depth, &cfg.ignore_dirs);
    if projects.is_empty() {
        if !quiet {
            println!("🔍 No supported projects found in {}.", opts.root.display());
        }
        return;
    }
    if !quiet {
        println!("🛡️  Sentinel: scanning {} project(s) in {}...", projects.len(), opts.root.display());
    }

    let (mut report, lock_vulns) = scan::run(&opts.root, &projects, &cfg);
    if !report.is_empty() {
        osv::attach_cves(&mut report);
    }
    cache::flush();

    let has_cve = !lock_vulns.is_empty() || report.values().flatten().any(|p| !p.cves.is_empty());
    let has_major = report.values().flatten().any(|p| p.kind == "MAJOR");
    let has_denied_lic = report.values().flatten().any(|p| cfg.deny_licenses.iter().any(|d| p.license.contains(d)));
    output::render(&opts, &report, &lock_vulns, started.elapsed().as_secs_f64());

    if (opts.fail_on_cve && has_cve) || (opts.fail_on_major && has_major) || has_denied_lic {
        std::process::exit(1);
    }
}
