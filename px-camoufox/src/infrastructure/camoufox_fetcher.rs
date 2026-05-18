//! `Fetcher` impl on `CamoufoxPool`. Runs a single HTTP request from
//! inside a fresh Camoufox session by navigating to the target URL's
//! origin (so cookies/JS context are issued), then executing a
//! `fetch()` from the page context. Captures status, headers, body.

use crate::infrastructure::camoufox_pool::CamoufoxPool;
use async_trait::async_trait;
use fantoccini::ClientBuilder;
use px_errors::AppError;
use px_pipeline::{FetchRequest, FetchResponse, Fetcher};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[async_trait]
impl Fetcher for CamoufoxPool {
    async fn fetch(&self, req: FetchRequest) -> Result<FetchResponse, AppError> {
        let navigate_timeout = self.config.navigate_timeout;
        let request_timeout = Duration::from_millis(req.timeout_ms);
        self.with_session(None, async move |endpoint, caps| {
            run_fetch(&endpoint, caps, &req, navigate_timeout, request_timeout).await
        })
        .await
    }
}

async fn run_fetch(
    endpoint: &str,
    caps: Map<String, Value>,
    req: &FetchRequest,
    navigate_timeout: Duration,
    request_timeout: Duration,
) -> Result<FetchResponse, AppError> {
    let started = Instant::now();
    let client = ClientBuilder::native()
        .capabilities(caps)
        .connect(endpoint)
        .await
        .map_err(|e| AppError::InternalError(format!("webdriver connect: {e}")))?;

    let origin = origin_of(&req.url)?;
    let nav = client.goto(&origin);
    if tokio::time::timeout(navigate_timeout, nav).await.is_err() {
        let _ = client.close().await;
        return Err(AppError::InternalError("navigate timeout".into()));
    }
    // Give Cloudflare / PerimeterX a beat to set their cookies.
    sleep(Duration::from_millis(1_500)).await;

    let script = build_fetch_script(req)?;
    let exec = client.execute_async(&script, vec![]);
    let raw = match tokio::time::timeout(request_timeout, exec).await {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            let _ = client.close().await;
            return Err(AppError::InternalError(format!("fetch eval: {e}")));
        }
        Err(_) => {
            let _ = client.close().await;
            return Err(AppError::InternalError("fetch timeout".into()));
        }
    };
    let _ = client.close().await;

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

fn build_fetch_script(req: &FetchRequest) -> Result<String, AppError> {
    let method = req.method().to_string();
    let headers_json = serde_json::to_string(&req.headers)
        .map_err(|e| AppError::InternalError(format!("encode headers: {e}")))?;
    let body_json = serde_json::to_string(req.body.as_deref().unwrap_or(""))
        .map_err(|e| AppError::InternalError(format!("encode body: {e}")))?;
    let url_json = serde_json::to_string(&req.url)
        .map_err(|e| AppError::InternalError(format!("encode url: {e}")))?;
    let method_json = serde_json::to_string(&method)
        .map_err(|e| AppError::InternalError(format!("encode method: {e}")))?;
    Ok(format!(
        r#"
        const cb = arguments[arguments.length - 1];
        const opts = {{ method: {method_json}, headers: {headers_json}, credentials: 'include' }};
        const body = {body_json};
        if (body !== '' && {method_json} !== 'GET') opts.body = body;
        fetch({url_json}, opts)
          .then(async (r) => {{
            const text = await r.text();
            const hdrs = {{}};
            r.headers.forEach((v, k) => {{ hdrs[k] = v; }});
            cb({{ status: r.status, headers: hdrs, body: text }});
          }})
          .catch((e) => cb({{ status: 0, headers: {{}}, body: String(e) }}));
        "#
    ))
}
