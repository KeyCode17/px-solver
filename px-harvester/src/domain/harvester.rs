use async_trait::async_trait;
use px_core::Fingerprint;
use px_errors::AppError;

#[derive(Debug, Clone)]
pub struct HarvestRequest {
    pub url: String,
    pub proxy: Option<String>,
    pub fingerprint: Option<Fingerprint>,
    pub wait_ms: u64,
}

impl HarvestRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            proxy: None,
            fingerprint: None,
            wait_ms: 2_500,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HarvestResult {
    pub html: String,
    pub user_agent: String,
    pub cookies: Vec<HarvestedCookie>,
}

#[derive(Debug, Clone)]
pub struct HarvestedCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[async_trait]
pub trait Harvester: Send + Sync {
    async fn harvest(&self, req: HarvestRequest) -> Result<HarvestResult, AppError>;
}
