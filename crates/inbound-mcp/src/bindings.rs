//! The MCP tools, one per use case, written by hand (ADR 0017). `call_tool` authenticates the
//! connection once, before any tool runs, and each tool runs its use case for the player it acts
//! for (ADR 0004 rule 24). Every caller-facing sentence is written here, in one exhaustive `match`
//! per use case `Error` (rule 13). Each tool call runs in one span, opened in `call_tool`, and a
//! port failure is logged once, in `unavailable` (ADR 0012).

use std::sync::Arc;

use application::use_cases::{
    authenticate_agent_connection, get_briefing, get_criteria, issue_stamp, list_my_projects, submit_report,
};
use application::{AgentToken, Grado, PortError};
use domain::player::PlayerId;
use domain::progress::StepState;
use domain::project::{CriterionId, ProjectId, StepId};
use domain::report::{HandoffNote, SubmitReport, Submitted, Verdict};
use http::header::AUTHORIZATION;
use http::request::Parts;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::{Extension, ToolCallContext};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::id::Id;
use tracing::Instrument;

use crate::REQUEST_ID;

const INSTRUCTIONS: &str = "Grado guides a person through a project one step at a time. Start with \
    list_my_projects to find the project you're working on; if your location, codebase or context \
    doesn't say which, ask the person. Then call get_briefing and follow it.";

/// What a tool sends back: a result the agent reads, or a protocol error when the server can't
/// serve the call at all.
type Reply = Result<CallToolResult, ErrorData>;

