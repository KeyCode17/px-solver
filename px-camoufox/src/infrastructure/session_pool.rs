//! Registry of `PersistentSession`s, keyed by target domain.
//!
//! Lazy spawn on first request; entries are recycled once they exceed
//! the configured TTL. The pool does not bound concurrent acquires —
//! one session per domain is the throttle, since callers serialize on
//! that session's mutex.

use crate::domain::config::CamoufoxConfig;
use crate::infrastructure::session::PersistentSession;
use px_errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub(crate) struct SessionPool {
    config: CamoufoxConfig,
    sessions: Mutex<HashMap<String, Arc<PersistentSession>>>,
    ttl: Duration,
}

impl SessionPool {
    pub(crate) fn new(config: CamoufoxConfig, ttl: Duration) -> Self {
        Self {
            config,
            sessions: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    /// Returns the warm session for `domain`, spawning a new one (and
    /// evicting any aged-out predecessor) when needed. Spawning a fresh
    /// browser holds the registry lock only briefly: the actual
    /// geckodriver launch happens with the registry unlocked so other
    /// domains can be acquired in parallel.
    pub(crate) async fn acquire(&self, domain: &str) -> Result<Arc<PersistentSession>, AppError> {
        {
            let mut guard = self.sessions.lock().await;
            if let Some(existing) = guard.get(domain)
                && !existing.is_aged_out(self.ttl)
            {
                return Ok(Arc::clone(existing));
            }
            if guard.contains_key(domain) {
                tracing::info!(%domain, "recycling aged-out Camoufox session");
                guard.remove(domain);
            }
        }
        let fresh = Arc::new(PersistentSession::spawn(&self.config, domain).await?);
        let mut guard = self.sessions.lock().await;
        // Double-check: another task may have raced and inserted a fresh
        // entry while we were spawning. Prefer the existing one to avoid
        // leaking the just-spawned browser.
        if let Some(existing) = guard.get(domain)
            && !existing.is_aged_out(self.ttl)
        {
            return Ok(Arc::clone(existing));
        }
        guard.insert(domain.into(), Arc::clone(&fresh));
        Ok(fresh)
    }

    #[allow(dead_code)]
    pub(crate) async fn len(&self) -> usize {
        self.sessions.lock().await.len()
    }
}
