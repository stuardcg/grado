//! Grado server: the website, its JSON API, and the MCP endpoint, all on
//! one axum router so the same binary runs locally or behind a serverless
//! adapter.

pub mod config;
pub mod grader;
pub mod http;
pub mod markdown;
pub mod mcp;
pub mod state;
pub mod trail;

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderValue, header};
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use grado_domain::{Catalog, OfflineRubricGrader, RubricGrader, Vars};
use grado_store::{SqliteStore, Store};

use crate::config::Config;
use crate::grader::ClaudeRubricGrader;
use crate::state::AppState;
use crate::trail::Trail;

/// Loads content, opens the database and wires the grader.
pub async fn build_state(config: Config) -> anyhow::Result<AppState> {
    let overrides = Vars::from([("site.url".to_owned(), config.public_url.clone())]);
    let catalog = Arc::new(Catalog::load_with(&config.content_dir, overrides)?);
    let store: Arc<dyn Store> = Arc::new(SqliteStore::connect(&config.database_url).await?);
    let grader: Arc<dyn RubricGrader> = match &config.grader.anthropic_api_key {
        Some(key) => Arc::new(ClaudeRubricGrader::new(&config.grader, key.clone())),
        None => {
            tracing::warn!("ANTHROPIC_API_KEY not set: rubric checks use the offline keyword grader");
            Arc::new(OfflineRubricGrader)
        }
    };
    tracing::info!(levels = catalog.levels().len(), version = %catalog.version, "content loaded");
    let trail = Arc::new(Trail::new(catalog, store, grader, config.stamp_ttl_hours));
    Ok(AppState { trail, config: Arc::new(config) })
}

pub fn router(state: AppState, cancel: CancellationToken) -> Router {
    let mcp = Router::new()
        .nest_service("/mcp", mcp::service(state.trail.clone(), state.config.allowed_hosts.clone(), cancel))
        .layer(axum::middleware::from_fn_with_state(state.clone(), mcp::require_bearer));

    let web_dir = &state.config.web_dir;
    // `no-cache` still allows conditional requests; it just makes browsers
    // revalidate, so a deploy is picked up on the next load.
    let site = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache"),
        ))
        .service(ServeDir::new(web_dir).not_found_service(ServeFile::new(web_dir.join("index.html"))));

    Router::new()
        .merge(http::auth::routes())
        .merge(http::api::routes())
        .merge(http::admin::routes())
        .with_state(state)
        .merge(mcp)
        .fallback_service(site)
        .layer(TraceLayer::new_for_http())
}
