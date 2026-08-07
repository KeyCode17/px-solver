use async_trait::async_trait;
use px_core::{CacheKey, PxAppId, PxCookieBundle, SolveRequest};
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerStatus, PageHtml, SolveAction};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use url::Url;

#[async_trait]
pub trait SolveDispatcher: Send + Sync {
    async fn solve(&self, req: SolveRequest) -> Result<SolveOutput, AppError>;
}

#[derive(Debug, Clone)]
pub struct SolveOutput {
    pub bundle: PxCookieBundle,
    pub user_agent: String,
    pub solve_ms: u64,
    pub cache_hit: bool,
    pub handler: String,
}

pub struct PxSolveDispatcher {
    handler: Arc<dyn ChallengeHandler>,
    handler_name: &'static str,
}

impl PxSolveDispatcher {
    pub fn new(handler: Arc<dyn ChallengeHandler>) -> Self {
        let handler_name = handler.name();
        Self {
            handler,
            handler_name,
        }
    }
}

#[async_trait]
impl SolveDispatcher for PxSolveDispatcher {
    async fn solve(&self, req: SolveRequest) -> Result<SolveOutput, AppError> {
        let action = SolveAction::new(PageHtml::new(&req.url, "")).with_proxy(req.proxy);
        let outcome = self.handler.solve(&action).await?;
        if !matches!(outcome.status, HandlerStatus::Solved) {
            return Err(AppError::Conflict(format!(
                "{} returned status {:?}",
                self.handler_name, outcome.status
            )));
        }
        let user_agent = outcome.user_agent.clone().unwrap_or_default();
        let bundle = PxCookieBundle::new(
            outcome.cookies.set.clone(),
            user_agent.clone(),
            SystemTime::now(),
            Duration::from_secs(600),
        );
        Ok(SolveOutput {
            bundle,
            user_agent,
            solve_ms: outcome.metrics.solve_ms,
            cache_hit: false,
            handler: outcome.handler,
        })
    }
}

pub fn domain_from_url(url: &str) -> Result<String, AppError> {
    let parsed = Url::parse(url).map_err(|e| AppError::BadRequest(format!("invalid url: {e}")))?;
    parsed
        .host_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::BadRequest("url has no host".into()))
}

pub fn cache_key_for(domain: &str, app_id: PxAppId, fp_key: u64) -> CacheKey {
    CacheKey::new(domain, app_id, fp_key)
}

/// Cache key for one solve. The egress is part of the identity: PX binds a
/// `_px3` bundle to the IP that earned it, so replaying a proxy-A bundle
/// for a request that asked for proxy B (or for a direct route) hands the
/// caller cookies their traffic will not match.
pub fn sentinel_cache_key(domain: &str, proxy: Option<&str>) -> Result<CacheKey, AppError> {
    let app_id = PxAppId::new("Unknown000")
        .map_err(|e| AppError::InternalError(format!("sentinel app_id invalid: {e}")))?;
    Ok(CacheKey::new(domain, app_id, egress_key(proxy)))
}

/// `0` for a direct route, so keys minted before proxies were honoured
/// keep resolving to the same entry.
fn egress_key(proxy: Option<&str>) -> u64 {
    match proxy {
        None => 0,
        Some(proxy) => {
            let mut hasher = DefaultHasher::new();
            proxy.hash(&mut hasher);
            hasher.finish()
        }
    }
}
