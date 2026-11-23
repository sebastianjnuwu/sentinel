use crate::config::{SentinelConfig, is_denied_license};
use crate::types::{OutdatedPkg, TransitiveVuln};
use crate::{collectors, lockfiles};
use std::{collections::BTreeMap, path::{Path, PathBuf}, thread};

pub type Report = BTreeMap<String, Vec<OutdatedPkg>>;
pub type LockVulns = BTreeMap<String, Vec<TransitiveVuln>>;

pub struct ScanOutcome {
    pub report: Report,
    pub lock_vulns: LockVulns,
    pub osv_err: bool,
    pub denied_lic: bool,
}

pub fn run(root: &Path, projects: &[PathBuf], cfg: &SentinelConfig) -> ScanOutcome {
    let (mut report, mut lock_inputs, mut denied_lic) = (Report::new(), Vec::new(), false);
    thread::scope(|s| {
        let handles: Vec<_> = projects.iter().map(|dir| {
            s.spawn(move || (collectors::project_label(root, dir), collectors::scan_project(dir), lockfiles::collect_lock_entries(dir)))
        }).collect();
        for h in handles {
            let Ok((label, mut scan, mut entries)) = h.join() else { continue };
            scan.outdated.retain(|p| !cfg.ignore_packages.iter().any(|ig| ig == &p.name));
            for (pkg, lic) in scan.licenses {
                if !cfg.ignore_packages.iter().any(|ig| ig == &pkg) && is_denied_license(&lic, &cfg.deny_licenses) {
                    eprintln!("sentinel: denied license '{lic}' on package '{pkg}' ({label})");
                    denied_lic = true;
                }
            }
            if !scan.outdated.is_empty() { report.insert(label.clone(), scan.outdated); }
            entries.extend(scan.direct);
            entries.retain(|(_, n, _)| !cfg.ignore_packages.iter().any(|ig| ig == n));
            if !entries.is_empty() { lock_inputs.push((label, entries)); }
        }
    });
    match lockfiles::scan_batch(lock_inputs) {
        Ok(lock_vulns) => ScanOutcome { report, lock_vulns, osv_err: false, denied_lic },
        Err(e) => { eprintln!("sentinel: {e}"); ScanOutcome { report, lock_vulns: LockVulns::new(), osv_err: true, denied_lic } }
    }
}
