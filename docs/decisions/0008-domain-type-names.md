# 0008. Domain type names

- **Status:** Accepted
- **Date:** 2026-11-04
- **Ticket:** STU-42

Types in `domain` are named after the type or operation that owns them:

| Role | Name | Example |
|---|---|---|
| Error from a constructor | `<Type>Error` | `ProjectError` from `Project::new` |
| Input to an operation | `<Verb><Noun>` | `SubmitReport`, taken by `Progress::submit` |
| Success of an operation | Named after the operation | `Submitted` |
| Error from an operation | `<Operation>Error` | `SubmitError`, `StampError` |
| Value only an operation creates | Named for what it is | `StepReport`, stored by `submit` |

An error type belongs to one constructor or operation, and lists only what that one can return. A
second fallible method on the same type gets its own `<Operation>Error`; it doesn't add variants
to an existing one.

**Consequences:** a reader can tell from a type's name where it comes from, and a caller matches
only the failures its call can produce. Use cases keep their own `Input`, `Output` and `Error`
(ADR 0004 rule 6) and convert domain errors into their `Error`.
