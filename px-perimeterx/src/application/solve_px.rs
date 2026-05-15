use crate::infrastructure::cookie_extractor::extract_px_cookies;
use px_core::{CookieJarDelta, NamedCookie};
use px_errors::AppError;
use px_harvester::{HarvestRequest, Harvester};
use std::sync::Arc;
use std::time::Instant;

pub struct SolvePx {
    harvester: Arc<dyn Harvester>,
}

pub struct SolveOutput {
    pub cookies: CookieJarDelta,
    pub user_agent: String,
    pub solve_ms: u64,
}

impl SolvePx {
    pub fn new(harvester: Arc<dyn Harvester>) -> Self {
        Self { harvester }
    }

    pub async fn execute(&self, url: &str) -> Result<SolveOutput, AppError> {
        let start = Instant::now();
        let result = self.harvester.harvest(HarvestRequest::new(url)).await?;
        let px_cookies: Vec<NamedCookie> = extract_px_cookies(&result.cookies)
            .into_iter()
            .map(|c| NamedCookie {
                name: c.name,
                value: c.value,
                domain: c.domain,
                path: c.path,
            })
            .collect();
        let delta = CookieJarDelta {
            set: px_cookies,
            removed: Vec::new(),
        };
        Ok(SolveOutput {
            cookies: delta,
            user_agent: result.user_agent,
            solve_ms: start.elapsed().as_millis() as u64,
        })
    }
}
