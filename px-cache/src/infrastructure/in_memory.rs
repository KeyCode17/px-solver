use crate::domain::cookie_cache::CookieCache;
use crate::domain::metrics::CacheMetrics;
use async_trait::async_trait;
use dashmap::DashMap;
use px_core::{CacheKey, PxCookieBundle};
use px_errors::AppError;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

#[derive(Default)]
struct Counters {
    hits: AtomicU64,
    misses: AtomicU64,
    puts: AtomicU64,
    expired_evictions: AtomicU64,
    invalidations: AtomicU64,
}

pub struct InMemoryCookieCache {
    entries: Arc<DashMap<CacheKey, PxCookieBundle>>,
    counters: Arc<Counters>,
}

impl InMemoryCookieCache {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(DashMap::new()),
            counters: Arc::new(Counters::default()),
        }
    }

    pub fn purge_expired(&self) -> u64 {
        let now = SystemTime::now();
        let stale: Vec<CacheKey> = self
            .entries
            .iter()
            .filter(|e| e.value().is_expired_at(now))
            .map(|e| e.key().clone())
            .collect();
        let count = stale.len() as u64;
        for k in stale {
            self.entries.remove(&k);
        }
        if count > 0 {
            self.counters
                .expired_evictions
                .fetch_add(count, Ordering::Relaxed);
        }
        count
    }
}

impl Default for InMemoryCookieCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CookieCache for InMemoryCookieCache {
    async fn get(&self, key: &CacheKey) -> Result<Option<PxCookieBundle>, AppError> {
        let now = SystemTime::now();
        let found = self.entries.get(key).map(|v| v.value().clone());
        match found {
            Some(bundle) if !bundle.is_expired_at(now) => {
                self.counters.hits.fetch_add(1, Ordering::Relaxed);
                Ok(Some(bundle))
            }
            Some(_) => {
                self.entries.remove(key);
                self.counters
                    .expired_evictions
                    .fetch_add(1, Ordering::Relaxed);
                self.counters.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
            None => {
                self.counters.misses.fetch_add(1, Ordering::Relaxed);
                Ok(None)
            }
        }
    }

    async fn put(&self, key: CacheKey, bundle: PxCookieBundle) -> Result<(), AppError> {
        self.entries.insert(key, bundle);
        self.counters.puts.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn invalidate(&self, domain: &str) -> Result<(), AppError> {
        let keys: Vec<CacheKey> = self
            .entries
            .iter()
            .filter(|e| e.key().domain == domain)
            .map(|e| e.key().clone())
            .collect();
        let count = keys.len() as u64;
        for k in keys {
            self.entries.remove(&k);
        }
        self.counters
            .invalidations
            .fetch_add(count, Ordering::Relaxed);
        Ok(())
    }

    fn metrics_snapshot(&self) -> CacheMetrics {
        CacheMetrics {
            hits: self.counters.hits.load(Ordering::Relaxed),
            misses: self.counters.misses.load(Ordering::Relaxed),
            puts: self.counters.puts.load(Ordering::Relaxed),
            expired_evictions: self.counters.expired_evictions.load(Ordering::Relaxed),
            invalidations: self.counters.invalidations.load(Ordering::Relaxed),
            entries: self.entries.len(),
        }
    }
}
