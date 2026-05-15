use crate::px_app_id::PxAppId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PxDetection {
    pub app_id: Option<PxAppId>,
    pub init_js_path: Option<String>,
    pub collector_base: Option<String>,
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
    pub fn with_app_id(
        app_id: PxAppId,
        init_js_path: impl Into<String>,
        collector_base: impl Into<String>,
        mode: PxMode,
        block_class: BlockClass,
    ) -> Self {
        Self {
            app_id: Some(app_id),
            init_js_path: Some(init_js_path.into()),
            collector_base: Some(collector_base.into()),
            mode,
            block_class,
        }
    }

    pub fn marker_only(mode: PxMode, block_class: BlockClass) -> Self {
        Self {
            app_id: None,
            init_js_path: None,
            collector_base: None,
            mode,
            block_class,
        }
    }
}
