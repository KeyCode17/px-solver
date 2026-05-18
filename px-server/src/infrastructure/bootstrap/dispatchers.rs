//! Builds the `SolveDispatcher` and the `FetchDispatcher` that the
//! `/v1/solve` and `/v1/fetch` HTTP handlers depend on.
//!
//! Both dispatchers share the same `CamoufoxPool` (when one is built)
//! so `harvest` and `fetch` against a Cloudflare-fronted target reuse
//! the same browser config. The pool is wrapped in `Arc` and handed to
//! the role-specific traits (`Harvester` for solve, `Fetcher` for fetch).

use crate::application::fetch_endpoint::{FetchDispatcher, RoutingFetchDispatcher};
use crate::application::routing::RoutingDispatcher;
use crate::application::solve_endpoint::{PxSolveDispatcher, SolveDispatcher};
use anyhow::{Context, Result};
use px_camoufox::{CamoufoxConfig, CamoufoxPool};
use px_cloudflare::CloudflareHandler;
use px_harvester::Harvester;
use px_pipeline::{ChallengeHandler, Fetcher};
use std::sync::Arc;

pub struct Dispatchers {
    pub solve: Arc<dyn SolveDispatcher>,
    pub fetch: Arc<dyn FetchDispatcher>,
}

pub fn build_dispatchers(
    default_handler: Arc<dyn ChallengeHandler>,
    cf_domains: Vec<String>,
) -> Result<Dispatchers> {
    if cf_domains.is_empty() {
        return Ok(Dispatchers {
            solve: Arc::new(PxSolveDispatcher::new(default_handler)),
            fetch: Arc::new(RoutingFetchDispatcher::new(None)),
        });
    }

    let cfg = CamoufoxConfig::from_env();
    if let Err(e) = cfg.validate() {
        tracing::warn!(
            error = %e,
            domains = ?cf_domains,
            "Cloudflare routes configured but Camoufox unavailable; falling back to Chromium-only solve dispatcher (no /v1/fetch)"
        );
        return Ok(Dispatchers {
            solve: Arc::new(PxSolveDispatcher::new(default_handler)),
            fetch: Arc::new(RoutingFetchDispatcher::new(None)),
        });
    }

    let pool = Arc::new(CamoufoxPool::new(cfg).context("build CamoufoxPool")?);
    let harvester: Arc<dyn Harvester> = Arc::clone(&pool) as Arc<dyn Harvester>;
    let fetcher: Arc<dyn Fetcher> = Arc::clone(&pool) as Arc<dyn Fetcher>;
    let cf_handler: Arc<dyn ChallengeHandler> =
        Arc::new(CloudflareHandler::with_harvester(harvester));

    let mut solve_router = RoutingDispatcher::new(default_handler);
    let mut fetch_router = RoutingFetchDispatcher::new(None);
    for d in &cf_domains {
        solve_router = solve_router.with_route(d.clone(), Arc::clone(&cf_handler));
        fetch_router = fetch_router.with_route(d.clone(), "cloudflare", Arc::clone(&fetcher));
    }
    tracing::info!(domains = ?cf_domains, "Camoufox routing enabled (solve + fetch)");
    Ok(Dispatchers {
        solve: Arc::new(solve_router),
        fetch: Arc::new(fetch_router),
    })
}
