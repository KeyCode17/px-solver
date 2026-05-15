use crate::application::solve_endpoint::domain_from_url;
use crate::infrastructure::bootstrap::app_state::AppState;
use crate::infrastructure::http::dto::{CookieDto, SolveRequestDto, SolveResponseDto};
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use px_auth::{AuditEvent, AuditOutcome};
use px_errors::AppError;
use px_types::SingleResponse;
use std::sync::atomic::Ordering;
use std::time::{Instant, SystemTime};

pub async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SolveRequestDto>,
) -> Result<Json<SingleResponse<SolveResponseDto>>, AppError> {
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
    state.metrics.solves_total.fetch_add(1, Ordering::Relaxed);
    let result = state.dispatcher.solve(&payload.url).await;
    let outcome = match &result {
        Ok(_) => AuditOutcome::Solved,
        Err(_) => {
            state.metrics.solves_failed.fetch_add(1, Ordering::Relaxed);
            AuditOutcome::HandlerFailed
        }
    };
    let elapsed_ms = started.elapsed().as_millis() as u64;
    let now_unix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let handler_name = result.as_ref().ok().map(|r| r.handler.clone());
    let event = AuditEvent {
        timestamp_unix: now_unix,
        key_id,
        target_domain: domain,
        outcome,
        latency_ms: elapsed_ms,
        handler: handler_name,
    };
    state.audit.record(&event).await?;
    let out = result?;
    let body = SolveResponseDto {
        user_agent: out.user_agent,
        solve_ms: out.solve_ms,
        cache_hit: out.cache_hit,
        handler: out.handler,
        cookies: out
            .bundle
            .cookies
            .iter()
            .map(|c| CookieDto {
                name: c.name.clone(),
                value: c.value.clone(),
                domain: c.domain.clone(),
                path: c.path.clone(),
            })
            .collect(),
        expires_at: out.bundle.expires_at,
    };
    Ok(Json(SingleResponse::new(body, "solved")))
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
