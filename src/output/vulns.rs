use crate::types::TransitiveVuln;
use std::collections::BTreeMap;

pub fn print_lockfile_vulns(vulns: &BTreeMap<String, Vec<TransitiveVuln>>) {
    println!("🔒 \x1b[1;31mLockfile / Transitive Vulnerabilities (OSV):\x1b[0m");
    for (proj, list) in vulns {
        println!("  📦 \x1b[1;36m{proj}\x1b[0m \x1b[90m({} vulnerable package(s))\x1b[0m", list.len());
        for v in list {
            println!(
                "    ⚠️  \x1b[1m{}@{}\x1b[0m \x1b[90m[{}]\x1b[0m → \x1b[1;31m{}\x1b[0m",
                v.package,
                v.version,
                v.ecosystem,
                v.ids.join(", ")
            );
        }
    }
    println!();
}
