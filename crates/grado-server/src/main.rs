use tokio_util::sync::CancellationToken;
use tracing_subscriber::EnvFilter;
use grado_server::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=info".into()))
        .init();

    let config = Config::from_env()?;
    let bind = config.bind.clone();
    let public_url = config.public_url.clone();
    let state = grado_server::build_state(config).await?;

    let cancel = CancellationToken::new();
    let app = grado_server::router(state, cancel.clone());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!("Grado running at {public_url}");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            cancel.cancel();
        })
        .await?;
    Ok(())
}
