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
    out
}
