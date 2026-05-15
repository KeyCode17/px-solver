use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub puts: u64,
    pub expired_evictions: u64,
    pub invalidations: u64,
    pub entries: usize,
}

impl CacheMetrics {
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            return 0.0;
        }
        (self.hits as f64) / (total as f64)
    }
}
