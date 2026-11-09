//! Local server, and the composition root for it (ADR 0010): it chooses the adapters, builds
//! `Grado` from them, and serves the MCP tools.
//!
//! Environment:
//! - `GRADO_DEV_AGENT_TOKEN` (required): the token of the sample player's agent connection. Dev
//!   only; STU-29 replaces it.
//! - `GRADO_DEV_STAMP`: `on` to complete steps at their stamp; anything else, or unset, completes
//!   them at approval.
//! - `GRADO_ADDR`: the address to listen on, `127.0.0.1:8080` when unset.

use std::env;
use std::sync::Arc;

use application::{AgentToken, Grado};
use domain::stamp::StampSetting;
use outbound_memory::sample::sample;

const DEFAULT_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let token = env::var("GRADO_DEV_AGENT_TOKEN")
        .ok()
        .and_then(AgentToken::new)
        .expect("set GRADO_DEV_AGENT_TOKEN to the token the agent connects with");
    let stamp = match env::var("GRADO_DEV_STAMP").as_deref() {
        Ok("on") => StampSetting::On,
        _ => StampSetting::Off,
    };
    let addr = env::var("GRADO_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.into());

    let sample = sample(token, stamp).await;
    let grado = Grado::new(
        Arc::new(sample.catalog),
        Arc::new(sample.progress),
        Arc::new(sample.settings),
        Arc::new(sample.connections),
        Arc::new(sample.stamp_codes),
    );
    let router = inbound_mcp::router(Arc::new(grado));

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Grado MCP server on http://{addr}{}", inbound_mcp::PATH);
    axum::serve(listener, router).await
}
