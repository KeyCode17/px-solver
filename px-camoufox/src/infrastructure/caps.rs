//! Shared Camoufox webdriver helpers used by both `Harvester` and
//! `Fetcher` implementations on `CamoufoxPool`.

use crate::domain::config::CamoufoxConfig;
use px_errors::AppError;
use serde_json::{Map, Value, json};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::sleep;

pub(crate) fn build_capabilities(
    config: &CamoufoxConfig,
    proxy: Option<&str>,
) -> Map<String, Value> {
    let mut prefs = Map::new();
    // Build a richer accept-language string when locale is regional
    // (es-AR → "es-AR,es;q=0.9,en-US;q=0.7,en;q=0.5"). PerimeterX
    // weights this against the IP geo; "en-US" only on an AR exit is a
    // bot tell.
    prefs.insert(
        "intl.accept_languages".into(),
        json!(accept_languages(&config.locale)),
    );
    prefs.insert("intl.locale.requested".into(), json!(config.locale.clone()));
    prefs.insert(
        "general.useragent.locale".into(),
        json!(config.locale.clone()),
    );
    prefs.insert("dom.webnotifications.enabled".into(), json!(false));
    prefs.insert("media.peerconnection.enabled".into(), json!(false));
    // History / privacy posture of a typical user — keeps history,
    // accepts third-party cookies (most PX-protected SPAs need them),
    // disables DNT (DNT=1 is a small but real bot signal).
    prefs.insert("privacy.donottrackheader.enabled".into(), json!(false));
    prefs.insert("places.history.enabled".into(), json!(true));
    prefs.insert("network.cookie.cookieBehavior".into(), json!(0));
    // When using SOCKS, resolve hostnames through the proxy too — keeps
    // DNS leaks off the operator's egress and matches the proxy's geo.
    if let Some(proxy_url) = proxy
        && proxy_url.starts_with("socks")
    {
        prefs.insert("network.proxy.socks_remote_dns".into(), json!(true));
    }

    let mut firefox_options = Map::new();
    let mut args: Vec<String> = Vec::new();
    if config.headless {
        args.push("-headless".into());
    }
    firefox_options.insert("args".into(), json!(args));
    firefox_options.insert("prefs".into(), Value::Object(prefs));
    firefox_options.insert(
        "binary".into(),
        json!(config.camoufox_bin.to_string_lossy()),
    );

    let mut caps = Map::new();
    caps.insert("browserName".into(), json!("firefox"));
    caps.insert("moz:firefoxOptions".into(), Value::Object(firefox_options));
    if let Some(proxy_url) = proxy {
        caps.insert("proxy".into(), build_proxy_capability(proxy_url));
    }
    caps
}

/// Translate a `scheme://[user:pass@]host:port` proxy URL into the W3C
/// `proxy` capability geckodriver expects. SOCKS variants set
/// `socksProxy` + `socksVersion`; everything else falls back to
/// `httpProxy` / `sslProxy`. Auth in the URL is ignored at the
/// capability layer — geckodriver/Firefox have no W3C field for proxy
/// credentials; operators that need auth should front the upstream
/// with a local unauth relay (gost, 3proxy, etc.).
fn build_proxy_capability(proxy_url: &str) -> Value {
    let stripped = proxy_url
        .strip_prefix("socks5h://")
        .or_else(|| proxy_url.strip_prefix("socks5://"))
        .or_else(|| proxy_url.strip_prefix("socks4://"))
        .or_else(|| proxy_url.strip_prefix("socks://"))
        .map(|rest| (true, rest))
        .or_else(|| {
            proxy_url
                .strip_prefix("http://")
                .or_else(|| proxy_url.strip_prefix("https://"))
                .map(|rest| (false, rest))
        });
    let (is_socks, host_port) = match stripped {
        Some((s, rest)) => (s, drop_userinfo(rest)),
        None => (false, drop_userinfo(proxy_url)),
    };
    if is_socks {
        json!({
            "proxyType": "manual",
            "socksProxy": host_port,
            "socksVersion": 5,
        })
    } else {
        json!({
            "proxyType": "manual",
            "httpProxy": host_port,
            "sslProxy": host_port,
        })
    }
}

fn drop_userinfo(rest: &str) -> String {
    match rest.rsplit_once('@') {
        Some((_userinfo, host_port)) => host_port.to_string(),
        None => rest.to_string(),
    }
}

/// Turn a single locale (`es-AR`) into a realistic browser
/// `Accept-Language` chain. Falls back to the raw locale when we don't
/// have a known regional mapping.
fn accept_languages(locale: &str) -> String {
    match locale {
        "es-AR" => "es-AR,es;q=0.9,en-US;q=0.7,en;q=0.5".into(),
        "es-MX" => "es-MX,es;q=0.9,en-US;q=0.7,en;q=0.5".into(),
        "es-ES" => "es-ES,es;q=0.9,en-US;q=0.7,en;q=0.5".into(),
        "pt-BR" => "pt-BR,pt;q=0.9,en-US;q=0.7,en;q=0.5".into(),
        "en-US" => "en-US,en;q=0.9".into(),
        other => other.into(),
    }
}

pub(crate) async fn pick_free_port() -> Result<u16, AppError> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| AppError::InternalError(format!("bind ephemeral: {e}")))?;
    let port = listener
        .local_addr()
        .map_err(|e| AppError::InternalError(format!("local_addr: {e}")))?
        .port();
    drop(listener);
    Ok(port)
}

pub(crate) async fn wait_for_geckodriver(port: u16, timeout: Duration) -> Result<(), AppError> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if let Ok(resp) = get_status(port).await
            && resp.contains("\"ready\":true")
        {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(AppError::InternalError(format!(
                "geckodriver did not become ready on port {port} within {timeout:?}"
            )));
        }
        sleep(Duration::from_millis(150)).await;
    }
}

pub(crate) async fn get_status(port: u16) -> Result<String, AppError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .map_err(|e| AppError::InternalError(format!("connect: {e}")))?;
    let req =
        format!("GET /status HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(req.as_bytes())
        .await
        .map_err(|e| AppError::InternalError(format!("write: {e}")))?;
    let mut buf = Vec::with_capacity(2048);
    stream
        .read_to_end(&mut buf)
        .await
        .map_err(|e| AppError::InternalError(format!("read: {e}")))?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}
