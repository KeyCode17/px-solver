//! Registry of `PersistentSession`s, keyed by target domain.
//!
//! For each domain we hold up to `max_per_domain` warm browsers, picked
//! round-robin per acquire. That lets `/v1/fetch` parallelize against
//! the same target while each session still holds the upstream's
//! coherent cookie jar.

use crate::domain::config::CamoufoxConfig;
use crate::infrastructure::proxy_pool::ProxyPool;
use crate::infrastructure::session::PersistentSession;
use crate::infrastructure::synthetic_user::SyntheticUserPool;
use px_errors::AppError;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Default)]
struct DomainSlot {
    sessions: Vec<Arc<PersistentSession>>,
    /// Round-robin cursor for `pick`.
    cursor: AtomicUsize,
}

pub(crate) struct SessionPool {
    config: CamoufoxConfig,
    domains: Mutex<HashMap<String, DomainSlot>>,
    ttl: Duration,
    max_per_domain: usize,
    proxies: Arc<ProxyPool>,
    users: Arc<SyntheticUserPool>,
}

impl SessionPool {
    pub(crate) fn new(
        config: CamoufoxConfig,
        ttl: Duration,
        max_per_domain: usize,
        proxies: Arc<ProxyPool>,
        users: Arc<SyntheticUserPool>,
    ) -> Self {
        Self {
            config,
            domains: Mutex::new(HashMap::new()),
            ttl,
            max_per_domain: max_per_domain.max(1),
            proxies,
            users,
        }
    }

    /// Acquire a session for `domain`. If the slot has room, lazily
    /// spawns a new browser; otherwise rotates round-robin across the
    /// existing sessions. Aged-out sessions are dropped on the way.
    pub(crate) async fn acquire(&self, domain: &str) -> Result<Arc<PersistentSession>, AppError> {
        {
            let mut guard = self.domains.lock().await;
            let slot = guard
                .entry(domain.to_string())
                .or_insert_with(DomainSlot::default);
            let before = slot.sessions.len();
            slot.sessions.retain(|s| !s.is_aged_out(self.ttl));
            if slot.sessions.len() < before {
                tracing::info!(
                    %domain,
                    evicted = before - slot.sessions.len(),
                    "evicted aged-out Camoufox sessions"
                );
            }
            if slot.sessions.len() >= self.max_per_domain {
                let idx = slot.cursor.fetch_add(1, Ordering::Relaxed) % slot.sessions.len();
                return Ok(Arc::clone(&slot.sessions[idx]));
            }
        }
        let proxy = self.proxies.next();
        let user = self.users.next();
        let fresh =
            Arc::new(PersistentSession::spawn(&self.config, domain, proxy.as_deref(), user).await?);
        let mut guard = self.domains.lock().await;
        let slot = guard
            .entry(domain.to_string())
            .or_insert_with(DomainSlot::default);
        if slot.sessions.len() < self.max_per_domain {
            slot.sessions.push(Arc::clone(&fresh));
            return Ok(fresh);
        }
        // Race: somebody else filled the slot while we spawned. Pick
        // one of the existing sessions; the just-spawned browser drops
        // here (kill_on_drop fires when `fresh`'s Arc count hits 0).
        let idx = slot.cursor.fetch_add(1, Ordering::Relaxed) % slot.sessions.len();
        Ok(Arc::clone(&slot.sessions[idx]))
    }

    #[allow(dead_code)]
    pub(crate) async fn total_sessions(&self) -> usize {
        self.domains
            .lock()
            .await
            .values()
            .map(|slot| slot.sessions.len())
            .sum()
    }
}
