pub mod format;
pub mod icons;
pub mod sarif;
pub mod table;
pub mod vulns;

use crate::cli::CliOpts;
use crate::scan::{LockVulns, Report};

pub fn render(opts: &CliOpts, report: &Report, lock_vulns: &LockVulns, elapsed: f64) {
    if opts.sarif {
        let root_lbl = crate::collectors::project_label(&opts.root, &opts.root);
        sarif::print_sarif(&root_lbl, report, lock_vulns);
    } else if opts.json {
        let out = serde_json::json!({
            "outdated": report,
            "lockfile_vulnerabilities": lock_vulns,
            "elapsed_secs": elapsed,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
    } else if report.is_empty() && lock_vulns.is_empty() {
        println!("✅ All packages are up to date and no lockfile vulnerabilities found.");
    } else {
        let lock_cve_count: usize = lock_vulns.values().flatten().map(|v| v.ids.len()).sum();
        if !report.is_empty() || lock_cve_count > 0 {
            table::print_table(report, lock_cve_count, elapsed);
        }
        if !lock_vulns.is_empty() {
            vulns::print_lockfile_vulns(lock_vulns);
        }
    }
}