#[derive(Clone)]
pub struct Tools {
    grado: Arc<Grado>,
    tool_router: ToolRouter<Self>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectRequest {
    /// The project's id, from list_my_projects.
    pub project: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReportRequest {
    /// The project's id, from list_my_projects.
    pub project: String,
    /// The step's id, from get_criteria.
    pub step: String,
    /// One verdict for each criterion get_criteria returned.
    pub verdicts: Vec<VerdictRequest>,
    /// Required when every criterion passes: a short summary for the agent that takes the next
    /// step. What was built, where it lives, and anything left open.
    pub handoff_note: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct VerdictRequest {
    /// The criterion's id, from get_criteria.
    pub criterion: String,
    /// Whether the codebase meets the criterion.
    pub pass: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StampRequest {
    /// The project's id, from list_my_projects.
    pub project: String,
    /// The approved step's id.
    pub step: String,
}

#[derive(Serialize)]
struct ProjectsResponse {
    projects: Vec<ProjectSummary>,
}

#[derive(Serialize)]
struct ProjectSummary {
    id: String,
    title: String,
}

#[derive(Serialize)]
struct BriefingResponse {
    project: String,
    title: String,
    current: Option<CurrentStepResponse>,
    next: Option<NextStepResponse>,
    handoff_note: Option<String>,
    next_action: String,
}

#[derive(Serialize)]
struct CurrentStepResponse {
    id: String,
    title: String,
    guide: String,
    state: &'static str,
}

#[derive(Serialize)]
struct NextStepResponse {
    id: String,
    title: String,
}

#[derive(Serialize)]
struct CriteriaResponse {
    step: String,
    criteria: Vec<CriterionResponse>,
    next_action: &'static str,
}

#[derive(Serialize)]
struct CriterionResponse {
    id: String,
    title: String,
    instructions: String,
}

#[derive(Serialize)]
struct ReportResponse {
    state: &'static str,
    failed: Vec<String>,
    next_step: Option<String>,
    next_action: String,
}

#[derive(Serialize)]
struct StampResponse {
    stamp: String,
    next_action: &'static str,
}

#[tool_router]
impl Tools {
    pub fn new(grado: Arc<Grado>) -> Self {
        Self { grado, tool_router: Self::tool_router() }
    }

    #[tool(description = "The projects the person has started, with their ids. Call this first.")]
    async fn list_my_projects(&self, Extension(player): Extension<PlayerId>) -> Reply {
        match self.grado.list_my_projects(list_my_projects::Input { player }).await {
            Ok(output) => json(&ProjectsResponse {
                projects: output
                    .projects
                    .into_iter()
                    .map(|(id, title)| ProjectSummary { id: id.as_str().into(), title: title.into_string() })
                    .collect(),
            }),
            Err(list_my_projects::Error::UnknownProject(project)) => refuse(format!(
                "The person started project {}, but it's no longer available. Tell the person.",
                project.as_str()
            )),
            Err(list_my_projects::Error::Port(error)) => unavailable(error),
        }
    }

    #[tool(description = "The project, the step to work on, a preview of the next one and the previous \
            step's handoff note. Call this at the start of every session. It has no criteria: those \
            come from get_criteria once the work is done.")]
    async fn get_briefing(
        &self,
        Extension(player): Extension<PlayerId>,
        Parameters(request): Parameters<ProjectRequest>,
    ) -> Reply {
        let project = match id(request.project, "project") {
            Ok(project) => project,
            Err(reply) => return reply,
        };
        match self.grado.get_briefing(get_briefing::Input { player, project }).await {
            Ok(get_briefing::Output { briefing }) => {
                let next_action = match &briefing.current {
                    Some(step) => match step.state {
                        StepState::Approved => format!(
                            "Step {} is approved and waits for its stamp. Call get_stamp and give it to \
                             the person.",
                            step.id.as_str()
                        ),
                        StepState::Open | StepState::Locked | StepState::Done | StepState::Skipped => {
                            format!(
                                "Build step {} with the person, following its guide. When the work is \
                                 done, call get_criteria.",
                                step.id.as_str()
                            )
                        }
                    },
                    None => "Every step of this project is done.".into(),
                };
                json(&BriefingResponse {
                    project: briefing.project.as_str().into(),
                    title: briefing.title.into_string(),
                    current: briefing.current.map(|step| CurrentStepResponse {
                        id: step.id.as_str().into(),
                        title: step.title.into_string(),
                        guide: step.guide.into_string(),
                        state: state(step.state),
                    }),
                    next: briefing.next.map(|step| NextStepResponse {
                        id: step.id.as_str().into(),
                        title: step.title.into_string(),
                    }),
                    handoff_note: briefing.handoff_note.map(|note| note.as_str().into()),
                    next_action,
                })
            }
            Err(get_briefing::Error::UnknownProject(project)) => unknown_project(&project),
            Err(get_briefing::Error::NotStarted(project)) => not_started(&project),
            Err(get_briefing::Error::Port(error)) => unavailable(error),
        }
    }

    #[tool(description = "The criteria for the open step. Call this only when the work is done, then check \
            the codebase against each one and call send_report.")]
    async fn get_criteria(
        &self,
        Extension(player): Extension<PlayerId>,
        Parameters(request): Parameters<ProjectRequest>,
    ) -> Reply {
        let project = match id(request.project, "project") {
            Ok(project) => project,
            Err(reply) => return reply,
        };
        match self.grado.get_criteria(get_criteria::Input { player, project }).await {
            Ok(get_criteria::Output { checklist }) => json(&CriteriaResponse {
                step: checklist.step.as_str().into(),
                criteria: checklist
                    .items
                    .into_iter()
                    .map(|item| CriterionResponse {
                        id: item.criterion.as_str().into(),
                        title: item.title.into_string(),
                        instructions: item.instructions.into_string(),
                    })
                    .collect(),
                next_action: "Check the codebase against each criterion, then call send_report with pass \
                    or fail for each. If your client can, spawn a fresh agent to review, so the work isn't \
                    checked by the agent that wrote it.",
            }),
            Err(get_criteria::Error::UnknownProject(project)) => unknown_project(&project),
            Err(get_criteria::Error::NotStarted(project)) => not_started(&project),
            Err(get_criteria::Error::NoOpenStep { current: Some(step) }) => refuse(format!(
                "No step takes a report: step {} is approved and waits for its stamp. Call get_stamp.",
                step.as_str()
            )),
            Err(get_criteria::Error::NoOpenStep { current: None }) => {
                refuse("No step takes a report: every step of this project is done.")
            }
            Err(get_criteria::Error::Port(error)) => unavailable(error),
        }
    }

    #[tool(description = "Report pass or fail for each criterion of the open step. When every criterion \
            passes, include a handoff note.")]
    async fn send_report(
        &self,
        Extension(player): Extension<PlayerId>,
        Parameters(request): Parameters<ReportRequest>,
    ) -> Reply {
        let (project, step) = match (id(request.project, "project"), id(request.step, "step")) {
            (Ok(project), Ok(step)) => (project, step),
            (Err(reply), _) | (_, Err(reply)) => return reply,
        };
        let mut verdicts = Vec::new();
        for verdict in request.verdicts {
            match id(verdict.criterion, "criterion") {
                Ok(criterion) => {
                    verdicts.push((criterion, if verdict.pass { Verdict::Pass } else { Verdict::Fail }))
                }
                Err(reply) => return reply,
            }
        }
        let report =
            SubmitReport { step, verdicts, handoff_note: request.handoff_note.and_then(HandoffNote::new) };
        match self.grado.submit_report(submit_report::Input { player, project, report }).await {
            Ok(submit_report::Output { submitted }) => json(&match submitted {
                Submitted::StepStillOpen { failed } => ReportResponse {
                    state: "open",
                    next_action: format!(
                        "The report is recorded. Stop and tell the person which criteria failed: {}. \
                         Don't fix them on your own; send a new report once the person has worked on them.",
                        list(&failed)
                    ),
                    failed: failed.iter().map(|criterion| criterion.as_str().into()).collect(),
                    next_step: None,
                },
                Submitted::StepApproved => ReportResponse {
                    state: "approved",
                    failed: Vec::new(),
                    next_step: None,
                    next_action: "The step is approved. Call get_stamp and give the person the stamp.".into(),
                },
                Submitted::StepDone { next: Some(next) } => ReportResponse {
                    state: "done",
                    failed: Vec::new(),
                    next_action: format!(
                        "The step is done and step {} is open. Suggest a fresh agent for it, or continue \
                         here if the person wants; either way, start with get_briefing.",
                        next.as_str()
                    ),
                    next_step: Some(next.as_str().into()),
                },
                Submitted::StepDone { next: None } => ReportResponse {
                    state: "done",
                    failed: Vec::new(),
                    next_step: None,
                    next_action: "The step is done, and so is the project. Tell the person.".into(),
                },
            }),
            Err(submit_report::Error::UnknownProject(project)) => unknown_project(&project),
            Err(submit_report::Error::NotStarted(project)) => not_started(&project),
            Err(submit_report::Error::UnknownStep(step)) => unknown_step(&step),
            Err(submit_report::Error::StepNotOpen { open: Some(open) }) => {
                refuse(format!("That step doesn't take a report. Only step {} does now.", open.as_str()))
            }
            Err(submit_report::Error::StepNotOpen { open: None }) => {
                refuse("No step takes a report now. Call get_briefing to see where the project stands.")
            }
            Err(submit_report::Error::CriteriaMismatch { expected }) => refuse(format!(
                "Send exactly one verdict for each criterion of the step: {}.",
                list(&expected)
            )),
            Err(submit_report::Error::HandoffNoteMissing) => refuse(
                "Every criterion passed, so the step needs a handoff note: a short summary for the agent \
                 that takes the next step. Send the report again with one.",
            ),
            Err(submit_report::Error::Conflict) => refuse(
                "Another connection changed this project at the same time, so the report wasn't recorded. \
                 Call get_briefing, then send the report again if it still applies.",
            ),
            Err(submit_report::Error::Port(error)) => unavailable(error),
        }
    }

    #[tool(
        description = "A stamp for an approved step, for the person to enter in the Grado app. A new call \
            voids the previous stamp."
    )]
    async fn get_stamp(
        &self,
        Extension(player): Extension<PlayerId>,
        Parameters(request): Parameters<StampRequest>,
    ) -> Reply {
        let (project, step) = match (id(request.project, "project"), id(request.step, "step")) {
            (Ok(project), Ok(step)) => (project, step),
            (Err(reply), _) | (_, Err(reply)) => return reply,
        };
        match self.grado.issue_stamp(issue_stamp::Input { player, project, step }).await {
            Ok(issue_stamp::Output { code }) => json(&StampResponse {
                stamp: code.as_str().into(),
                next_action: "Give the person this stamp exactly as written; they enter it in the Grado app.",
            }),
            Err(issue_stamp::Error::UnknownProject(project)) => unknown_project(&project),
            Err(issue_stamp::Error::NotStarted(project)) => not_started(&project),
            Err(issue_stamp::Error::UnknownStep(step)) => unknown_step(&step),
            Err(issue_stamp::Error::NotApproved { pending }) => refuse(format!(
                "The step isn't approved yet. These criteria still need a passing report: {}.",
                list(&pending)
            )),
            Err(issue_stamp::Error::StampOff) => {
                refuse("This project doesn't use stamps: a step is done as soon as its report passes.")
            }
            Err(issue_stamp::Error::Port(error)) => unavailable(error),
        }
    }

    /// The player the request's connection acts for (E8).
    async fn player(&self, token: Option<AgentToken>) -> Result<PlayerId, Reply> {
        let Some(token) = token else {
            return Err(no_connection());
        };
        match self.grado.authenticate_agent_connection(authenticate_agent_connection::Input { token }).await {
            Ok(output) => {
                tracing::Span::current().record("player", output.player.as_str());
                Ok(output.player)
            }
            Err(authenticate_agent_connection::Error::NoActiveConnection) => Err(no_connection()),
            Err(authenticate_agent_connection::Error::Port(error)) => Err(unavailable(error)),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for Tools {
    /// Runs every tool call in one span: its request id, the tool's name and, once
    /// authenticated, the player. The connection is authenticated here, once, before any tool
    /// runs; a tool reads the player it acts for as `Extension<PlayerId>` (ADR 0017).
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        mut context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let parts = context.extensions.get::<Parts>();
        let request_id = parts
            .and_then(|parts| parts.headers.get(REQUEST_ID))
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default()
            .to_owned();
        let token = parts.and_then(token);
        let span = tracing::info_span!("tool_call", request_id, tool = %request.name, player = tracing::field::Empty);
        async move {
            let player = match self.player(token).await {
                Ok(player) => player,
                Err(reply) => return reply.map(CallToolResponse::from),
            };
            context.extensions.insert(player);
            self.tool_router.call(ToolCallContext::new(self, request, context)).await
        }
        .instrument(span)
        .await
    }

    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("grado", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }
}

/// The agent connection's token, from the request's `Authorization: Bearer` header.
fn token(parts: &Parts) -> Option<AgentToken> {
    parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .and_then(AgentToken::new)
}

/// Reads an id the agent sent; a blank one is refused.
fn id<T>(value: String, name: &str) -> Result<Id<T>, Reply> {
    Id::new(value).ok_or_else(|| refuse(format!("The {name} id is blank.")))
}

fn json(response: &impl Serialize) -> Reply {
    let value = serde_json::to_value(response).map_err(|error| {
        tracing::error!(cause = %error, "a response couldn't be encoded");
        ErrorData::internal_error("the response couldn't be encoded", None)
    })?;
    Ok(CallToolResult::structured(value))
}

fn refuse(message: impl Into<String>) -> Reply {
    Ok(CallToolResult::error(vec![ContentBlock::text(message)]))
}

/// The one place a port failure is logged. The caller sees none of its cause.
fn unavailable(error: PortError) -> Reply {
    tracing::error!(cause = %error, "a port failed");
    Err(ErrorData::internal_error("Grado is unavailable right now", None))
}

fn no_connection() -> Reply {
    refuse(
        "This connection is unknown or was revoked. Ask the person to create a new agent connection in \
         the Grado app.",
    )
}

fn unknown_project(project: &ProjectId) -> Reply {
    refuse(format!(
        "There's no project {}. Call list_my_projects for the ones the person started.",
        project.as_str()
    ))
}

fn not_started(project: &ProjectId) -> Reply {
    refuse(format!(
        "The person hasn't started project {}. They start projects in the Grado app.",
        project.as_str()
    ))
}

fn unknown_step(step: &StepId) -> Reply {
    refuse(format!("This project has no step {}. Call get_briefing for the step to work on.", step.as_str()))
}

fn list(criteria: &[CriterionId]) -> String {
    criteria.iter().map(|criterion| criterion.as_str()).collect::<Vec<_>>().join(", ")
}

fn state(state: StepState) -> &'static str {
    match state {
        StepState::Locked => "locked",
        StepState::Open => "open",
        StepState::Approved => "approved",
        StepState::Done => "done",
        StepState::Skipped => "skipped",
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::Mutex;

    use super::*;

    /// Keeps what a subscriber writes, to read it back.
    #[derive(Clone, Default)]
    struct Written(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Written {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Written {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[test]
    fn a_port_failure_is_logged_once_with_its_cause_and_the_caller_sees_none_of_it() {
        let written = Written::default();
        let subscriber = tracing_subscriber::fmt().with_writer(written.clone()).with_ansi(false).finish();

        let reply = tracing::subscriber::with_default(subscriber, || {
            unavailable(PortError::unavailable("connection refused"))
        });

        let log = String::from_utf8(written.0.lock().unwrap().clone()).unwrap();
        assert_eq!(log.matches("a port failed").count(), 1, "{log}");
        assert!(log.contains("connection refused"), "{log}");
        let error = reply.expect_err("a protocol error");
        assert_eq!(error.message, "Grado is unavailable right now");
    }
}
