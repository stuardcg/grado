# 0011. Configuration

- **Status:** Accepted
- **Date:** 2026-11-09
- **Ticket:** STU-47

The server starts only from a complete, valid configuration. `app/src/config.rs` loads it once at
startup with `figment`, from `grado.toml` in the working directory if it's there and then every
`GRADO_*` environment variable on top, into one typed `Config`. A nested key is joined with `__`
in an environment variable: `dev.agent_token` is `GRADO_DEV__AGENT_TOKEN`. Every setting is
required and is text. A missing, invalid or unknown setting stops startup, and every problem is
listed at once, never with its value. `grado.example.toml` documents every setting, and a test
loads it. The composition root (ADR 0010) hands each adapter only its part of `Config`.

**Consequences:** ADR 0004 rule 17 now also lets `app` load configuration, in `app/src/config.rs`
only, and nothing else reads the environment. ADR 0004's "Where a change goes" gains a row: a new
setting is a field in `Config`, one read in `config.rs` and an entry in `grado.example.toml`, with
no default in code. Figment's `extract` stops at the first error, so `config.rs` reads key by key
to list every problem. A secret store joins later as one more figment provider, decided with the
first real secret. The `dev` settings are only for the sample data and go away with STU-27 and
STU-29.
