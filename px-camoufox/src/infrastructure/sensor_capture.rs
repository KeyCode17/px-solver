//! Ground-truth XHR + plaintext capture for the px-3 sensor.
//!
//! Spawns a fresh Camoufox session, navigates to `about:blank`, injects
//! a small monkey-patch that records:
//!   * every `JSON.stringify(arr)` where `arr` is a `[{t, d}]` shape —
//!     i.e. the plaintext sensor events before XOR/base64;
//!   * every `XMLHttpRequest.send(body)` whose URL contains `/b/s` —
//!     the final wire payload.
//!
//! Then navigates to the target URL and waits `wait_ms` for the
//! runtime to fire. Captures are returned as a `CaptureResult` plus the
//! cookies the session collected.

use crate::domain::config::CamoufoxConfig;
use crate::infrastructure::caps::{build_capabilities, pick_free_port, wait_for_geckodriver};
use fantoccini::ClientBuilder;
use px_errors::AppError;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

const HOOK_SCRIPT: &str = include_str!("sensor_capture_hook.js");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureXhr {
    pub url: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureResult {
    pub plaintext_events: Vec<serde_json::Value>,
    pub xhr_sends: Vec<CaptureXhr>,
    pub cookies: Vec<(String, String)>,
    pub user_agent: String,
}

pub async fn capture_sensor(
    config: &CamoufoxConfig,
    proxy: Option<&str>,
    target_url: &str,
    wait_ms: u64,
) -> Result<CaptureResult, AppError> {
    let port = pick_free_port().await?;
    let mut child = Command::new(&config.geckodriver_bin)
        .arg("--port")
        .arg(port.to_string())
        .arg("--binary")
        .arg(&config.camoufox_bin)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| AppError::InternalError(format!("spawn geckodriver: {e}")))?;
    wait_for_geckodriver(port, Duration::from_secs(15)).await?;
    let caps = build_capabilities(config, proxy);
    let endpoint = format!("http://127.0.0.1:{port}");
    let client = ClientBuilder::native()
        .capabilities(caps)
        .connect(&endpoint)
        .await
        .map_err(|e| AppError::InternalError(format!("webdriver connect: {e}")))?;

    let outcome = run_capture(&client, target_url, wait_ms).await;
    let _ = client.close().await;
    let _ = child.kill().await;
    outcome
}

async fn run_capture(
    client: &fantoccini::Client,
    target_url: &str,
    wait_ms: u64,
) -> Result<CaptureResult, AppError> {
    if tokio::time::timeout(Duration::from_secs(20), client.goto("about:blank"))
        .await
        .is_err()
    {
        return Err(AppError::InternalError("nav about:blank timeout".into()));
    }
    client
        .execute(HOOK_SCRIPT, vec![])
        .await
        .map_err(|e| AppError::InternalError(format!("inject hook: {e}")))?;

    if tokio::time::timeout(Duration::from_secs(60), client.goto(target_url))
        .await
        .is_err()
    {
        return Err(AppError::InternalError(format!("nav {target_url} timeout")));
    }
    sleep(Duration::from_millis(wait_ms)).await;

    let dump = client
        .execute(
            "return JSON.stringify({\
                captures: window.__pxCaptures || [],\
                xhr: window.__pxXhr || [],\
                ua: navigator.userAgent\
            });",
            vec![],
        )
        .await
        .map_err(|e| AppError::InternalError(format!("read captures: {e}")))?;
    let raw = dump.as_str().unwrap_or("");
    #[derive(Deserialize)]
    struct Raw {
        captures: Vec<serde_json::Value>,
        xhr: Vec<CaptureXhr>,
        ua: String,
    }
    let parsed: Raw = serde_json::from_str(raw)
        .map_err(|e| AppError::InternalError(format!("parse captures: {e}")))?;

    let raw_cookies = client
        .get_all_cookies()
        .await
        .map_err(|e| AppError::InternalError(format!("cookies: {e}")))?;
    let cookies = raw_cookies
        .into_iter()
        .map(|c| (c.name().to_string(), c.value().to_string()))
        .collect();

    Ok(CaptureResult {
        plaintext_events: parsed.captures,
        xhr_sends: parsed.xhr,
        cookies,
        user_agent: parsed.ua,
    })
}
