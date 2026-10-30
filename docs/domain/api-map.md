# API map

Each call the backend exposes, the use case behind it, the step of the [user flow](user-flow.md) it
serves, and why it exists. Names are working names; paths, payloads and sign-in come later
(STU-25, STU-29).

## MCP tools (agent, through an agent connection)

Every tool except `list_my_projects` takes a project.

Before every tool, the MCP adapter calls `authenticate_agent_connection` with the connection's
token. It returns the player the connection acts for, or refuses a revoked or unknown connection
(E8). The tool's use case then runs for that player (ADR 0004 rule 24). It is not a tool itself.

| Tool | Use case | Flow | Why |
|---|---|---|---|
| `list_my_projects` | `list_my_projects` | M5 | The agent finds which project it is working on. If there are several and its location, codebase or context doesn't say which, it asks the person. |
| `get_briefing` | `get_briefing` | M5, E7, E11, E12 | A fresh agent needs the project, the current step, a preview of the next one and the previous handoff note. No criteria. |
| `get_criteria` | `get_criteria` | M7 | Criteria come only after the work, so the agent doesn't build towards the checks. The response suggests a fresh review agent. |
| `send_report` | `submit_report` | M8, M9, failed report | The agent's pass or fail per criterion, plus the handoff note when everything passes. A fail tells the agent to stop and tell the person. |
| `get_stamp` | `issue_stamp` | M9, E10, E14 | Hands the person a stamp once the step is approved. A new call voids the previous stamp. Refused when the stamp is off. |

## HTTP: app (signed-in person)

| Call | Use case | Flow | Why |
|---|---|---|---|
| List projects | `list_projects` | M2 | Shows the predefined projects to choose from. |
| List my projects | `list_my_projects` | M2, O3 | The projects the person has started. |
| Start a project | `start_project` | M2 | Creates the player's progress for that project. |
| View a project | `view_project` | M3, O3 | The map: every step and its state. |
| View a step | `view_step` | M3, O4, failed report | Guide, criteria, hints, and the latest report. |
| Get a stamp | `issue_stamp` | E10 | The app's backup when the agent's stamp is lost or expired. |
| Enter a stamp | `redeem_stamp` | M10 | Completes an approved step when the stamp is on. |
| Skip a step | `skip_step` | O8 | The way out of a stuck step, with a confirmation. |
| Reopen a step | `reopen_step` | O8 | Returns to a skipped step. |
| Reset progress | `reset_progress` | O5 | Replays a step or the whole project. |
| Create, list, revoke agent connections | `create_agent_connection`, `list_agent_connections`, `revoke_agent_connection` | M4, E8 | Connects the person's LLM client, and cuts off a leaked one. |

## HTTP: devtools (`developer` role)

| Call | Use case | Flow | Why |
|---|---|---|---|
| List players | `list_players` | O6 | Find a player to inspect. |
| View a player | `view_player` | O6 | Every project, step state and report. |
| Set a step's state | `set_step_state` | O6 | Put a player in any state for testing. |
| Set the stamp setting | `set_stamp_setting` | O1, O6 | Turn the stamp on or off globally, per project or per player. |
| Reset a player | `reset_progress` | O5, O6 | The same reset, for any player. |

## Open

- Whether opening a hint is recorded.
