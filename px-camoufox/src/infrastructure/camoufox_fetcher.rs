//! `Fetcher` impl on `CamoufoxPool`. Reuses a per-domain persistent
//! Camoufox session so the geckodriver/browser cold-start (~13s)
//! happens once per domain, PerimeterX sees a coherent browser
//! session instead of a stream of throwaway ones, and `cf_clearance`
//! issued during the first navigation is reused.
//!
//! GET routes through `navigate_and_read` so the browser handles
//! interactive Cloudflare challenges; POST routes through
//! `in_page_fetch` because webdriver navigation can't carry a body.
//! See `fetch_strategies` for both.

use crate::infrastructure::camoufox_pool::CamoufoxPool;
use crate::infrastructure::fetch_strategies::{
    in_page_fetch, navigate_and_read, navigate_with_wait,
};
use crate::infrastructure::session::PersistentSession;
use async_trait::async_trait;
use px_errors::AppError;
use px_pipeline::{FetchRequest, FetchResponse, Fetcher};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[async_trait]
impl Fetcher for CamoufoxPool {
    async fn fetch(&self, req: FetchRequest) -> Result<FetchResponse, AppError> {
        let domain = domain_of(&req.url)?;
        let session = self.sessions.acquire(&domain).await?;
        let navigate_timeout = self.config.navigate_timeout;
        let request_timeout = Duration::from_millis(req.timeout_ms);
        run_through_session(session, req, navigate_timeout, request_timeout).await
    }
}

async fn run_through_session(
    session: Arc<PersistentSession>,
    req: FetchRequest,
    navigate_timeout: Duration,
    request_timeout: Duration,
) -> Result<FetchResponse, AppError> {
    let started = Instant::now();
    let mut inner = session.inner.lock().await;

    // Warm the browser once per session: load the origin so Cloudflare's
    // managed challenge JS runs and `cf_clearance` lands in the jar.
    if !inner.warmed {
        let origin = origin_of(&req.url)?;
        if let Err(e) = navigate_with_wait(
            &inner.client,
            &origin,
            navigate_timeout,
            Duration::from_millis(2_500),
        )
        .await
        {
            tracing::warn!(error = %e, "session warmup failed; downstream will likely 403");
        } else {
            inner.warmed = true;
            inner.last_visited = Some(origin);
        }
    }
    // Navigate to the referer only when it actually differs from where
    // the webdriver is currently parked. Saves a full 1-2s nav per
    // fetch when consecutive callers share the same SPA page.
    if let Some(referer) = referer_header(&req)
        && !referer.is_empty()
        && inner.last_visited.as_deref() != Some(referer.as_str())
        && navigate_with_wait(
            &inner.client,
            &referer,
            navigate_timeout,
            Duration::from_millis(1_000),
        )
        .await
        .is_ok()
    {
        inner.last_visited = Some(referer);
    }

    let outcome = if req.method().eq_ignore_ascii_case("GET") {
        navigate_and_read(&inner.client, &req, navigate_timeout).await
    } else {
        in_page_fetch(&inner.client, &req, request_timeout).await
    };
    if req.method().eq_ignore_ascii_case("GET") {
        inner.last_visited = Some(req.url.clone());
    }
    inner.last_used = Instant::now();
    inner.fetch_count = inner.fetch_count.saturating_add(1);
    drop(inner);

    let (status, headers, body) = outcome?;
    Ok(FetchResponse {
        status,
        headers,
        body,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

fn origin_of(url: &str) -> Result<String, AppError> {
    let parsed =
        url::Url::parse(url).map_err(|e| AppError::BadRequest(format!("invalid url: {e}")))?;
    Ok(format!(
        "{}://{}{}",
        parsed.scheme(),
        parsed.host_str().unwrap_or(""),
        parsed.port().map(|p| format!(":{p}")).unwrap_or_default()
    ))
}

fn domain_of(url: &str) -> Result<String, AppError> {
    let parsed =
        url::Url::parse(url).map_err(|e| AppError::BadRequest(format!("invalid url: {e}")))?;
    Ok(parsed.host_str().unwrap_or("").to_string())
}

fn referer_header(req: &FetchRequest) -> Option<String> {
    req.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("referer"))
        .map(|(_, v)| v.clone())
}
