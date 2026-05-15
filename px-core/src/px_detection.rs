use crate::px_app_id::PxAppId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PxDetection {
    pub app_id: PxAppId,
    pub init_js_path: String,
    pub collector_base: String,
    pub mode: PxMode,
    pub block_class: BlockClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PxMode {
    ReverseProxy,
    Hosted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum BlockClass {
    None,
    Block,
    Captcha,
}

impl PxDetection {
    pub fn new(
        app_id: PxAppId,
        init_js_path: impl Into<String>,
        collector_base: impl Into<String>,
        mode: PxMode,
        block_class: BlockClass,
    ) -> Self {
        Self {
            app_id,
            init_js_path: init_js_path.into(),
            collector_base: collector_base.into(),
            mode,
            block_class,
        }
    }
}
