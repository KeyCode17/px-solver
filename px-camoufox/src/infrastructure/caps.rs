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
    prefs.insert("intl.accept_languages".into(), json!(config.locale.clone()));
    prefs.insert("dom.webnotifications.enabled".into(), json!(false));
    prefs.insert("media.peerconnection.enabled".into(), json!(false));

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
        caps.insert(
            "proxy".into(),
            json!({ "proxyType": "manual", "httpProxy": proxy_url, "sslProxy": proxy_url }),
        );
    }
    caps
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
