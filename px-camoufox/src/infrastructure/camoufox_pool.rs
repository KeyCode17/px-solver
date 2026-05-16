use crate::domain::config::CamoufoxConfig;
use async_trait::async_trait;
use fantoccini::ClientBuilder;
use px_errors::AppError;
use px_harvester::{HarvestRequest, HarvestResult, HarvestedCookie, Harvester};
use serde_json::{Map, Value, json};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::time::sleep;

pub struct CamoufoxPool {
    config: CamoufoxConfig,
    permits: Arc<Semaphore>,
}

impl CamoufoxPool {
    pub fn new(config: CamoufoxConfig) -> Result<Self, AppError> {
        config
            .validate()
            .map_err(|e| AppError::InternalError(format!("camoufox config: {e}")))?;
        let permits = Arc::new(Semaphore::new(config.max_concurrent));
        Ok(Self { config, permits })
    }

    async fn pick_free_port() -> Result<u16, AppError> {
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

    async fn wait_for_geckodriver(port: u16, timeout: Duration) -> Result<(), AppError> {
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

    fn build_capabilities(&self, req: &HarvestRequest) -> Map<String, Value> {
        let mut prefs = Map::new();
        prefs.insert(
            "intl.accept_languages".into(),
            json!(self.config.locale.clone()),
        );
        prefs.insert("dom.webnotifications.enabled".into(), json!(false));
        prefs.insert("media.peerconnection.enabled".into(), json!(false));

        let mut firefox_options = Map::new();
        let mut args: Vec<String> = Vec::new();
        if self.config.headless {
            args.push("-headless".into());
        }
        firefox_options.insert("args".into(), json!(args));
        firefox_options.insert("prefs".into(), Value::Object(prefs));
        firefox_options.insert(
            "binary".into(),
            json!(self.config.camoufox_bin.to_string_lossy()),
        );

        let mut caps = Map::new();
        caps.insert("browserName".into(), json!("firefox"));
        caps.insert("moz:firefoxOptions".into(), Value::Object(firefox_options));
        if let Some(proxy_url) = &req.proxy {
            caps.insert(
                "proxy".into(),
                json!({ "proxyType": "manual", "httpProxy": proxy_url, "sslProxy": proxy_url }),
            );
        }
        caps
    }
}

async fn get_status(port: u16) -> Result<String, AppError> {
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

#[async_trait]
impl Harvester for CamoufoxPool {
    async fn harvest(&self, req: HarvestRequest) -> Result<HarvestResult, AppError> {
        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|e| AppError::InternalError(format!("semaphore: {e}")))?;
        let port = Self::pick_free_port().await?;
        let mut child = Command::new(&self.config.geckodriver_bin)
            .arg("--port")
            .arg(port.to_string())
            .arg("--binary")
            .arg(&self.config.camoufox_bin)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::InternalError(format!("spawn geckodriver: {e}")))?;
        Self::wait_for_geckodriver(port, Duration::from_secs(15)).await?;

        let caps = self.build_capabilities(&req);
        let endpoint = format!("http://127.0.0.1:{port}");
        let outcome = run_session(&endpoint, caps, &req, self.config.navigate_timeout).await;

        let _ = child.kill().await;
        outcome
    }
}

async fn run_session(
    endpoint: &str,
    caps: Map<String, Value>,
    req: &HarvestRequest,
    navigate_timeout: Duration,
) -> Result<HarvestResult, AppError> {
    let client = ClientBuilder::native()
        .capabilities(caps)
        .connect(endpoint)
        .await
        .map_err(|e| AppError::InternalError(format!("webdriver connect: {e}")))?;
    let nav = client.goto(&req.url);
    if tokio::time::timeout(navigate_timeout, nav).await.is_err() {
        let _ = client.close().await;
        return Err(AppError::InternalError("navigate timeout".into()));
    }
    sleep(Duration::from_millis(req.wait_ms)).await;
    let html = client
        .source()
        .await
        .map_err(|e| AppError::InternalError(format!("source: {e}")))?;
    let ua_val = client
        .execute("return navigator.userAgent;", vec![])
        .await
        .map_err(|e| AppError::InternalError(format!("ua eval: {e}")))?;
    let user_agent = ua_val.as_str().unwrap_or("").to_string();
    let raw_cookies = client
        .get_all_cookies()
        .await
        .map_err(|e| AppError::InternalError(format!("cookies: {e}")))?;
    let cookies = raw_cookies
        .into_iter()
        .map(|c| HarvestedCookie {
            name: c.name().to_string(),
            value: c.value().to_string(),
            domain: c.domain().unwrap_or("").to_string(),
            path: c.path().unwrap_or("/").to_string(),
        })
        .collect();
    let _ = client.close().await;
    Ok(HarvestResult {
        html,
        user_agent,
        cookies,
    })
}
