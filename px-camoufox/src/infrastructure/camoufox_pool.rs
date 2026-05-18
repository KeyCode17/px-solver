use crate::domain::config::CamoufoxConfig;
use crate::infrastructure::caps::{build_capabilities, pick_free_port, wait_for_geckodriver};
use async_trait::async_trait;
use fantoccini::ClientBuilder;
use px_errors::AppError;
use px_harvester::{HarvestRequest, HarvestResult, HarvestedCookie, Harvester};
use serde_json::{Map, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::time::sleep;

use crate::infrastructure::session_pool::SessionPool;

pub struct CamoufoxPool {
    pub(crate) config: CamoufoxConfig,
    pub(crate) permits: Arc<Semaphore>,
    pub(crate) sessions: Arc<SessionPool>,
}

impl CamoufoxPool {
    pub fn new(config: CamoufoxConfig) -> Result<Self, AppError> {
        config
            .validate()
            .map_err(|e| AppError::InternalError(format!("camoufox config: {e}")))?;
        let permits = Arc::new(Semaphore::new(config.max_concurrent));
        let max_per_domain = std::env::var("PX_FETCH_MAX_PER_DOMAIN")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(2);
        let sessions = Arc::new(SessionPool::new(
            config.clone(),
            Duration::from_secs(300),
            max_per_domain,
        ));
        Ok(Self {
            config,
            permits,
            sessions,
        })
    }

    /// Spawn geckodriver + Camoufox, hand the resulting webdriver
    /// endpoint to `body`, kill the child no matter what. Both the
    /// `Harvester` and `Fetcher` impls share this lifecycle.
    pub(crate) async fn with_session<F, R>(
        &self,
        proxy: Option<&str>,
        body: F,
    ) -> Result<R, AppError>
    where
        F: AsyncFnOnce(String, Map<String, Value>) -> Result<R, AppError>,
    {
        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|e| AppError::InternalError(format!("semaphore: {e}")))?;
        let port = pick_free_port().await?;
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
        wait_for_geckodriver(port, Duration::from_secs(15)).await?;
        let caps = build_capabilities(&self.config, proxy);
        let endpoint = format!("http://127.0.0.1:{port}");
        let outcome = body(endpoint, caps).await;
        let _ = child.kill().await;
        outcome
    }
}

#[async_trait]
impl Harvester for CamoufoxPool {
    async fn harvest(&self, req: HarvestRequest) -> Result<HarvestResult, AppError> {
        let navigate_timeout = self.config.navigate_timeout;
        let proxy = req.proxy.clone();
        self.with_session(proxy.as_deref(), async move |endpoint, caps| {
            harvest_session(&endpoint, caps, &req, navigate_timeout).await
        })
        .await
    }
}

async fn harvest_session(
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
