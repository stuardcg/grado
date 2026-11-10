# 0013. Testing and CI

- **Status:** Accepted
- **Date:** 2026-11-09
- **Ticket:** STU-34

Every change reaches `main` through a pull request, and GitHub Actions runs the required checks on
it. Tests run with `cargo test`. Each layer is tested where ADR 0004 puts its tests, and an inbound
adapter's tests cover authentication and every caller-facing refusal it writes.

## What each layer tests

| Layer | Tests | Where |
|---|---|---|
| `shared`, `domain` | Unit tests of each type and rule; `proptest` for an invariant that must hold over any sequence of operations | The module, in `#[cfg(test)]` |
| Use cases | Every outcome of `run`, on the testkit fakes | The use case's file (ADR 0004) |
| Outbound adapters | The port's shared suite | The adapter crate, through `testkit` (ADR 0004) |
| Inbound adapters | Every tool, the authentication refusal, and every arm of each `Error` `match`, through the real binary and a real client | `app/tests/` |
| `app` | Configuration loading | `app/src/config.rs` |

## Checks on every pull request

All are required: a pull request merges only when they pass. Pull requests merge by rebase, so the
commits on `main` are the ones checked.

| Check | Tool | Config |
|---|---|---|
| Formatting | `cargo fmt --check` | `rustfmt.toml` |
| Lints | `cargo clippy --all-targets --locked -- -D warnings` | |
| Tests | `cargo test --locked` | |
| Dependencies: RustSec advisories, licences, sources | cargo-deny | `deny.toml` |
| Every commit message in the pull request (ADR 0005) | committed | `committed.toml` |

The workflow is `.github/workflows/ci.yml`. It runs on the latest stable Rust; a new lint that
fails is fixed in the pull request that meets it. `main` runs the same checks after each merge.

## Outside the pull request

- **Mutation testing:** cargo-mutants runs nightly on `main` (`.github/workflows/mutants.yml`) and
  reports each change to the code that no test caught. It never blocks.
- **Dependency updates:** Dependabot opens grouped pull requests weekly, for crates and for
  GitHub Actions (`.github/dependabot.yml`).

## Decided elsewhere

| Check | Ticket |
|---|---|
| ADR 0004's architecture rules: clippy per crate, cargo-deny bans, ast-grep, and a `cargo metadata` check for rule 2 | STU-38 |
| Content validation | STU-27 |
| Preview deploy | STU-37 |
| AI review and QA | STU-52 |
| Duplication, complexity and coverage | STU-51 |
| CodeQL, once the repo is public | STU-50 |

**Consequences:** the local checks in `AGENTS.md` are the same as CI's first three, so a change
that passes them locally fails CI only on a dependency or a commit message, and `cargo deny check`
and `committed` run locally too. Branch protection on `main` requires the three jobs; it is set in
the repository settings, not in code. No other test runner, end-to-end tool or pinned toolchain is
used. The MCP adapter's tests don't yet cover every arm of every `Error` `match`.
