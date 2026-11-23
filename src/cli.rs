use std::{env, path::PathBuf, process};

pub struct CliOpts {
    pub root: PathBuf,
    pub json: bool,
    pub sarif: bool,
    pub fail_on_cve: bool,
    pub fail_on_major: bool,
    pub no_cache: bool,
    pub max_depth: usize,
}

fn die(msg: &str) -> ! {
    eprintln!("sentinel: {msg}");
    process::exit(2);
}

pub fn parse() -> CliOpts {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut o = CliOpts {
        root: PathBuf::new(), json: false, sarif: false,
        fail_on_cve: false, fail_on_major: false, no_cache: false, max_depth: 3,
    };
    let (mut root, mut i) = (None, 0);
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => { print_help(); process::exit(0); }
            "-V" | "--version" => { println!("sentinel {}", env!("CARGO_PKG_VERSION")); process::exit(0); }
            "-u" | "--update" => {
                if let Err(e) = crate::update::run_self_update() { die(&e); }
                process::exit(0);
            }
            "--json" => o.json = true,
            "--sarif" => o.sarif = true,
            "--fail-on-cve" => o.fail_on_cve = true,
            "--fail-on-major" => o.fail_on_major = true,
            "--no-cache" => o.no_cache = true,
            "--depth" => {
                let raw = args.get(i + 1).unwrap_or_else(|| die("--depth requires a number"));
                o.max_depth = raw.parse().unwrap_or_else(|_| die(&format!("invalid --depth: {raw}")));
                i += 1;
            }
            a if a.starts_with('-') => die(&format!("unknown flag: {a}")),
            a if root.is_none() => root = Some(PathBuf::from(a)),
            a => die(&format!("unexpected argument: {a}")),
        }
        i += 1;
    }
    o.root = root.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    if !o.root.exists() || !o.root.is_dir() {
        die(&format!("invalid directory: {}", o.root.display()));
    }
    o
}

fn print_help() {
    println!("sentinel {} - Multi-ecosystem dependency & CVE scanner\n", env!("CARGO_PKG_VERSION"));
    println!("USAGE:\n  sentinel [OPTIONS] [PATH]\n\nOPTIONS:");
    println!("  --json            Output machine-readable JSON");
    println!("  --sarif           Output SARIF 2.1.0 for GitHub Security");
    println!("  --fail-on-cve     Exit with code 1 if any CVE/GHSA is found");
    println!("  --fail-on-major   Exit with code 1 if any MAJOR update is found");
    println!("  --depth <N>       Max directory scan depth (default: 3)");
    println!("  --no-cache        Bypass 15-min local registry cache");
    println!("  -u, --update      Download and install the latest Sentinel release");
    println!("  -V, --version     Print version");
    println!("  -h, --help        Print help");
}
