#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use px_cache::{CookieCache, InMemoryCookieCache};
use px_core::{CacheKey, PxAppId, PxCookieBundle};
use std::time::{Duration, SystemTime};

fn key(domain: &str, app: &str, fp: u64) -> CacheKey {
    CacheKey::new(domain, PxAppId::new(app).expect("valid app id"), fp)
}

fn bundle(ttl_secs: u64) -> PxCookieBundle {
    PxCookieBundle::new(
        vec![],
        "UA",
        SystemTime::now(),
        Duration::from_secs(ttl_secs),
    )
}

#[tokio::test]
async fn put_then_get_returns_bundle() {
    let cache = InMemoryCookieCache::new();
    let k = key("pedidosya.com.ar", "eT15wiaE", 1);
    cache.put(k.clone(), bundle(60)).await.expect("put");
    let got = cache.get(&k).await.expect("get");
    assert!(got.is_some());
    let metrics = cache.metrics_snapshot();
    assert_eq!(metrics.puts, 1);
    assert_eq!(metrics.hits, 1);
    assert_eq!(metrics.entries, 1);
}

#[tokio::test]
async fn get_missing_key_is_miss() {
    let cache = InMemoryCookieCache::new();
    let k = key("x.com", "eT15wiaE", 1);
    assert!(cache.get(&k).await.expect("get").is_none());
    assert_eq!(cache.metrics_snapshot().misses, 1);
}

#[tokio::test]
async fn expired_bundle_is_evicted_on_get() {
    let cache = InMemoryCookieCache::new();
    let k = key("x.com", "eT15wiaE", 1);
    cache.put(k.clone(), bundle(0)).await.expect("put");
    std::thread::sleep(Duration::from_millis(1100));
    let got = cache.get(&k).await.expect("get");
    assert!(got.is_none());
    let m = cache.metrics_snapshot();
    assert_eq!(m.expired_evictions, 1);
    assert_eq!(m.misses, 1);
    assert_eq!(m.entries, 0);
}

#[tokio::test]
async fn invalidate_clears_only_matching_domain() {
    let cache = InMemoryCookieCache::new();
    let a = key("a.com", "eT15wiaE", 1);
    let b = key("b.com", "eT15wiaE", 1);
    cache.put(a.clone(), bundle(60)).await.expect("put a");
    cache.put(b.clone(), bundle(60)).await.expect("put b");
    cache.invalidate("a.com").await.expect("invalidate");
    assert!(cache.get(&a).await.expect("get a").is_none());
    assert!(cache.get(&b).await.expect("get b").is_some());
    assert_eq!(cache.metrics_snapshot().invalidations, 1);
}

#[tokio::test]
async fn purge_expired_returns_count() {
    let cache = InMemoryCookieCache::new();
    cache
        .put(key("a.com", "eT15wiaE", 1), bundle(0))
        .await
        .expect("put");
    cache
        .put(key("b.com", "eT15wiaE", 1), bundle(0))
        .await
        .expect("put");
    std::thread::sleep(Duration::from_millis(1100));
    let n = cache.purge_expired();
    assert_eq!(n, 2);
}
