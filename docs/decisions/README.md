# Decisions

One file per architectural decision, numbered in order. Accepted records are not edited; a later
record supersedes them. Discussion happens in the linked Linear ticket; the record keeps only the
outcome.

| # | Decision | Status |
|---|---|---|
| [0001](0001-record-decisions.md) | Record decisions in the repo | Accepted |
| [0002](0002-rust.md) | Write the backend in Rust | Accepted |
| [0003](0003-hosting.md) | Host on AWS Lambda behind a Function URL | Accepted |
| [0004](0004-architecture.md) | Hexagon with one module per use case | Accepted |
| [0005](0005-conventional-commits.md) | Conventional Commits | Accepted |
| [0006](0006-domain-model.md) | Domain model | Accepted |
| [0007](0007-non-blank-values.md) | Ids and readable text are never blank | Accepted |

## Template

```markdown
# NNNN. Title

- **Status:** Proposed | Accepted | Superseded by NNNN
- **Date:** YYYY-MM-DD
- **Ticket:** STU-NN

Decision in one or two sentences.

**Consequences:** what this rules in or out.
```
