use crate::infrastructure::bootstrap::app_state::AppState;
use crate::infrastructure::http::handlers::{fetch, health, metrics, solve};
use axum::Router;
use axum::routing::{get, post};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v1/solve", post(solve::handle))
        .route("/v1/fetch", post(fetch::handle))
        .route("/v1/health", get(health::handle))
        .route("/v1/metrics", get(metrics::handle))
        .with_state(state)
}
