//! The player's JSON API, used by the website.

use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use grado_domain::secrets;
use grado_store::{AccessToken, Idea, RedeemOutcome};

use super::auth::CurrentUser;
use super::error::{ApiError, ApiResult};
use crate::markdown;
use crate::state::AppState;
use crate::trail::{GuideView, LevelSummary, LevelView};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/journey", get(journey))
        .route("/api/levels/{id}", get(level))
        .route("/api/levels/{id}/stamp", post(redeem))
        .route("/api/levels/{id}/guides/next", post(next_guide))
        .route("/api/ideas", get(ideas))
        .route("/api/connect", get(connect))
        .route("/api/tokens", post(create_token))
        .route("/api/tokens/{id}", delete(revoke_token))
}

#[derive(Serialize)]
struct JourneyResponse {
    levels: Vec<LevelSummary>,
    finished: bool,
    content_version: String,
}

async fn journey(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<Json<JourneyResponse>> {
    let levels = state.trail.journey_summary(&user).await?;
    let finished = levels.iter().all(|l| l.status == grado_domain::LevelStatus::Completed);
    Ok(Json(JourneyResponse { levels, finished, content_version: state.trail.catalog.version.clone() }))
}

async fn level(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> ApiResult<Json<LevelView>> {
    Ok(Json(state.trail.level_view(&user, &id).await?))
}

#[derive(Deserialize)]
struct RedeemBody {
    stamp: String,
}

async fn redeem(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
    Json(body): Json<RedeemBody>,
) -> ApiResult<Json<Value>> {
    let outcome = state.trail.redeem_stamp(&user, &id, &body.stamp).await?;
    let message = match outcome {
        RedeemOutcome::Redeemed => "Stamped! The next checkpoint is open.",
        RedeemOutcome::AlreadyCompleted => "This checkpoint is already stamped.",
        RedeemOutcome::Expired => {
            "That stamp expired. Ask your coach to finish the checkpoint again for a fresh one."
        }
        RedeemOutcome::Invalid => {
            "That stamp doesn't match this checkpoint. Check for typos, or ask your coach for it again."
        }
    };
    let next = state.trail.catalog.next_level(&id).map(|l| l.id.clone());
    Ok(Json(json!({ "outcome": outcome, "message": message, "next_level_id": next })))
}

async fn next_guide(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> ApiResult<Json<GuideView>> {
    let guide = state
        .trail
        .open_next_guide(&user, &id)
        .await?
        .ok_or_else(|| ApiError::bad_request("You've opened every guide for this checkpoint."))?;
    Ok(Json(GuideView {
        tier: guide.tier,
        title: guide.title.clone(),
        html: Some(markdown::to_html(&guide.body_md)),
    }))
}

async fn ideas(State(state): State<AppState>, CurrentUser(user): CurrentUser) -> ApiResult<Json<Vec<Idea>>> {
    Ok(Json(state.trail.store.ideas(&user.id).await?))
}

#[derive(Serialize)]
struct ConnectInfo {
    mcp_url: String,
    server_name: String,
    tokens: Vec<AccessToken>,
}

fn mcp_url(state: &AppState) -> String {
    format!("{}/mcp", state.config.public_url)
}

fn server_name(state: &AppState) -> String {
    state.trail.catalog.vars.get("site.mcp_server_name").cloned().unwrap_or_else(|| "grado".into())
}

async fn connect(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> ApiResult<Json<ConnectInfo>> {
    Ok(Json(ConnectInfo {
        mcp_url: mcp_url(&state),
        server_name: server_name(&state),
        tokens: state.trail.store.access_tokens(&user.id).await?,
    }))
}

#[derive(Deserialize)]
struct NewToken {
    label: Option<String>,
}

async fn create_token(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(body): Json<NewToken>,
) -> ApiResult<Json<Value>> {
    let token = secrets::access_token();
    let label = body.label.filter(|l| !l.trim().is_empty()).unwrap_or_else(|| "Claude Code".into());
    let record =
        state.trail.store.create_access_token(&user.id, label.trim(), &secrets::hash(&token)).await?;
    let command = format!(
        "claude mcp add --scope user --transport http {} {} --header \"Authorization: Bearer {}\"",
        server_name(&state),
        mcp_url(&state),
        token
    );
    Ok(Json(json!({ "token": token, "record": record, "command": command })))
}

async fn revoke_token(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let revoked = state.trail.store.revoke_access_token(&user.id, &id).await?;
    Ok(Json(json!({ "revoked": revoked })))
}
