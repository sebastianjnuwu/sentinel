use crate::console::c;
use crate::output::format::safe;
use crate::types::TransitiveVuln;
use std::collections::BTreeMap;

pub fn print_lockfile_vulns(vulns: &BTreeMap<String, Vec<TransitiveVuln>>) {
    let (dim, bold, cyan, red, rst) = (c("\x1b[90m"), c("\x1b[1m"), c("\x1b[1;36m"), c("\x1b[1;31m"), c("\x1b[0m"));
    println!("🔒 {red}Lockfile / Transitive Vulnerabilities (OSV):{rst}");
    for (proj, list) in vulns {
        println!("  📦 {cyan}{}{rst} {dim}({} vulnerable package(s)){rst}", safe(proj), list.len());
        for v in list {
            println!("    ⚠️  {bold}{}@{}{rst} {dim}[{}]{rst} → {red}{}{rst}", safe(&v.package), safe(&v.version), safe(&v.ecosystem), safe(&v.ids.join(", ")));
        }
    }
    println!();
}
