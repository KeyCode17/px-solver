use anyhow::{Context, Result};
use px_auth::{CheckAllowlist, StdoutAuditSink, VerifyKey, YamlAllowlistStore, YamlKeyStore};
use px_cache::InMemoryCookieCache;
use px_camoufox::{CamoufoxConfig, CamoufoxPool};
use px_cloudflare::CloudflareHandler;
use px_harvester::{ChromiumoxidePool, Harvester, PoolConfig};
use px_perimeterx::PerimeterxHandler;
use px_pipeline::ChallengeHandler;
use px_server::application::routing::{RoutingDispatcher, parse_camoufox_domains};
use px_server::application::solve_endpoint::{PxSolveDispatcher, SolveDispatcher};
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

    let harvester: Arc<dyn Harvester> = Arc::new(ChromiumoxidePool::new(PoolConfig::default()));
    let px_handler: Arc<dyn ChallengeHandler> =
        Arc::new(PerimeterxHandler::new(Arc::clone(&harvester)));

    let dispatcher = build_dispatcher(px_handler)?;

    let state = AppState::new(AppStateConfig {
        verify_key: Arc::new(VerifyKey::new(Arc::new(key_store))),
        check_allowlist: Arc::new(CheckAllowlist::new(Arc::new(allowlist_store))),
        dispatcher,
        cache: Arc::new(InMemoryCookieCache::new()),
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

/// Build the SolveDispatcher. If `PX_CAMOUFOX_DOMAINS` lists at least one
/// domain AND the Camoufox + geckodriver binaries validate at the configured
/// paths, returns a RoutingDispatcher that maps those domains to a
/// CloudflareHandler backed by a CamoufoxPool. Otherwise returns the
/// default Chromium-only dispatcher.
fn build_dispatcher(
    default_handler: Arc<dyn ChallengeHandler>,
) -> Result<Arc<dyn SolveDispatcher>> {
    let raw = env::var("PX_CAMOUFOX_DOMAINS").ok();
    let domains = parse_camoufox_domains(raw.as_deref());
    if domains.is_empty() {
        return Ok(Arc::new(PxSolveDispatcher::new(default_handler)));
    }

    let cfg = CamoufoxConfig::from_env();
    if let Err(e) = cfg.validate() {
        tracing::warn!(
            error = %e,
            "PX_CAMOUFOX_DOMAINS set but Camoufox unavailable; falling back to Chromium-only dispatcher"
        );
        return Ok(Arc::new(PxSolveDispatcher::new(default_handler)));
    }

    let pool = CamoufoxPool::new(cfg).context("build CamoufoxPool")?;
    let camoufox: Arc<dyn Harvester> = Arc::new(pool);
    let cf_handler: Arc<dyn ChallengeHandler> =
        Arc::new(CloudflareHandler::with_harvester(camoufox));

    let mut router = RoutingDispatcher::new(default_handler);
    for d in &domains {
        router = router.with_route(d.clone(), Arc::clone(&cf_handler));
    }
    tracing::info!(domains = ?domains, "Camoufox routing enabled");
    Ok(Arc::new(router))
}
