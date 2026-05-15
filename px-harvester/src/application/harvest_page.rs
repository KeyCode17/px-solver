use crate::domain::harvester::{HarvestRequest, HarvestResult, Harvester};
use px_errors::AppError;
use std::sync::Arc;

pub struct HarvestPage {
    harvester: Arc<dyn Harvester>,
}

impl HarvestPage {
    pub fn new(harvester: Arc<dyn Harvester>) -> Self {
        Self { harvester }
    }

    pub async fn execute(&self, req: HarvestRequest) -> Result<HarvestResult, AppError> {
        self.harvester.harvest(req).await
    }
}
