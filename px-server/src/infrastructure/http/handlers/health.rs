use crate::infrastructure::bootstrap::app_state::AppState;
use crate::infrastructure::http::dto::HealthDto;
use axum::Json;
use axum::extract::State;
use px_errors::AppError;
use px_types::SingleResponse;
use std::time::SystemTime;

pub async fn handle(
    State(state): State<AppState>,
) -> Result<Json<SingleResponse<HealthDto>>, AppError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let uptime_secs = now.saturating_sub(state.start_unix);
    let body = HealthDto {
        status: "ok",
        build_sha: state.build_sha,
        uptime_secs,
    };
    Ok(Json(SingleResponse::new(body, "healthy")))
}
