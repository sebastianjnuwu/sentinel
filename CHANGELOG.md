# Changelog

All notable changes to **Sentinel** will be documented in this file.

## [Unreleased]

### Changed
- The binary now consumes the `sentinel-scan` library target instead of declaring
  its own copy of the modules, so the crate is compiled once. No behavior change.

### Security
- Bumped the transitive `cookie` crate from `0.18.2` to `0.18.3`, fixing
  [RUSTSEC-2026-0594](https://rustsec.org/advisories/RUSTSEC-2026-0594.html)
  (`cookie` via `ureq` -> `cookie_store`). Lockfile-only change; no API impact.

## [1.0.1] - 2026-11-22

### Added
- Library target (`src/lib.rs`) for `docs.rs` documentation builds.
- Self-update flag (`-u`, `--update`) and non-blocking background update notice (configurable via `check_updates = false` in `sentinel.toml`).
- Modular integration test suites under `tests/` (`cli_test`, `config_test`, `lockfiles_test`, `collectors_test`, `output_test`).
- OSV vulnerability scanning for GitHub Actions (`versions` and `ECOSYSTEM`/`SEMVER` range matching), Swift (`SwiftURL`), Docker Hub images (via OCI SBOM attestations + PURL OSV batching), and up-to-date direct dependencies.
- Native lockfile/manifest support for `go.sum` + `go.mod replace`, `package-lock.json` v1, Yarn Berry (v2+), `Cargo.toml` table & target sections, Maven `${properties}` / `<dependencyManagement>` / local parent POMs, Gradle Groovy map notation + `libs.versions.toml`, Swift `Package.resolved` v1, and lockfile-only project discovery.

### Changed
- Upgraded `ureq` from `2.12.1` to `3.5.2` with a 15s global HTTP request timeout.
- Replaced host-environment Python `pip list` scanning with project manifest (`requirements.txt` / `pyproject.toml`) + PyPI lookups.
- Unified OSV batching (chunks of 500), bounded 16-worker concurrency across all HTTP collectors, and moved disk cache to user cache directory with expired-entry pruning.
- Respected `NO_COLOR` / non-TTY output, sanitized control chars, added SARIF `driver.rules`, removed redundant `package.homepage` in `Cargo.toml`.

### Fixed
- Removed 2-CVE truncation per package across direct and transitive results.
- Propagated OSV lookup failures with retries and non-zero exit instead of reporting silent clean scans.
- Rejected unknown CLI flags, invalid `--depth`, and nonexistent/non-directory paths with exit code `2`.
- Applied `ignore_packages` to transitive lockfile vulnerabilities, supported multi-line/single-quoted `sentinel.toml` arrays, and used token-based SPDX matching for `deny_licenses`.
- Isolated lockfile discovery to each project directory, filtered git/path entries from `Cargo.lock`/`uv.lock`, fixed `resolve_ver` fallback, and warned on unsupported Docker registries/tags.

## [1.0.0] - 2026-11-22

### Added
- **Multi-Ecosystem Dependency Discovery & Collectors (14 ecosystems):**
  - 🟩 **JS / TS:** `npm`, `pnpm`, `yarn`, and `bun` (`package.json` + npm registry API).
  - 🐹 **Go:** Go modules (`go.mod` / `go list`).
  - 🦀 **Rust:** Cargo (`Cargo.toml` + `crates.io` API).
  - 🐍 **Python:** `uv` and `pip` (`pyproject.toml`, `requirements.txt`).
  - 🐘 **PHP:** Composer (`composer.json`).
  - 🟣 **.NET:** NuGet (`*.csproj`, `*.fsproj`).
  - ☕ **Java / Kotlin:** Maven (`pom.xml`) and Gradle (`build.gradle`, `build.gradle.kts`) via Maven Central API.
  - 💎 **Ruby:** Bundler (`Gemfile` + RubyGems API).
  - 🐦 **Dart / Flutter:** Pub (`pubspec.yaml` + `pub.dev` API).
  - ⚡ **Elixir:** Hex (`mix.exs` + `hex.pm` API).
  - 🍎 **Swift / iOS:** Swift Package Manager (`Package.resolved` + GitHub Releases API).
  - 🧱 **Terraform:** Terraform Providers (`.terraform.lock.hcl` + Terraform Registry API).
  - 🐳 **Docker:** Base images in `Dockerfile`, `docker-compose.yml`, and `docker-compose.yaml` via Docker Hub API.
  - 🐙 **GitHub Actions:** Workflow action tags in `.github/workflows/*.yml` and `*.yaml` via GitHub Releases API.
- **Native Lockfile Parsers & Transitive Vulnerability Scanning (12 lockfile formats):**
  - Direct Rust parsers for `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `Cargo.lock`, `uv.lock`, `poetry.lock`, `go.sum`, `composer.lock`, `Gemfile.lock`, `pubspec.lock`, `mix.lock`, and `packages.lock.json`.
  - Batch queries against the **Google OSV** `/v1/querybatch` endpoint (chunks of 500) detecting `CVE`, `GHSA`, `RUSTSEC`, `GO`, and `PYSEC` advisories across direct and transitive dependencies.
- **License Auditing:**
  - Extracts package license metadata (`MIT`, `Apache-2.0`, `ISC`, etc.) and highlights copyleft licenses (`GPL`, `AGPL`).
- **Configuration (`sentinel.toml`):**
  - Project-level configuration supporting `ignore_packages`, `ignore_dirs`, and `deny_licenses`.
- **Performance & Concurrency:**
  - Shared `ureq` HTTP Keep-Alive connection pool with pure-Rust TLS (`rustls`).
  - Bounded 16-worker thread pool (`std::thread::scope`) for parallel project and package scanning.
  - Atomic 15-minute local disk cache (`sentinel-registry-cache.json`) with `--no-cache` bypass flag.
- **CLI & CI/CD Output Modes:**
  - Compact, colored Unicode box-drawing terminal table with automatic Windows UTF-8 console support (`SetConsoleOutputCP(65001)`).
  - Structured `--json` output and **SARIF 2.1.0** (`--sarif`) output for **GitHub Security → Code Scanning**.
  - Quality-gate flags `--fail-on-cve`, `--fail-on-major`, and `--depth <N>`.
- **Distribution & CI/CD Workflows:**
  - Multi-stage `FROM scratch` `Dockerfile` published to GitHub Container Registry (`ghcr.io/sebastianjnuwu/sentinel`).
  - Automated GitHub Releases (`release.yml`) building native binaries for **Windows (`amd64`)**, **Linux (`amd64`)**, and **macOS (`arm64`)**.
  - Strict modular architecture enforcing `<= 70` lines per `.rs` file in CI (`ci.yml`).
