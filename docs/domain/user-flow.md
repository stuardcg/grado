# User flow

The flow, its edge cases and the decisions behind it, recorded in
[ADR 0006](../decisions/0006-domain-model.md). Implementation choices (IDs, storage, formats) are
left out on purpose.

The product: a game to build apps with AI by following recommended steps, with a game-like design,
connected to the player's LLM through MCP.

## Concepts

| Concept | What it is |
|---|---|
| Player | A person with an account. Has the `player` role, and optionally the `developer` role. |
| Project | A predefined app to build, chosen from a list. Each project has its own ordered steps. |
| Step | One recommended step of a project. Its page has a guide; it has criteria. |
| Guide | Reading material on the step page: why the step exists, how to approach it, tips. Always visible. |
| Criterion | One thing the step's work must satisfy: a title and instructions the agent checks against. |
| Hint | A short, usable answer linked to a criterion. Shown in the app only, at any time. |
| Progress | A player's state in one project: which step is open and the state of each step. |
| Agent connection | How the player's LLM client reaches the game through MCP, acting for one player. |
| Briefing | What an agent receives when it connects: project, current step, preview of the next step, previous handoff note. |
| Report | The agent's result for a step: pass or fail for each criterion. No evidence. |
| Handoff note | A short summary the agent sends with an all-pass report; required for approval. Shown in the next briefing. |
| Stamp | A one-time code issued when a step is approved. When the stamp is on, entering it completes the step. |

Step states: `locked`, `open`, `approved` (every criterion passed, waiting for the stamp),
`done`, `skipped`.

## Main flow

| # | Step | Where |
|---|---|---|
| M1 | The person signs in. | App |
| M2 | They pick a project from the predefined list. They can have several in progress. | App |
| M3 | They open the current step and read its guide. | App |
| M4 | First time only: they create an agent connection and add the MCP server to their client. | App, client |
| M5 | They open an agent. It lists the person's projects and picks the one it's working on; if it can't tell from its location, codebase or context, it asks. Then it gets the briefing. The codebase is the rest of its context. | MCP |
| M6 | The person and the agent build the step. | Client |
| M7 | When the work is done, the agent requests the criteria. The response suggests spawning a fresh review agent if the client can. | MCP |
| M8 | The agent checks the codebase against the criteria and sends a report. | MCP |
| M9 | All passed and a handoff note is sent: the step is approved, and a stamp is available from the MCP and the API. | MCP, API |
| M10 | Stamp on: the person enters the stamp and the step is done. Stamp off: the step is done at approval. | App |
| M11 | The next step opens. The MCP response suggests a fresh agent for it, or continuing here if the person wants. | MCP |

**Failed report (M8 with a fail):** the report is recorded. The response tells the agent which
criteria failed and to stop and tell the person. The agent doesn't fix anything on its own. The app
shows the failed criteria and surfaces their linked hints. The person fixes the work, and the agent
reports again.

## Optional features

| # | Feature | Decision |
|---|---|---|
| O1 | Stamp | Toggleable globally, per project and per player. Player wins over project, project over global. |
| O2 | Handoff note | Required for approval, no minimum length. Shown in the next briefing, for context when a new agent starts. |
| O3 | Progress map in the app | Kept. |
| O4 | Hints | Per criterion, visible at any time in the app, surfaced when a criterion fails. Not sent over MCP. |
| O5 | Reset my own progress | Kept: a step or the whole project. |
| O6 | Devtools | `developer` role. A small, unpolished frontend that changes any player's state live, through the same HTTP adapter. |
| O7 | Predefined projects | Kept. Players can't define their own project. |
| O8 | Skip a stuck step | A button in the app, with a confirmation. The step is marked `skipped`, not `done`, and can be reopened. |

## Edge cases

| # | Case | Behaviour |
|---|---|---|
| E1 | Report for a step that isn't open, or whose criteria don't match the step's. | Refused; the response names the open step. |
| E2 | Agent asks for the stamp before approval. | Refused; the response says which criteria haven't passed. |
| E3 | The same criterion keeps failing. | Hints in the app, then skip (O8). |
| E4 | The same report arrives twice (client retry). | Recorded once; the second call gets the same answer. |
| E5 | Two agent connections act at once. | Both can read. Writes don't conflict silently; the second sees the new state. |
| E6 | An agent reports passes that aren't true. | Accepted. It's a learning platform; cheating only cheats the player. |
| E7 | New agent, or lost context partway through a step. | The briefing, the handoff note and the codebase restore context. |
| E8 | A connection is revoked or leaked. | Revoked connections are refused at once. Progress is unaffected. |
| E9 | Project content changes while a player is on it. | Chosen, may change: migrate only at a step boundary. |
| E10 | Stamp lost, expired, entered twice, or by another player. | Once approved, a new stamp is always available from the MCP or the app and voids the old one. Second entry and other players are refused. Stamps don't expire for now; whether they get a lifetime is STU-39. |
| E11 | The agent keeps going after a step is done. | The MCP response tells it to suggest a fresh agent, or to fetch the next briefing if the person wants to continue here. |
| E12 | The player comes back after a long time. | Same as E7, plus self-reset (O5). |
| E13 | The person deletes their account. | Later; not in the foundation. |
| E14 | Agent asks for a stamp while the stamp is off. | Refused; the response says the step was done at approval and needs no stamp. |

## Decisions

1. A player is a person with an account; agent connections belong to it.
2. Projects are predefined. Each project has its own ordered steps, until hands-on testing shows otherwise.
3. A player can have several projects in progress, each with its own progress.
4. No server-side grader. The agent checks the work and reports pass or fail per criterion.
5. Criteria are sent to the agent only on request, after the work. The briefing has no criteria.
6. The briefing has the current step and a short preview of the next one. Its content can be cut down later.
7. A failed report is recorded. The agent stops and tells the person; the app shows the failure and its hints.
8. The stamp is toggleable at three levels (see O1).
9. Hints are app-only.
10. A skipped step stays `skipped` and can be reopened.
11. Devtools are a `developer` role and a small separate frontend.
12. Chosen, may change: the first developer gets the role through a one-off bootstrap command.
13. A step can't be approved without a handoff note. There is no minimum length.
14. The player sees the latest report of each step; devtools show every report.
15. The devtools frontend lives in this repository (monorepo); its layout is STU-31.
16. Every agent call names a project. The agent lists the person's projects first; if several fit
    and nothing in its location, codebase or context says which, it asks the person.
