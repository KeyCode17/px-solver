use crate::application::solve_endpoint::SolveDispatcher;
use crate::infrastructure::bootstrap::server_metrics::ServerMetrics;
use px_auth::{AuditSink, CheckAllowlist, VerifyKey};
use px_cache::CookieCache;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub verify_key: Arc<VerifyKey>,
    pub check_allowlist: Arc<CheckAllowlist>,
    pub dispatcher: Arc<dyn SolveDispatcher>,
    pub cache: Arc<dyn CookieCache>,
    pub audit: Arc<dyn AuditSink>,
    pub build_sha: &'static str,
    pub start_unix: u64,
    pub metrics: Arc<ServerMetrics>,
}

pub struct AppStateConfig {
    pub verify_key: Arc<VerifyKey>,
    pub check_allowlist: Arc<CheckAllowlist>,
    pub dispatcher: Arc<dyn SolveDispatcher>,
    pub cache: Arc<dyn CookieCache>,
    pub audit: Arc<dyn AuditSink>,
    pub build_sha: &'static str,
}

impl AppState {
    pub fn new(cfg: AppStateConfig) -> Self {
        let start_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            verify_key: cfg.verify_key,
            check_allowlist: cfg.check_allowlist,
            dispatcher: cfg.dispatcher,
            cache: cfg.cache,
            audit: cfg.audit,
            build_sha: cfg.build_sha,
            start_unix,
            metrics: Arc::new(ServerMetrics::default()),
        }
    }
}
