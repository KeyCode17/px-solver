pub mod domain;
pub mod infrastructure;

pub use domain::detector::{Detected, Detector};
pub use infrastructure::regex_detector::RegexDetector;
