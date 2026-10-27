# 0005. Conventional Commits

- **Status:** Accepted
- **Date:** 2026-10-27

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/). The scope is
the crate's name when a change stays inside one crate, or `infra` or `ci` for those areas; a change
spanning several areas has no scope. A `!` after the type or scope marks a breaking change.

**Consequences:** CI checks every commit in a pull request with a mature commit-message checker,
chosen in the CI ticket.
