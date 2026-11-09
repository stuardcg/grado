//! Turns MCP tool calls into use-case calls, over stateless Streamable HTTP.

mod bindings;

use std::sync::Arc;

use application::Grado;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};

use crate::bindings::Tools;

/// The path MCP clients connect to.
pub const PATH: &str = "/mcp";

/// An HTTP router serving the MCP tools at [`PATH`]. Every request stands alone: no MCP session,
/// and each answer is one JSON body.
pub fn router(grado: Arc<Grado>) -> axum::Router {
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_sse_keep_alive(None);
    let service = StreamableHttpService::new(
        move || Ok(Tools::new(grado.clone())),
        Arc::new(NeverSessionManager::default()),
        config,
    );
    axum::Router::new().nest_service(PATH, service)
}
