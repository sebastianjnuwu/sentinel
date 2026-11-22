# AGENTS.md — Sentinel

## 1. Project Identity & Scope

**Sentinel** is a fast, pure-Rust, cross-platform CLI for **Dependency Scanning** across multiple ecosystems.

- **Core mission:** Recursively discover projects, detect outdated packages (`MAJOR`, `MINOR`, `PATCH`), inspect lockfiles for direct and transitive vulnerabilities via **Google OSV** (`CVE`, `GHSA`, `RUSTSEC`, `GO`, `PYSEC`, etc.), report package licenses, and provide exact update commands.
- **No AI/LLM runtime:** Sentinel is strictly a deterministic CLI. Never add LLM/Ollama/AI dependencies or probabilistic output.
- **Language:** All CLI output, code, comments, identifiers, and documentation MUST be in **English**.

---

## 2. Hard File & Module Size Rule (MANDATORY)

- **70-Line Maximum per `.rs` File:** No Rust source file may exceed **70 lines** total.
- **Mandatory Subdivision:** Whenever a file reaches or exceeds 70 lines, immediately refactor it into a directory module with focused submodules:
  ```text
  src/
  ├── main.rs                  # CLI args, orchestration, exit codes (<= 70 lines)
  ├── cmd.rs                   # Cross-platform subprocess runner (<= 70 lines)
  ├── discover.rs              # Recursive project discovery (<= 70 lines)
  ├── types.rs                 # Shared structs (OutdatedPkg, TransitiveVuln) (<= 70 lines)
  ├── version.rs               # Semver cleaning & MAJOR/MINOR/PATCH classifier (<= 70 lines)
  ├── osv.rs                   # Google OSV batch vulnerability queries (<= 70 lines)
  ├── output/
  │   ├── mod.rs
  │   ├── table.rs             # Unicode box-drawing table renderer (<= 70 lines)
  │   └── icons.rs             # Ecosystem icons & update command hints (<= 70 lines)
  ├── collectors/
  │   ├── mod.rs               # Dispatcher across ecosystems (<= 70 lines)
  │   ├── node.rs              # npm / pnpm / yarn / bun + npm registry (<= 70 lines)
  │   ├── go.rs                # go.mod / go list (<= 70 lines)
  │   ├── rust.rs              # Cargo.toml + crates.io API (<= 70 lines)
  │   ├── python.rs            # uv / pip (<= 70 lines)
  │   ├── php.rs               # composer (<= 70 lines)
  │   ├── dotnet.rs            # dotnet (*.csproj / *.fsproj) (<= 70 lines)
  │   ├── docker.rs            # Dockerfile / docker-compose + Docker Hub (<= 70 lines)
  │   └── gha.rs               # .github/workflows + GitHub Releases (<= 70 lines)
  └── lockfiles/
      ├── mod.rs               # Deduplication & OSV batching (<= 70 lines)
      ├── npm.rs               # package-lock.json & pnpm-lock.yaml (<= 70 lines)
      ├── cargo.rs             # Cargo.lock (<= 70 lines)
      ├── python.rs            # uv.lock (<= 70 lines)
      ├── go.rs                # go.sum (<= 70 lines)
      └── misc.rs              # composer.lock, Gemfile.lock, pubspec.lock, mix.lock (<= 70 lines)
  ```
- **Function Size:** Keep individual functions **<= 30 lines**. Prefer early returns (`let Some(x) = ... else { return ... };`) to avoid deep nesting.
- **Single Responsibility:** Each module owns exactly one ecosystem, one parser, or one rendering concern.

---

## 3. Explicit Rust Engineering Rules

### 3.1 Compiler & Lints
- Target **Rust Edition 2024**.
- Zero warnings allowed: `cargo clippy --all-targets -- -D warnings` MUST pass.
- Format all code with `cargo fmt` before finishing.

### 3.2 Error Handling & Safety
- **Never** use `.unwrap()` or `.expect()` on filesystem reads, network responses, JSON parsing, environment variables, or subprocess output.
- Network failures, missing CLIs, or malformed files in one project MUST fail gracefully (skip that collector/item) and never crash the entire scan.
- Avoid `unsafe` unless strictly required for OS FFI (e.g., `SetConsoleOutputCP(65001)` on Windows), and always isolate it behind a `#[cfg(target_os = "...")]` helper.

### 3.3 Dependencies & Performance
- Keep `Cargo.toml` minimal: prefer `std` + `ureq` (sync HTTP) + `serde` / `serde_json`.
- **Do NOT** introduce heavy async runtimes (`tokio`, `reqwest`, `hyper`) unless there is a proven bottleneck that `std::thread::scope` + `ureq` cannot solve.
- Run independent project scans and registry HTTP lookups in parallel using `std::thread::scope`.
- Batch external API calls whenever supported (e.g., OSV `/v1/querybatch` in chunks of 500).
- Always send a descriptive `User-Agent` header (`sentinel-deps-cli/<version>`) on third-party API requests (`crates.io`, GitHub API, etc.).

### 3.4 Cross-Platform Compatibility (Windows, Linux, macOS)
- Never hardcode path separators (`/` or `\`). Always use `std::path::Path` and `PathBuf::join`.
- Gate Windows-specific behavior with `#[cfg(target_os = "windows")]` and POSIX behavior with `#[cfg(not(target_os = "windows"))]`.
- Enable UTF-8 console output on Windows (`SetConsoleOutputCP(65001)`) so Unicode box-drawing borders and emoji icons render properly across Windows Terminal, PowerShell, Linux, and macOS.

---

## 4. Dependency Scanning & CLI Standards

- **Accuracy first:** Never guess or fabricate versions, licenses, or vulnerability IDs. Report `-` when metadata is unavailable.
- **Lockfiles + Transitive Deps:** Always parse lockfiles directly in Rust when possible so scanning works even without ecosystem CLIs or `node_modules` installed.
- **Vulnerability Identifiers:** Support all OSV ecosystems and IDs (`CVE-*`, `GHSA-*`, `RUSTSEC-*`, `GO-*`, `PYSEC-*`, etc.).
- **Licenses:** Extract license metadata from registries/lockfiles and visually highlight copyleft licenses (`GPL`, `AGPL`, `LGPL`).
- **Dual Output Modes:**
  - Default: colored, dynamically-sized Unicode table with ecosystem icons (`🟩`, `🐹`, `🦀`, `🐍`, `🐘`, `🟣`, `🐳`, `⚙️`), severity badges (`🔴 MAJOR`, `🟡 MINOR`, `🟢 PATCH`), `LICENSE`, `CVE`, exact update hints, and elapsed time.
  - `--json`: clean, valid JSON to `stdout` for CI/CD and scripting (no ANSI codes or decorative logs mixed into JSON output).

---

## 5. Mandatory Verification Checklist

Before considering any change complete:

1. Check line counts: verify every `.rs` file in `src/` is **<= 70 lines**.
2. Run `cargo fmt --check`.
3. Run `cargo clippy --all-targets -- -D warnings`.
4. Run `cargo install --path . --force`.
5. Run a real smoke test with `sentinel <path>` and `sentinel --json <path>` to confirm output formatting, icons, CVE detection, and exit status.
