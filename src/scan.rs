use crate::config::SentinelConfig;
use crate::types::{OutdatedPkg, TransitiveVuln};
use crate::{collectors, lockfiles};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::thread;

pub type Report = BTreeMap<String, Vec<OutdatedPkg>>;
pub type LockVulns = BTreeMap<String, Vec<TransitiveVuln>>;

pub fn run(root: &Path, projects: &[PathBuf], cfg: &SentinelConfig) -> (Report, LockVulns, bool) {
    let (mut report, mut lock_inputs) = (Report::new(), Vec::new());
    thread::scope(|s| {
        let handles: Vec<_> = projects
            .iter()
            .map(|dir| {
                s.spawn(move || {
                    let (outdated, direct) = collectors::scan_project(root, dir);
                    let label = collectors::project_label(root, dir);
                    let mut entries = lockfiles::collect_lock_entries(dir);
                    entries.extend(direct);
                    (outdated, label, entries)
                })
            })
            .collect();
        for h in handles {
            let Ok((outdated, label, mut entries)) = h.join() else { continue };
            if let Some((name, mut items)) = outdated {
                items.retain(|p| !cfg.ignore_packages.iter().any(|ig| ig == &p.name));
                if !items.is_empty() { report.insert(name, items); }
            }
            entries.retain(|(_, n, _)| !cfg.ignore_packages.iter().any(|ig| ig == n));
            if !entries.is_empty() { lock_inputs.push((label, entries)); }
        }
    });
    match lockfiles::scan_batch(lock_inputs) {
        Ok(lock_vulns) => (report, lock_vulns, false),
        Err(e) => { eprintln!("sentinel: {e}"); (report, LockVulns::new(), true) }
    }
}
