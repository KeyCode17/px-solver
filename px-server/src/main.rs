use anyhow::{Context, Result};
use px_auth::{
    AllowlistStore, CheckAllowlist, StdoutAuditSink, VerifyKey, YamlAllowlistStore, YamlKeyStore,
};
use px_cache::InMemoryCookieCache;
use px_harvester::{ChromiumoxidePool, Harvester, PoolConfig};
use px_perimeterx::PerimeterxHandler;
use px_pipeline::ChallengeHandler;
use px_server::application::routing::parse_camoufox_domains;
use px_server::infrastructure::bootstrap::dispatchers::build_dispatchers;
use px_server::{AppState, AppStateConfig, build_router};
use std::collections::BTreeSet;
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
    let allowlist_store: Arc<dyn AllowlistStore> = Arc::new(
        YamlAllowlistStore::load(&allowlist_path)
            .await
            .with_context(|| format!("load {allowlist_path}"))?,
    );

    let harvester: Arc<dyn Harvester> = Arc::new(ChromiumoxidePool::new(PoolConfig::default()));
    let px_handler: Arc<dyn ChallengeHandler> =
        Arc::new(PerimeterxHandler::new(Arc::clone(&harvester)));

    let cf_domains = resolve_cf_domains(allowlist_store.as_ref()).await?;
    let dispatchers = build_dispatchers(px_handler, cf_domains)?;

    let state = AppState::new(AppStateConfig {
        verify_key: Arc::new(VerifyKey::new(Arc::new(key_store))),
        check_allowlist: Arc::new(CheckAllowlist::new(Arc::clone(&allowlist_store))),
        dispatcher: dispatchers.solve,
        fetch_dispatcher: dispatchers.fetch,
        cache: Arc::new(InMemoryCookieCache::new()),
        audit: Arc::new(StdoutAuditSink::new()),
        build_sha: env!("CARGO_PKG_VERSION"),
    });

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&bind)
        .await
        .with_context(|| format!("bind {bind}"))?;
    tracing::info!(%bind, "px-server listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("axum serve")?;
    tracing::info!("px-server stopped cleanly");
    Ok(())
}

/// Resolves when the process should begin a graceful shutdown.
///
/// Listens for SIGINT (Ctrl-C from a TTY) and, on Unix, SIGTERM (sent by
/// systemd, Kubernetes, and `kill`). The first signal wins. Axum's
/// `with_graceful_shutdown` then stops accepting new connections and lets
/// in-flight `/v1/solve` requests finish before returning.
async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::warn!(error = %e, "ctrl_c signal handler install failed");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(e) => {
                tracing::warn!(error = %e, "SIGTERM handler install failed");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("SIGINT received, draining in-flight solves"),
        _ = terminate => tracing::info!("SIGTERM received, draining in-flight solves"),
    }
}

/// Collect the set of domains that should route through the Cloudflare
/// (Camoufox) handler. Sources, unioned and lowercased:
///
/// 1. `allowlist.yaml` entries with `handler: cloudflare` (preferred per
///    ADR-0023 — configuration lives next to the operator's review).
/// 2. The deprecated `PX_CAMOUFOX_DOMAINS` env CSV (kept as a fallback so
///    operators can roll out v1.0.x → v1.1.x without touching the YAML;
///    removal planned for v2.x).
async fn resolve_cf_domains(store: &dyn AllowlistStore) -> Result<Vec<String>> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for e in store.list().await.context("list allowlist")? {
        if e.handler.as_deref().map(str::to_lowercase).as_deref() == Some("cloudflare") {
            set.insert(e.domain.to_lowercase());
        }
    }
    for d in parse_camoufox_domains(env::var("PX_CAMOUFOX_DOMAINS").ok().as_deref()) {
        set.insert(d);
    }
    Ok(set.into_iter().collect())
}
