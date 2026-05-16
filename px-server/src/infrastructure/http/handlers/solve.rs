use crate::application::solve_endpoint::{SolveOutput, domain_from_url, sentinel_cache_key};
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
    let cache_key = sentinel_cache_key(&domain)?;
    let cached = state.cache.get(&cache_key).await?;
    let (out, cache_hit) = if let Some(bundle) = cached {
        (
            SolveOutput {
                user_agent: bundle.user_agent.clone(),
                bundle,
                solve_ms: 0,
                cache_hit: true,
                handler: "cache".into(),
            },
            true,
        )
    } else {
        let result = state.dispatcher.solve(&payload.url).await;
        let solved = match result {
            Ok(v) => v,
            Err(e) => {
                state.metrics.solves_failed.fetch_add(1, Ordering::Relaxed);
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
        state
            .cache
            .put(cache_key.clone(), solved.bundle.clone())
            .await?;
        (solved, false)
    };
    record_audit(
        &state,
        key_id,
        domain,
        started,
        AuditOutcome::Solved,
        Some(out.handler.clone()),
    )
    .await?;
    let body = SolveResponseDto {
        user_agent: out.user_agent,
        solve_ms: out.solve_ms,
        cache_hit,
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
