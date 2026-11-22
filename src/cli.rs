use std::env;
use std::path::PathBuf;

pub struct CliOpts {
    pub root: PathBuf,
    pub json: bool,
    pub sarif: bool,
    pub fail_on_cve: bool,
    pub fail_on_major: bool,
    pub no_cache: bool,
    pub max_depth: usize,
}

pub fn parse() -> CliOpts {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        std::process::exit(0);
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("sentinel {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let mut max_depth = 3usize;
    let mut root: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--depth" if i + 1 < args.len() => {
                max_depth = args[i + 1].parse().unwrap_or(3);
                i += 2;
            }
            a if a.starts_with('-') => i += 1,
            a => {
                if root.is_none() { root = Some(PathBuf::from(a)); }
                i += 1;
            }
        }
    }

    CliOpts {
        root: root.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        json: args.iter().any(|a| a == "--json"),
        sarif: args.iter().any(|a| a == "--sarif"),
        fail_on_cve: args.iter().any(|a| a == "--fail-on-cve"),
        fail_on_major: args.iter().any(|a| a == "--fail-on-major"),
        no_cache: args.iter().any(|a| a == "--no-cache"),
        max_depth,
    }
}

fn print_help() {
    println!("sentinel {} - Multi-ecosystem dependency & CVE scanner\n", env!("CARGO_PKG_VERSION"));
    println!("USAGE:\n  sentinel [OPTIONS] [PATH]\n");
    println!("OPTIONS:");
    println!("  --json            Output machine-readable JSON");
    println!("  --sarif           Output SARIF 2.1.0 for GitHub Security");
    println!("  --fail-on-cve     Exit with code 1 if any CVE/GHSA is found");
    println!("  --fail-on-major   Exit with code 1 if any MAJOR update is found");
    println!("  --depth <N>       Max directory scan depth (default: 3)");
    println!("  --no-cache        Bypass 15-min local registry cache");
    println!("  -V, --version     Print version");
    println!("  -h, --help        Print help");
}
