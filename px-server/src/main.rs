use anyhow::{Context, Result};
use px_auth::{CheckAllowlist, StdoutAuditSink, VerifyKey, YamlAllowlistStore, YamlKeyStore};
use px_harvester::{ChromiumoxidePool, PoolConfig};
use px_perimeterx::PerimeterxHandler;
use px_server::application::solve_endpoint::PxSolveDispatcher;
use px_server::{AppState, AppStateConfig, build_router};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let bind = env::var("PX_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let keys_path = env::var("PX_KEYS").unwrap_or_else(|_| "config/keys.yaml".into());
    let allowlist_path =
        env::var("PX_ALLOWLIST").unwrap_or_else(|_| "config/allowlist.yaml".into());

    let key_store = YamlKeyStore::load(&keys_path)
        .await
        .with_context(|| format!("load {keys_path}"))?;
    let allowlist_store = YamlAllowlistStore::load(&allowlist_path)
        .await
        .with_context(|| format!("load {allowlist_path}"))?;

    let harvester = Arc::new(ChromiumoxidePool::new(PoolConfig::default()));
    let handler = Arc::new(PerimeterxHandler::new(harvester));
    let dispatcher = Arc::new(PxSolveDispatcher::new(handler));

    let state = AppState::new(AppStateConfig {
        verify_key: Arc::new(VerifyKey::new(Arc::new(key_store))),
        check_allowlist: Arc::new(CheckAllowlist::new(Arc::new(allowlist_store))),
        dispatcher,
        audit: Arc::new(StdoutAuditSink::new()),
        build_sha: env!("CARGO_PKG_VERSION"),
    });

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("bind {bind}"))?;
    tracing::info!(%bind, "px-server listening");
    axum::serve(listener, app).await.context("axum serve")?;
    Ok(())
}
