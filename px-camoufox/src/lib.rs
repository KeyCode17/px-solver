pub mod domain;
pub mod infrastructure;

pub use domain::config::{CamoufoxConfig, CamoufoxConfigError};
pub use infrastructure::camoufox_pool::CamoufoxPool;
