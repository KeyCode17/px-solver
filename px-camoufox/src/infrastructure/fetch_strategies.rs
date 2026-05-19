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

/// Synthetic "human" signal pack injected before `in_page_fetch`.
///
/// PerimeterX scores the bare `navigate → fetch` sequence as automation
/// (no scroll, no pointer, no idle dwell). This script:
///
/// * dispatches a handful of `mousemove` events along a curved path,
/// * scrolls to a randomised midpoint and back,
/// * seeds a couple of realistic-looking `localStorage`/`sessionStorage`
///   keys that PX consults for cross-visit continuity signals,
/// * sleeps a randomised 600-1400 ms so request rhythm has jitter.
///
/// The whole thing is best-effort — any error from `execute` is logged
/// at debug and swallowed so the fetch still runs.
/// Synthetic "human" signal pack injected before `in_page_fetch`.
///
/// Beyond the bare scroll/mouse jitter, this seeds the page with the
/// kind of "lived-in browser" state PerimeterX scores positively: a
/// long-running GA `_ga` cookie, a `_gid` and `_fbp`, a synthetic
/// `peya:firstVisit` localStorage timestamp set to N days ago, and a
/// realistic visit count. When `user` is `Some`, those values come
/// from a `SyntheticUser`; otherwise random anonymous values are used.
pub(crate) async fn humanize_for(
    client: &fantoccini::Client,
    user: Option<&crate::infrastructure::synthetic_user::SyntheticUser>,
) -> Result<(), AppError> {
    let ga = user.map(|u| u.ga_client_id.clone()).unwrap_or_default();
    let gid = user.map(|u| u.gid.clone()).unwrap_or_default();
    let fbp = user.map(|u| u.fbp.clone()).unwrap_or_default();
    let first_visit_days = user.map(|u| u.first_visit_days_ago).unwrap_or(7);
    let sessions = user.map(|u| u.session_count).unwrap_or(3);
    let user_id = user.map(|u| u.id.clone()).unwrap_or_else(|| "anon".into());
    let user_id_js = serde_json::to_string(&user_id).unwrap_or_else(|_| "\"anon\"".into());
    let ga_js = serde_json::to_string(&ga).unwrap_or_else(|_| "\"\"".into());
    let gid_js = serde_json::to_string(&gid).unwrap_or_else(|_| "\"\"".into());
    let fbp_js = serde_json::to_string(&fbp).unwrap_or_else(|_| "\"\"".into());
    let script = format!(
        r#"
        const cb = arguments[arguments.length - 1];
        try {{
          const rand = (min, max) => min + Math.random() * (max - min);
          const w = window.innerWidth || 1280;
          const h = window.innerHeight || 720;
          for (let i = 0; i < 6; i++) {{
            document.dispatchEvent(new MouseEvent('mousemove', {{
              clientX: rand(40, w - 40),
              clientY: rand(40, h - 40),
              bubbles: true,
            }}));
          }}
          window.scrollTo({{ top: rand(120, 480), behavior: 'instant' }});
          setTimeout(() => window.scrollTo({{ top: rand(0, 200), behavior: 'instant' }}), 200);
          try {{
            const visitMs = Date.now() - {first_visit_days} * 86400000;
            localStorage.setItem('peya:firstVisit', String(visitMs));
            localStorage.setItem('peya:sessions', String({sessions}));
            localStorage.setItem('peya:userTag', {user_id_js});
            sessionStorage.setItem('peya:lastInteraction', String(Date.now()));
            if ({ga_js}) document.cookie = '_ga=' + {ga_js} + '; path=/; max-age=63072000';
            if ({gid_js}) document.cookie = '_gid=' + {gid_js} + '; path=/; max-age=86400';
            if ({fbp_js}) document.cookie = '_fbp=' + {fbp_js} + '; path=/; max-age=7776000';
          }} catch (e) {{ /* storage / cookie may be partitioned */ }}
          setTimeout(cb, Math.floor(rand(600, 1400)));
        }} catch (e) {{ cb(); }}
        "#,
    );
    let exec = client.execute_async(&script, vec![]);
    if tokio::time::timeout(Duration::from_secs(5), exec)
        .await
        .is_err()
    {
        tracing::debug!("humanize timeout (non-fatal)");
    }
    Ok(())
}
