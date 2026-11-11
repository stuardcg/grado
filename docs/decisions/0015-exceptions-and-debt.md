# 0015. Exceptions and debt

- **Status:** Accepted
- **Date:** 2026-11-11
- **Ticket:** STU-35

A rule in force is never broken. A change that needs to break one first changes the rule with a
decision record. Known debt is a Linear ticket, and the code it affects names that ticket in a
comment that says what replaces the code, such as `until STU-27 decides the content format`. The
change that closes the ticket removes the comment.

**Consequences:**
- There is no exceptions register and no debt register. `grep -rn "STU-" crates` lists the debt in
  the code, and Linear lists the rest.
- A gap a record knows about is stated in its Consequences and has a ticket, as ADR 0013 does with
  STU-53.
- ADR 0004 and `docs/rules.md` together are the architecture contract. STU-38's tool configs are
  its machine-readable part, and STU-52's `docs/review.md` is the reviewer's part.
