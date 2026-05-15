use px_core::PxDetection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Detected {
    Yes(PxDetection),
    No,
}

pub trait Detector: Send + Sync {
    fn detect(&self, html: &str) -> Detected;
}
