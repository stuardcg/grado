//! Browser sign-in. v1 ships a development sign-in (name + email, no
//! password). A real identity provider plugs in here: it ends the same way,
//! with `start_session` for a verified email.

use axum::extract::{FromRef, FromRequestParts, State};
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue};
use axum::response::IntoResponse;
use axum::{Json, Router, routing::get, routing::post};
use serde::{Deserialize, Serialize};
use grado_domain::secrets;
use grado_store::{User, now};

use super::error::{ApiError, ApiResult};
use crate::config::AuthMode;
use crate::state::AppState;

const COOKIE_NAME: &str = "grado_session";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/dev-login", post(dev_login))
        .route("/api/auth/logout", post(logout))
        .route("/api/me", get(me))
}

/// The signed-in player, from the session cookie.
pub struct CurrentUser(pub User);

impl<S> FromRequestParts<S> for CurrentUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let session = session_cookie(&parts.headers).ok_or_else(ApiError::unauthorized)?;
        let user = state.trail.store.session_user(&secrets::hash(&session)).await?;
        user.map(CurrentUser).ok_or_else(ApiError::unauthorized)
    }
}

/// A signed-in player listed in `GRADO_ADMIN_EMAILS`.
pub struct AdminUser(#[allow(dead_code)] pub User);

impl<S> FromRequestParts<S> for AdminUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;
        let state = AppState::from_ref(state);
        if state.config.is_admin(&user.email) { Ok(AdminUser(user)) } else { Err(ApiError::forbidden()) }
    }
}

fn session_cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, _)| *name == COOKIE_NAME)
        .map(|(_, value)| value.to_owned())
}

#[derive(Deserialize)]
struct DevLogin {
    name: String,
    email: String,
}

async fn dev_login(
    State(state): State<AppState>,
    Json(body): Json<DevLogin>,
) -> ApiResult<impl IntoResponse> {
    if state.config.auth_mode != AuthMode::Dev {
        return Err(ApiError::forbidden());
    }
    let (name, email) = (body.name.trim(), body.email.trim());
    if name.is_empty() || !email.contains('@') {
        return Err(ApiError::bad_request("Enter your name and an email address."));
    }
    let user = state.trail.store.upsert_user(email, name).await?;
    let cookie = start_session(&state, &user).await?;
    Ok(([(SET_COOKIE, cookie)], Json(user)))
}

/// Creates a session for an already-verified user and returns its cookie.
pub async fn start_session(state: &AppState, user: &User) -> ApiResult<HeaderValue> {
    let session = secrets::session_id();
    let ttl = state.config.session_ttl_days * 86_400;
    state.trail.store.create_session(&user.id, &secrets::hash(&session), now() + ttl).await?;
    let secure = if state.config.public_url.starts_with("https://") { "; Secure" } else { "" };
    let cookie = format!("{COOKIE_NAME}={session}; Path=/; HttpOnly; SameSite=Lax; Max-Age={ttl}{secure}");
    HeaderValue::from_str(&cookie).map_err(|_| ApiError::bad_request("bad cookie"))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> ApiResult<impl IntoResponse> {
    if let Some(session) = session_cookie(&headers) {
        state.trail.store.delete_session(&secrets::hash(&session)).await?;
    }
    let expired = HeaderValue::from_static("grado_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0");
    Ok(([(SET_COOKIE, expired)], Json(serde_json::json!({ "ok": true }))))
}

#[derive(Serialize)]
struct Me {
    user: User,
    is_admin: bool,
}

async fn me(State(state): State<AppState>, CurrentUser(user): CurrentUser) -> Json<Me> {
    let is_admin = state.config.is_admin(&user.email);
    Json(Me { user, is_admin })
}
