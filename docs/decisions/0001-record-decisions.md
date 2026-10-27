# 0001. Record decisions in the repo

- **Status:** Accepted
- **Date:** 2026-10-27

Every architectural decision gets a short record in `docs/decisions/`, using the template in the
README. A decision ticket is closed only once its record is committed.

**Consequences:** the repo explains itself without access to the tracker. Rules that can be
enforced in code (crate boundaries, tests, infrastructure as code) cite the record they come from.
