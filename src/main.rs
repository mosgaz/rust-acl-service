use anyhow::Context;
use rust_acl_service::app::build_router;
use rust_acl_service::config::Config;
use rust_acl_service::db::Db;
use rust_acl_service::metrics::init_metrics;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Config::from_env().context("failed to load config")?;
    let db = Db::connect(&config.database_url)
        .await
        .context("failed to connect database")?;

    db.run_migrations()
        .await
        .context("failed to run migrations")?;

    let metrics_handle = init_metrics(&config.metrics_addr)?;
    let app = build_router(db.pool.clone(), metrics_handle);

    let listener = TcpListener::bind(&config.http_addr)
        .await
        .with_context(|| format!("failed to bind {}", config.http_addr))?;

    info!(http_addr = %config.http_addr, "acl-service started");
    axum::serve(listener, app).await.context("server crashed")?;
    Ok(())
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,rust_acl_service=debug,sqlx=warn".into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .flatten_event(true)
        .init();
}
