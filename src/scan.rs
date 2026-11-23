use crate::config::SentinelConfig;
use crate::types::{OutdatedPkg, TransitiveVuln};
use crate::{collectors, lockfiles};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::thread;

pub type Report = BTreeMap<String, Vec<OutdatedPkg>>;
pub type LockVulns = BTreeMap<String, Vec<TransitiveVuln>>;

pub fn run(root: &Path, projects: &[PathBuf], cfg: &SentinelConfig) -> (Report, LockVulns, bool) {
    let (mut report, mut lock_vulns, mut osv_err) = (Report::new(), LockVulns::new(), false);
    thread::scope(|s| {
        let handles: Vec<_> = projects
            .iter()
            .map(|dir| {
                s.spawn(move || {
                    let (outdated, direct) = collectors::scan_project(root, dir);
                    let label = collectors::project_label(root, dir);
                    let vulns = lockfiles::scan_lockfile_vulns(dir, &direct);
                    (outdated, label, vulns)
                })
            })
            .collect();
        for h in handles {
            let Ok((outdated, label, vulns_res)) = h.join() else { continue };
            if let Some((name, mut items)) = outdated {
                items.retain(|p| !cfg.ignore_packages.iter().any(|ig| ig == &p.name));
                if !items.is_empty() { report.insert(name, items); }
            }
            match vulns_res {
                Ok(mut vulns) => {
                    vulns.retain(|v| !cfg.ignore_packages.iter().any(|ig| ig == &v.package));
                    if !vulns.is_empty() { lock_vulns.insert(label, vulns); }
                }
                Err(e) => { eprintln!("sentinel: {e}"); osv_err = true; }
            }
        }
    });
    (report, lock_vulns, osv_err)
}
