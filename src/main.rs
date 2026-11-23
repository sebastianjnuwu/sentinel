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
mod update;
mod version;

use std::time::Instant;

fn main() {
    console::enable_utf8();
    update::cleanup_old_exe();
    let started = Instant::now();
    let opts = cli::parse();
    let cfg = config::load(&opts.root);
    cache::init(!opts.no_cache);
    let quiet = opts.json || opts.sarif;

    let projects = discover::discover_projects(&opts.root, opts.max_depth, &cfg.ignore_dirs);
    if projects.is_empty() {
        if !quiet { println!("🔍 No supported projects found in {}.", opts.root.display()); }
        return;
    }
    if !quiet {
        println!("🛡️  Sentinel: scanning {} project(s) in {}...", projects.len(), opts.root.display());
    }

    let upd_handle = (!quiet && cfg.check_updates).then(|| std::thread::spawn(update::fetch_latest_tag));
    let mut out = scan::run(&opts.root, &projects, &cfg);
    if !out.report.is_empty()
        && let Err(e) = osv::attach_cves(&mut out.report)
    {
        eprintln!("sentinel: {e}");
        out.osv_err = true;
    }
    let latest_tag = upd_handle.and_then(|h| h.join().ok().flatten());
    cache::flush();

    let has_cve = !out.lock_vulns.is_empty() || out.report.values().flatten().any(|p| !p.cves.is_empty());
    let has_major = out.report.values().flatten().any(|p| p.kind == "MAJOR");
    output::render(&opts, &out.report, &out.lock_vulns, started.elapsed().as_secs_f64());
    if !quiet { update::maybe_print_notice(latest_tag); }

    if out.osv_err || (opts.fail_on_cve && has_cve) || (opts.fail_on_major && has_major) || out.denied_lic {
        std::process::exit(1);
    }
}
