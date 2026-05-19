pub mod camoufox_fetcher;
pub mod camoufox_pool;
pub mod caps;
pub mod fetch_script;
pub mod fetch_strategies;
pub mod proxy_pool;
pub mod sensor_capture;
pub mod session;
pub mod session_pool;
pub mod synthetic_user;

pub use sensor_capture::{CaptureResult, CaptureXhr, capture_sensor};
