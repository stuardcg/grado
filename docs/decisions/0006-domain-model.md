# 0006. Domain model

- **Status:** Accepted
- **Date:** 2026-10-28
- **Ticket:** STU-26

The concepts, flow and edge cases are in [`docs/domain/user-flow.md`](../domain/user-flow.md).
Decisions:

1. A player is a person with an account; agent connections belong to it.
2. Projects are predefined. Each has its own ordered steps.
3. A player can have several projects in progress, each with its own progress.
4. No server-side grader. The agent checks the work and reports pass or fail per criterion, with no
   evidence. False reports are accepted.
5. The agent gets the criteria only on request, after the work. The briefing has the current step
   and a preview of the next one, and no criteria.
6. A failed report is recorded. The agent stops and tells the person; the app shows the failed
   criteria and their hints.
7. Hints belong to criteria and are shown in the app only.
8. The stamp is toggleable globally, per project and per player; the most specific setting wins.
   Once a step is approved, a new stamp is always available and voids the old one.
9. A player can skip a step; it stays `skipped`, not `done`, and can be reopened. A player can reset
   their own progress.
10. A `developer` role can change any player's state through devtools.
11. A step can't be approved without a handoff note. There is no minimum length.
12. The player sees the latest report of each step; devtools show every report.
13. Every agent call names a project. The agent lists the person's projects first; if several fit
    and nothing in its location, codebase or context says which, it asks the person.

**Consequences:** the backend never judges work, so grading needs no model or external service.
