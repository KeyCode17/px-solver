//! Egress-IP rotation for Camoufox sessions.
//!
//! Pedidosya's PerimeterX (and most aggressive WAFs) rate-limit by
//! source IP. When the operator wants sustained scraping above what a
//! single IP can sustain (~10-30 req/min before flagging), each
//! `PersistentSession` should attach to a different egress proxy.
//!
//! Config is operator-side: a list of proxy URLs supplied via env or
//! by the embedder. `ProxyPool::next` round-robins through the list;
//! a `None` return signals "go direct" so the operator can opt out
//! by not setting any proxies.

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct ProxyPool {
    proxies: Vec<String>,
    cursor: AtomicUsize,
}

impl ProxyPool {
    pub fn new(proxies: Vec<String>) -> Self {
        // Strip blank entries so a trailing comma in `PX_PROXIES`
        // doesn't poison the rotation.
        let proxies = proxies
            .into_iter()
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();
        Self {
            proxies,
            cursor: AtomicUsize::new(0),
        }
    }

    /// Parse `PX_PROXIES` CSV into a pool. Unset / empty → empty pool
    /// (direct connection).
    pub fn from_env(var: &str) -> Self {
        let raw = std::env::var(var).unwrap_or_default();
        let entries = raw.split(',').map(|s| s.to_string()).collect();
        Self::new(entries)
    }

    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }

    pub fn len(&self) -> usize {
        self.proxies.len()
    }

    /// Next proxy by round-robin order. `None` when the pool is empty —
    /// the caller treats that as "no proxy, use the default route".
    pub fn next(&self) -> Option<String> {
        if self.proxies.is_empty() {
            return None;
        }
        let idx = self.cursor.fetch_add(1, Ordering::Relaxed) % self.proxies.len();
        Some(self.proxies[idx].clone())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn empty_pool_returns_none() {
        let pool = ProxyPool::new(vec![]);
        assert!(pool.is_empty());
        assert!(pool.next().is_none());
    }

    #[test]
    fn round_robins_across_entries() {
        let pool = ProxyPool::new(vec![
            "http://a:1".into(),
            "http://b:2".into(),
            "http://c:3".into(),
        ]);
        let picks: Vec<String> = (0..6).filter_map(|_| pool.next()).collect();
        assert_eq!(
            picks,
            vec![
                "http://a:1",
                "http://b:2",
                "http://c:3",
                "http://a:1",
                "http://b:2",
                "http://c:3",
            ]
        );
    }

    #[test]
    fn trims_blank_and_whitespace_entries() {
        let pool = ProxyPool::new(vec![
            "  http://a:1 ".into(),
            "".into(),
            "   ".into(),
            "http://b:2".into(),
        ]);
        assert_eq!(pool.len(), 2);
        assert_eq!(pool.next().as_deref(), Some("http://a:1"));
        assert_eq!(pool.next().as_deref(), Some("http://b:2"));
    }
}
