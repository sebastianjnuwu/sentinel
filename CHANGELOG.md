# Changelog

All notable changes to **Sentinel** will be documented in this file.

## [Unreleased]

### Changed
- crates.io package renamed to `sentinel-scan` (the binary remains `sentinel`).
- Added crates.io metadata: description, repository, homepage, docs, keywords, categories and MSRV.

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
