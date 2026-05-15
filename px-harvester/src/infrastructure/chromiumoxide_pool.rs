use crate::domain::harvester::{HarvestRequest, HarvestResult, HarvestedCookie, Harvester};
use crate::domain::stealth::{StealthBundle, default_stealth_bundle};
use async_trait::async_trait;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
use px_errors::AppError;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_concurrent: usize,
    pub navigate_timeout: Duration,
    pub headless: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            navigate_timeout: Duration::from_secs(30),
            headless: true,
        }
    }
}

pub struct ChromiumoxidePool {
    config: PoolConfig,
    stealth: StealthBundle,
    permits: Arc<Semaphore>,
}

impl ChromiumoxidePool {
    pub fn new(config: PoolConfig) -> Self {
        let permits = Arc::new(Semaphore::new(config.max_concurrent));
        Self {
            config,
            stealth: default_stealth_bundle(),
            permits,
        }
    }

    pub fn with_stealth(mut self, stealth: StealthBundle) -> Self {
        self.stealth = stealth;
        self
    }

    async fn launch_browser(&self) -> Result<(Browser, tokio::task::JoinHandle<()>), AppError> {
        let mut cfg = BrowserConfig::builder();
        if !self.config.headless {
            cfg = cfg.with_head();
        }
        let cfg = cfg
            .build()
            .map_err(|e| AppError::InternalError(format!("BrowserConfig build: {e}")))?;
        let (browser, mut handler) = Browser::launch(cfg)
            .await
            .map_err(|e| AppError::InternalError(format!("browser launch: {e}")))?;
        let handle = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });
        Ok((browser, handle))
    }

    async fn extract_cookies(page: &Page) -> Result<Vec<HarvestedCookie>, AppError> {
        let cookies = page
            .get_cookies()
            .await
            .map_err(|e| AppError::InternalError(format!("get_cookies: {e}")))?;
        Ok(cookies
            .into_iter()
            .map(|c| HarvestedCookie {
                name: c.name,
                value: c.value,
                domain: c.domain,
                path: c.path,
            })
            .collect())
    }
}

#[async_trait]
impl Harvester for ChromiumoxidePool {
    async fn harvest(&self, req: HarvestRequest) -> Result<HarvestResult, AppError> {
        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|e| AppError::InternalError(format!("semaphore: {e}")))?;
        let (mut browser, _handle) = self.launch_browser().await?;
        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| AppError::InternalError(format!("new_page: {e}")))?;
        let script = self.stealth.combined();
        if !script.is_empty() {
            page.evaluate_on_new_document(script.as_str())
                .await
                .map_err(|e| AppError::InternalError(format!("inject stealth: {e}")))?;
        }
        let navigate = page.goto(&req.url);
        tokio::time::timeout(self.config.navigate_timeout, navigate)
            .await
            .map_err(|_| AppError::InternalError("navigate timeout".into()))?
            .map_err(|e| AppError::InternalError(format!("goto: {e}")))?;
        tokio::time::sleep(Duration::from_millis(req.wait_ms)).await;
        let html = page
            .content()
            .await
            .map_err(|e| AppError::InternalError(format!("content: {e}")))?;
        let ua = page
            .evaluate("navigator.userAgent")
            .await
            .map_err(|e| AppError::InternalError(format!("eval ua: {e}")))?
            .into_value::<String>()
            .map_err(|e| AppError::InternalError(format!("ua parse: {e}")))?;
        let cookies = Self::extract_cookies(&page).await?;
        let _ = browser.close().await;
        Ok(HarvestResult {
            html,
            user_agent: ua,
            cookies,
        })
    }
}
