//! Local server, and the composition root for it (ADR 0010): it loads the configuration, chooses
//! the adapters, builds `Grado` from them, and serves the MCP tools. Every setting is listed in
//! `grado.example.toml` (ADR 0011).

use std::process;
use std::sync::Arc;

use app::config::Config;
use application::Grado;
use outbound_memory::sample::sample;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = Config::from_env().unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });

    let sample = sample(config.dev.agent_token, config.dev.stamp).await;
    let grado = Grado::new(
        Arc::new(sample.catalog),
        Arc::new(sample.progress),
        Arc::new(sample.settings),
        Arc::new(sample.connections),
        Arc::new(sample.stamp_codes),
    );
    let router = inbound_mcp::router(Arc::new(grado));

    let listener = tokio::net::TcpListener::bind(config.addr).await?;
    println!("Grado MCP server on http://{}{}", config.addr, inbound_mcp::PATH);
    axum::serve(listener, router).await
}
