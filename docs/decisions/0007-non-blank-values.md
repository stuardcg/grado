# 0007. Ids and readable text are never blank

- **Status:** Accepted
- **Date:** 2026-11-03
- **Ticket:** STU-42

Ids, and every text a player or agent reads (project, step and criterion titles, step guides,
criterion instructions and hints, handoff notes), are never blank. Blank means empty or only
whitespace, as `str::trim` defines it. The type `shared::text::NonBlankString` holds such text and
can't be built from a blank value; `Id::new` refuses a blank value too. Values are stored as
written. A blank handoff note counts as no note (ADR 0006 #11).

**Consequences:** code that receives a value checks it once, where it builds the type, and nothing
downstream checks again. Ids and text from a request, a file or storage are rejected at that
boundary. There is still no minimum length beyond one non-whitespace character.
