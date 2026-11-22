# Contributing to Sentinel

Thanks for helping improve Sentinel! This document covers the workflow and the
project rules. By participating, you agree to follow the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Prerequisites

- Rust with **Edition 2024** support (1.85+)
- Git
- Optional: ecosystem CLIs (`npm`, `cargo`, `go`, `pip`, ...) for manual testing.
  Sentinel parses lockfiles natively, so most work does not need them.

## Build, test and lint

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Before opening a PR, also run a real smoke test:

```bash
cargo install --path . --force
sentinel .
sentinel --json . | head
```

## Project rules

See [AGENTS.md](AGENTS.md) for the full rules. The essentials:

- **English only** for code, comments, identifiers, CLI output and docs.
- **70 lines maximum** per Rust source file; split into focused submodules.
  Functions should stay under 30 lines.
- Never `unwrap()` / `expect()` on filesystem, network, JSON, env or subprocess
  results. Fail gracefully per item.
- Keep dependencies minimal (`std` + `ureq` + `serde`). Do not add async
  runtimes without a proven need.
- Parse lockfiles directly in Rust whenever possible.

## Finding something to work on

- Issues labeled [`good first issue`](https://github.com/sebastianjnuwu/sentinel/labels/good%20first%20issue)
- The audit index in [#5](https://github.com/sebastianjnuwu/sentinel/issues/5)
- Issues labeled `severity:critical` have the highest impact

## Pull requests

1. Branch from `production`.
2. Use a conventional commit message (`fix:`, `feat:`, `docs:`, `chore:`).
3. Reference the issue in the PR body (`Closes #123`).
4. Add a test or fixture for every bug fix.
5. Keep the PR focused; separate refactors from behavior changes.
6. Update `CHANGELOG.md` for user-visible changes.
7. Make sure CI is green and the PR has the required review.

Security reports must follow [SECURITY.md](SECURITY.md), not the public issue
tracker.
