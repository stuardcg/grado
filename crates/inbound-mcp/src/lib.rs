//! Turns MCP tool calls into use-case calls, over stateless Streamable HTTP.

mod bindings;

use std::sync::Arc;

use application::Grado;
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};

use crate::bindings::Tools;

/// The path MCP clients connect to.
pub const PATH: &str = "/mcp";
/// The header carrying a request's id, kept from the request or made new, and sent back.
const REQUEST_ID: &str = "x-request-id";

/// An HTTP router serving the MCP tools at [`PATH`]. Every request stands alone: no MCP session,
/// and each answer is one JSON body. Every request has an id in [`REQUEST_ID`] (ADR 0012).
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
    axum::Router::new()
        .nest_service(PATH, service)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}
