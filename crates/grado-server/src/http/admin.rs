//! The game master's view: who is where, and every piece of evidence submitted.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{Value, json};
use grado_store::{Attempt, PlayerSummary};

use super::auth::AdminUser;
use super::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/admin/players", get(players)).route("/api/admin/players/{id}", get(player))
}

async fn players(State(state): State<AppState>, _: AdminUser) -> ApiResult<Json<Vec<PlayerSummary>>> {
    Ok(Json(state.trail.store.players().await?))
}

async fn player(
    State(state): State<AppState>,
    _: AdminUser,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let store = &state.trail.store;
    let user = store
        .user(&id)
        .await?
        .ok_or_else(|| ApiError(axum::http::StatusCode::NOT_FOUND, "No such player.".into()))?;
    let journey = state.trail.journey_summary(&user).await?;
    let attempts: Vec<Attempt> = store.attempts(&id, None).await?;
    let ideas = store.ideas(&id).await?;
    let mut handoffs = serde_json::Map::new();
    for level in state.trail.catalog.levels() {
        if let Some(body) = store.handoff(&id, &level.id).await? {
            handoffs.insert(level.id.clone(), Value::String(body));
        }
    }
    Ok(Json(
        json!({ "user": user, "journey": journey, "attempts": attempts, "ideas": ideas, "handoffs": handoffs }),
    ))
}
