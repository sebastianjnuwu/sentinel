use crate::config::SentinelConfig;
use crate::types::{OutdatedPkg, TransitiveVuln};
use crate::{collectors, lockfiles};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::thread;

pub type Report = BTreeMap<String, Vec<OutdatedPkg>>;
pub type LockVulns = BTreeMap<String, Vec<TransitiveVuln>>;

pub fn run(root: &Path, projects: &[PathBuf], cfg: &SentinelConfig) -> (Report, LockVulns) {
    let (mut report, mut lock_vulns) = (Report::new(), LockVulns::new());
    thread::scope(|s| {
        let handles: Vec<_> = projects
            .iter()
            .map(|dir| {
                s.spawn(move || {
                    (
                        collectors::scan_project(root, dir),
                        collectors::project_label(root, dir),
                        lockfiles::scan_lockfile_vulns(dir),
                    )
                })
            })
            .collect();
        for h in handles {
            let Ok((outdated, label, vulns)) = h.join() else { continue };
            if let Some((name, mut items)) = outdated {
                items.retain(|p| !cfg.ignore_packages.iter().any(|ig| ig == &p.name));
                if !items.is_empty() {
                    report.insert(name, items);
                }
            }
            if !vulns.is_empty() {
                lock_vulns.insert(label, vulns);
            }
        }
    });
    (report, lock_vulns)
}
