# Agents

Grado is a game for building apps with AI: a player follows a project's steps, and their LLM
connects to this backend over MCP. The backend is a Rust workspace.

## Read first

- `docs/rules.md`: every rule in force, one line each, with the decision it comes from. Read the
  section for the code you're building or reviewing.
- `docs/domain/user-flow.md`: what the product does.
- `docs/domain/api-map.md`: every call, its use case, and why it exists.
- `docs/decisions/`: accepted decisions. `0004-architecture.md` is the architecture.

## Layout

- `crates/`: one crate per folder, as in ADR 0004.
- `docs/`: decisions and domain docs.
- `content/`: proof-of-concept content, kept until the content format is decided. Don't build on it.

## Rules

- Follow `docs/rules.md`.
- If `docs/` doesn't answer an architecture question, ask. The answer is written into `docs/`
  before code relies on it.
- A change that accepts or supersedes a decision updates `docs/rules.md` in the same pull request.
- Commits follow Conventional Commits (ADR 0005).
- Before every commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`.
