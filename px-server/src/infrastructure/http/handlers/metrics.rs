use crate::infrastructure::bootstrap::app_state::AppState;
use axum::extract::State;
use std::sync::atomic::Ordering;

pub async fn handle(State(state): State<AppState>) -> String {
    let m = &state.metrics;
    let mut out = String::new();
    out.push_str("# HELP px_solves_total Total /v1/solve requests\n");
    out.push_str("# TYPE px_solves_total counter\n");
    out.push_str(&format!(
        "px_solves_total {}\n",
        m.solves_total.load(Ordering::Relaxed)
    ));
    out.push_str("# HELP px_solves_failed Total failed solves\n");
    out.push_str("# TYPE px_solves_failed counter\n");
    out.push_str(&format!(
        "px_solves_failed {}\n",
        m.solves_failed.load(Ordering::Relaxed)
    ));
    out.push_str("# HELP px_auth_denied Total auth-denied requests\n");
    out.push_str("# TYPE px_auth_denied counter\n");
    out.push_str(&format!(
        "px_auth_denied {}\n",
        m.auth_denied.load(Ordering::Relaxed)
    ));
    out.push_str("# HELP px_allowlist_denied Total allowlist-denied requests\n");
    out.push_str("# TYPE px_allowlist_denied counter\n");
    out.push_str(&format!(
        "px_allowlist_denied {}\n",
        m.allowlist_denied.load(Ordering::Relaxed)
    ));
    let c = state.cache.metrics_snapshot();
    out.push_str("# HELP px_cache_hits_total Total cache hits\n");
    out.push_str("# TYPE px_cache_hits_total counter\n");
    out.push_str(&format!("px_cache_hits_total {}\n", c.hits));
    out.push_str("# HELP px_cache_misses_total Total cache misses\n");
    out.push_str("# TYPE px_cache_misses_total counter\n");
    out.push_str(&format!("px_cache_misses_total {}\n", c.misses));
    out.push_str("# HELP px_cache_entries Cache live entries\n");
    out.push_str("# TYPE px_cache_entries gauge\n");
    out.push_str(&format!("px_cache_entries {}\n", c.entries));
    out.push_str("# HELP px_cache_hit_ratio Cache hit ratio (0.0..=1.0)\n");
    out.push_str("# TYPE px_cache_hit_ratio gauge\n");
    out.push_str(&format!("px_cache_hit_ratio {:.4}\n", c.hit_ratio()));
    out
}
