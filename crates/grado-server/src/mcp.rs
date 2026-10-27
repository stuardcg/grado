//! The MCP endpoint Claude Code connects to.
//!
//! Every request carries a personal access token (`Authorization: Bearer`).
//! [`require_bearer`] resolves it to a player before rmcp sees the request,
//! and each tool reads that player from the request extensions. Tools only
//! ever act on the player's current checkpoint.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::Extension;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, PromptMessage, Role, ServerCapabilities, ServerConfig,
};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{
    ErrorData, ServerHandler, prompt, prompt_handler, prompt_router, tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;
use grado_domain::{EvidenceKind, secrets};
use grado_store::User;

use crate::state::AppState;
use crate::trail::{CurrentCheck, Trail, TrailError};

const INSTRUCTIONS: &str = "Grado coaching server. At the start of every session call get_briefing and follow it: \
    it contains who you are coaching, the rules for talking to her, and the current checkpoint.";

/// The authenticated player for this MCP request.
#[derive(Clone)]
pub struct Player(pub User);

pub async fn require_bearer(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim);
    let user = match token {
        Some(token) => state.trail.store.access_token_user(&secrets::hash(token)).await,
        None => Ok(None),
    };
    match user {
        Ok(Some(user)) => {
            request.extensions_mut().insert(Player(user));
            next.run(request).await
        }
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            "Missing or revoked access token. Create one on the Grado website under Connect.",
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "token lookup failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub fn service(
    trail: Arc<Trail>,
    allowed_hosts: Vec<String>,
    cancel: CancellationToken,
) -> StreamableHttpService<TrailMcp, LocalSessionManager> {
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .with_allowed_hosts(allowed_hosts)
        .with_cancellation_token(cancel);
    StreamableHttpService::new(move || Ok(TrailMcp::new(trail.clone())), Default::default(), config)
}

#[derive(Clone)]
pub struct TrailMcp {
    trail: Arc<Trail>,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

fn player(parts: &http::request::Parts) -> Result<User, ErrorData> {
    parts
        .extensions
        .get::<Player>()
        .map(|p| p.0.clone())
        .ok_or_else(|| ErrorData::invalid_request("not authenticated", None))
}

fn text(message: impl Into<String>) -> Result<CallToolResult, ErrorData> {
    Ok(CallToolResult::success(vec![ContentBlock::text(message.into())]))
}

/// Rule violations go back to the agent as a tool error it can read and act
/// on; only storage failures become protocol errors.
fn outcome<T>(
    result: Result<T, TrailError>,
    render: impl FnOnce(T) -> String,
) -> Result<CallToolResult, ErrorData> {
    match result {
        Ok(value) => text(render(value)),
        Err(TrailError::Store(error)) => {
            tracing::error!(%error, "storage error in MCP tool");
            Err(ErrorData::internal_error("storage error", None))
        }
        Err(other) => Ok(CallToolResult::error(vec![ContentBlock::text(other.to_string())])),
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SubmitEvidence {
    /// The `criterion_id` from get_current_check.
    pub criterion_id: String,
    /// For output checks: the raw, unedited terminal output. For text checks: her words.
    pub evidence: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SharedFile {
    /// File name, e.g. `registrations.csv`. An unknown name lists the available files.
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ParkIdea {
    /// The idea, in a sentence, close to her words.
    pub idea: String,
    /// Checkpoint number where it belongs, if you know it.
    pub belongs_to_checkpoint: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FinishCheckpoint {
    /// 5–8 lines written with her: what she built, where it lives, decisions, open questions.
    pub handoff: String,
}

#[tool_router]
impl TrailMcp {
    pub fn new(trail: Arc<Trail>) -> Self {
        Self { trail, tool_router: Self::tool_router(), prompt_router: Self::prompt_router() }
    }

    #[tool(
        description = "Read this first in every session. Returns the coach instructions, the current checkpoint briefing, the previous handoff and her parked ideas."
    )]
    async fn get_briefing(
        &self,
        Extension(parts): Extension<http::request::Parts>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = player(&parts)?;
        outcome(self.trail.briefing(&user).await, |b| b)
    }

    #[tool(
        description = "The check she is working on now: what to collect and how to submit it. Later checks stay hidden."
    )]
    async fn get_current_check(
        &self,
        Extension(parts): Extension<http::request::Parts>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = player(&parts)?;
        outcome(self.trail.current_check(&user).await, |check| match check {
            CurrentCheck::Pending(view) => format!(
                "Check {} of {}: {}\ncriterion_id: {}\nevidence: {}\n\n{}",
                view.position,
                view.total,
                view.title,
                view.criterion_id,
                match view.evidence {
                    EvidenceKind::Output => "raw output from running commands in her project",
                    EvidenceKind::Text => "her own words",
                },
                view.ask
            ),
            CurrentCheck::AllPassed { level_id } => format!(
                "Every check for {level_id} has passed. Write the handoff note with her, then call finish_checkpoint."
            ),
            CurrentCheck::Finished => "She has finished every checkpoint.".into(),
        })
    }

    #[tool(
        description = "Submit evidence for the current check. The server judges it and returns pass/fail with feedback that names what is missing without giving the answer."
    )]
    async fn submit_evidence(
        &self,
        Extension(parts): Extension<http::request::Parts>,
        Parameters(input): Parameters<SubmitEvidence>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = player(&parts)?;
        outcome(self.trail.submit_evidence(&user, &input.criterion_id, &input.evidence).await, |result| {
            let progress = format!("{} of {} checks passed.", result.checks_passed, result.checks_total);
            if result.passed {
                let next = if result.checks_passed == result.checks_total {
                    "That was the last one: write the handoff note with her, then call finish_checkpoint."
                } else {
                    "Call get_current_check for the next one."
                };
                format!("PASSED. {}\n{progress} {next}", result.feedback)
            } else {
                format!(
                    "NOT YET. {}\n{progress} Coach her toward the gap, then submit again.",
                    result.feedback
                )
            }
        })
    }

    #[tool(
        description = "Fetch a sample data file or event document to save into her project, e.g. registrations.csv, riders_small.csv, event_rules.md."
    )]
    async fn get_shared_file(
        &self,
        Parameters(input): Parameters<SharedFile>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.trail.catalog.shared_file(&input.name) {
            Some(content) => text(content),
            None => {
                let names: Vec<&str> = self.trail.catalog.shared_file_names().collect();
                Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                    "No file called {}. Available: {}",
                    input.name,
                    names.join(", ")
                ))]))
            }
        }
    }

    #[tool(
        description = "Open the next, more detailed guide for the current checkpoint when she is stuck. Each call opens one more tier; she can also see opened guides on the website."
    )]
    async fn get_guide(
        &self,
        Extension(parts): Extension<http::request::Parts>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = player(&parts)?;
        let level_id = match self.trail.current_level(&user).await {
            Ok(Some(level)) => level.id.clone(),
            Ok(None) => return text("She has finished every checkpoint; there are no guides left."),
            Err(error) => return outcome::<()>(Err(error), |_| String::new()),
        };
        outcome(self.trail.open_next_guide(&user, &level_id).await, |guide| match guide {
            Some(g) => format!("Guide {} (opened for her too):\n\n{}", g.tier, g.body_md),
            None => "Every guide for this checkpoint is already open. Coach her directly.".into(),
        })
    }

    #[tool(
        description = "Save an idea that belongs to a later checkpoint, so she keeps it without leaving the current one."
    )]
    async fn park_idea(
        &self,
        Extension(parts): Extension<http::request::Parts>,
        Parameters(input): Parameters<ParkIdea>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = player(&parts)?;
        outcome(self.trail.park_idea(&user, &input.idea, input.belongs_to_checkpoint).await, |idea| {
            format!("Parked: \"{}\". She can see it on the website.", idea.body)
        })
    }

    #[tool(
        description = "Call after every check passed. Saves the handoff note for the next checkpoint and returns the stamp she enters on the website."
    )]
    async fn finish_checkpoint(
        &self,
        Extension(parts): Extension<http::request::Parts>,
        Parameters(input): Parameters<FinishCheckpoint>,
    ) -> Result<CallToolResult, ErrorData> {
        let user = player(&parts)?;
        let url = self.trail.catalog.vars.get("site.url").cloned().unwrap_or_default();
        outcome(self.trail.finish_checkpoint(&user, &input.handoff).await, |stamp| {
            format!(
                "Checkpoint \"{}\" complete. Her stamp: {}\nShe enters it at {url} within {} hours to unlock the next checkpoint. Give it to her exactly as written.",
                stamp.level_title, stamp.stamp, stamp.expires_in_hours
            )
        })
    }
}

#[prompt_router]
impl TrailMcp {
    #[prompt(
        name = "checkpoint",
        description = "Start a coaching session for your current Grado checkpoint."
    )]
    async fn checkpoint(
        &self,
        Extension(parts): Extension<http::request::Parts>,
    ) -> Result<Vec<PromptMessage>, ErrorData> {
        let user = player(&parts)?;
        let briefing =
            self.trail.briefing(&user).await.map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(vec![PromptMessage::new_text(
            Role::User,
            format!(
                "{briefing}\n\n---\n\nStart the session now, following the session flow above. You already have the briefing; no need to call get_briefing again."
            ),
        )])
    }
}

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
impl ServerHandler for TrailMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().enable_prompts().build())
            .with_server_info(Implementation::new("grado", env!("CARGO_PKG_VERSION")))
            .with_instructions(INSTRUCTIONS)
    }
}
