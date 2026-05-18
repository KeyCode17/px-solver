use crate::application::solve_endpoint::domain_from_url;
use crate::infrastructure::bootstrap::app_state::AppState;
use crate::infrastructure::http::dto::{FetchRequestDto, FetchResponseDto};
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use px_auth::{AuditEvent, AuditOutcome};
use px_errors::AppError;
use px_pipeline::FetchRequest;
use px_types::SingleResponse;
use std::sync::atomic::Ordering;
use std::time::{Instant, SystemTime};

pub async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<FetchRequestDto>,
) -> Result<Json<SingleResponse<FetchResponseDto>>, AppError> {
    let started = Instant::now();
    let key_id = verify_authorization(&state, &headers)
        .await
        .inspect_err(|_| {
            state.metrics.auth_denied.fetch_add(1, Ordering::Relaxed);
        })?;
    let domain = domain_from_url(&payload.url)?;
    state
        .check_allowlist
        .execute(&domain)
        .await
        .inspect_err(|_| {
            state
                .metrics
                .allowlist_denied
                .fetch_add(1, Ordering::Relaxed);
        })?;

    let req = FetchRequest {
        url: payload.url.clone(),
        method: payload.method,
        headers: payload.headers,
        body: payload.body,
        timeout_ms: payload.timeout_ms.unwrap_or(15_000),
    };
    let dispatched = state.fetch_dispatcher.dispatch(req).await;
    let (handler_name, response) = match dispatched {
        Ok(v) => v,
        Err(e) => {
            record_audit(
                &state,
                key_id.clone(),
                domain.clone(),
                started,
                AuditOutcome::HandlerFailed,
                None,
            )
            .await?;
            return Err(e);
        }
    };
    record_audit(
        &state,
        key_id,
        domain,
        started,
        AuditOutcome::Solved,
        Some(handler_name.clone()),
    )
    .await?;
    let body = FetchResponseDto {
        status: response.status,
        headers: response.headers,
        body: response.body,
        duration_ms: response.duration_ms,
        handler: handler_name,
    };
    Ok(Json(SingleResponse::new(body, "fetched")))
}

async fn record_audit(
    state: &AppState,
    key_id: String,
    target_domain: String,
    started: Instant,
    outcome: AuditOutcome,
    handler: Option<String>,
) -> Result<(), AppError> {
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let now_unix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let event = AuditEvent {
        timestamp_unix: now_unix,
        key_id,
        target_domain,
        outcome,
        latency_ms: elapsed_ms,
        handler,
    };
    state.audit.record(&event).await
}

async fn verify_authorization(state: &AppState, headers: &HeaderMap) -> Result<String, AppError> {
    let raw = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;
    let token = raw.strip_prefix("Bearer ").ok_or_else(|| {
        AppError::Unauthorized("Authorization must be 'Bearer <id>:<secret>'".into())
    })?;
    let (id, secret) = token
        .split_once(':')
        .ok_or_else(|| AppError::Unauthorized("Bearer token must be 'id:secret'".into()))?;
    state.verify_key.execute(id, secret).await?;
    Ok(id.to_string())
}
