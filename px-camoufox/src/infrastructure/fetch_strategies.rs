//! The two execution strategies for `/v1/fetch` against a persistent
//! Camoufox session.
//!
//! * `navigate_and_read` (used for GET): drives the webdriver to the
//!   target URL so the browser handles Cloudflare's interactive
//!   challenge, then reads the response body via
//!   `document.body.innerText`. Status is sniffed from the body shape
//!   because webdriver doesn't surface response codes.
//! * `in_page_fetch` (used for POST/PUT/etc.): runs an in-page
//!   `fetch()` since webdriver navigation can't carry a request body.
//!   Any CF interactive challenge surfaced here arrives as HTML and
//!   is reported back as a 403.

use crate::infrastructure::fetch_script;
use px_errors::AppError;
use px_pipeline::FetchRequest;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

pub(crate) type FetchTriple = (u16, HashMap<String, String>, String);

pub(crate) async fn navigate_and_read(
    client: &fantoccini::Client,
    req: &FetchRequest,
    navigate_timeout: Duration,
) -> Result<FetchTriple, AppError> {
    // Warm session has CF clearance already; the new navigation only
    // needs DOM to settle before we read body text. 1s is enough for
    // the niles JSON endpoint to render `document.body.innerText`.
    navigate_with_wait(
        client,
        &req.url,
        navigate_timeout,
        Duration::from_millis(1_000),
    )
    .await?;
    let body_val = client
        .execute("return document.body.innerText;", vec![])
        .await
        .map_err(|e| AppError::InternalError(format!("read body: {e}")))?;
    let body = body_val.as_str().unwrap_or("").to_string();
    let trimmed = body.trim_start();
    let status = if trimmed.starts_with('{') || trimmed.starts_with('[') {
        200
    } else if looks_like_challenge(&body) {
        403
    } else {
        500
    };
    Ok((status, HashMap::new(), body))
}

pub(crate) async fn in_page_fetch(
    client: &fantoccini::Client,
    req: &FetchRequest,
    request_timeout: Duration,
) -> Result<FetchTriple, AppError> {
    let script = fetch_script::build(req)?;
    let exec = client.execute_async(&script, vec![]);
    let raw = match tokio::time::timeout(request_timeout, exec).await {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => return Err(AppError::InternalError(format!("fetch eval: {e}"))),
        Err(_) => return Err(AppError::InternalError("fetch timeout".into())),
    };
    let status = raw
        .get("status")
        .and_then(Value::as_u64)
        .ok_or_else(|| AppError::InternalError("fetch result missing status".into()))?
        as u16;
    let body = raw
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let headers: HashMap<String, String> = raw
        .get("headers")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default();
    Ok((status, headers, body))
}

fn looks_like_challenge(body: &str) -> bool {
    body.contains("__cf_chl")
        || body.contains("Just a moment")
        || body.contains("\"appId\":\"PXeT15wiaE\"")
        || body.contains("tráfico inusual")
        || body.contains("trafico inusual")
}

pub(crate) async fn navigate_with_wait(
    client: &fantoccini::Client,
    url: &str,
    navigate_timeout: Duration,
    settle: Duration,
) -> Result<(), AppError> {
    let nav = client.goto(url);
    if tokio::time::timeout(navigate_timeout, nav).await.is_err() {
        return Err(AppError::InternalError(format!("navigate timeout: {url}")));
    }
    sleep(settle).await;
    Ok(())
}
