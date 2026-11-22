# Security Policy

## Supported versions

| Version | Supported |
| --- | :---: |
| 1.0.x | ✅ |
| < 1.0 | ❌ |

## Reporting a vulnerability

Please **do not open a public issue** for security problems.

Use GitHub's private vulnerability reporting:

<https://github.com/sebastianjnuwu/sentinel/security/advisories/new>

Include as much of the following as possible:

- Sentinel version (`sentinel --version`) and OS / architecture
- A minimal reproduction (project tree, command, output)
- Impact and, if known, the affected module (`src/...`)
- Whether the problem is in Sentinel itself or in a scanned dependency

## What to expect

- Acknowledgement within **72 hours**
- Triage and severity assessment within **7 days**
- A fix or mitigation plan for confirmed issues
- Credit in the advisory, unless you prefer to stay anonymous
- Please allow a reasonable window before public disclosure

## Scope

**In scope:** the Sentinel CLI, manifest and lockfile parsers, OSV integration,
JSON/SARIF output, release artifacts and the CI workflows of this repository.

**Out of scope:** vulnerabilities in third-party packages or container images
that Sentinel *reports* (report those upstream), and vulnerabilities in GitHub
Actions used by this repository (report upstream, but let us know so we can bump
the version).

## Safe harbor

We will not pursue legal action for good-faith research that follows this
policy, avoids privacy violations and service disruption, and gives us a
reasonable amount of time to fix the issue before public disclosure.
