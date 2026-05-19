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
    /// Each entry is a JSON string of the form `[{"t":"PX…","d":{…}},…]`
    /// — either grabbed from a `JSON.stringify` call (rare) or from
    /// the `Array.prototype.join` hook (common, since `hY` builds
    /// JSON manually).
    pub plaintext_events: Vec<String>,
    /// Loose-filter dump of all JSON.stringify calls on arrays of
    /// objects. Useful when the runtime serialises events in a
    /// non-`{t, d}` shape and the strict filter misses them.
    #[serde(default)]
    pub all_stringify: Vec<String>,
    pub xhr_sends: Vec<CaptureXhr>,
    pub cookies: Vec<(String, String)>,
    pub user_agent: String,
    /// Diagnostics: how many times Array.prototype.join fired in the
    /// session (proves the hook is live) and the first ten short
    /// outputs (helps tune the filter).
    #[serde(default)]
    pub join_calls: u64,
    #[serde(default)]
    pub join_samples: Vec<String>,
    #[serde(default)]
    pub hooked: bool,
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
    // Navigate first — page context belongs to the target. Inject the
    // hook immediately after, before any periodic sensor send fires.
    if tokio::time::timeout(Duration::from_secs(60), client.goto(target_url))
        .await
        .is_err()
    {
        return Err(AppError::InternalError(format!("nav {target_url} timeout")));
    }
    client
        .execute(HOOK_SCRIPT, vec![])
        .await
        .map_err(|e| AppError::InternalError(format!("inject hook: {e}")))?;
    // Nudge the runtime: spread synthetic interactions across the wait
    // window so PX's periodic flush has events to send.
    let nudge_steps = std::cmp::max(1, wait_ms / 4_000);
    let step = wait_ms / nudge_steps;
    for i in 0..nudge_steps {
        let script = format!(
            "try {{ window.scrollTo(0, {y}); \
             document.dispatchEvent(new MouseEvent('mousemove', {{clientX: {x}, clientY: {y}, bubbles: true}})); \
             document.dispatchEvent(new KeyboardEvent('keydown', {{key: 'a', bubbles: true}})); \
            }} catch(_){{}}",
            x = 100 + i * 37,
            y = 50 + i * 73,
        );
        let _ = client.execute(&script, vec![]).await;
        sleep(Duration::from_millis(step)).await;
    }

    // Trigger pagehide → many PX runtimes flush the pending sensor on
    // unload via sendBeacon. We capture it before the dump.
    let _ = client
        .execute(
            "try { document.dispatchEvent(new Event('visibilitychange')); \
             window.dispatchEvent(new Event('pagehide')); \
             window.dispatchEvent(new Event('beforeunload')); } catch(_){}",
            vec![],
        )
        .await;
    sleep(Duration::from_millis(1_500)).await;

    let dump = client
        .execute(
            "return JSON.stringify({\
                captures: window.__pxCaptures || [],\
                all_stringify: window.__pxAllStringify || [],\
                join_calls: window.__pxJoinCalls || 0,\
                join_samples: window.__pxJoinSamples || [],\
                hooked: !!window.__pxHooked,\
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
        captures: Vec<String>,
        all_stringify: Vec<String>,
        #[allow(dead_code)]
        join_calls: u64,
        #[allow(dead_code)]
        join_samples: Vec<String>,
        #[allow(dead_code)]
        hooked: bool,
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
        all_stringify: parsed.all_stringify,
        xhr_sends: parsed.xhr,
        cookies,
        user_agent: parsed.ua,
        join_calls: parsed.join_calls,
        join_samples: parsed.join_samples,
        hooked: parsed.hooked,
    })
}
