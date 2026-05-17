//! Server-side runtime metrics.
//!
//! Counters live on [`ServerMetrics`]. Per-handler latency histograms live
//! on [`SolveHistogram`] and follow the Prometheus exposition format with
//! cumulative `_bucket{le=...}` counts plus `_sum` and `_count`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::AtomicU64;

/// Histogram bucket upper bounds in milliseconds. Picked to cover both the
/// production AC-2 budget (6s for the Chromium-only `perimeterx` handler)
/// and the deep-stealth budget per ADR-0022 (15s median / 20s p95 for the
/// `cloudflare` Camoufox path). The `+Inf` bucket is implicit.
pub const SOLVE_BUCKETS_MS: &[u64] = &[100, 250, 500, 1000, 2500, 5000, 10000, 15000, 20000, 30000];

#[derive(Default)]
pub struct ServerMetrics {
    pub solves_total: AtomicU64,
    pub solves_failed: AtomicU64,
    pub auth_denied: AtomicU64,
    pub allowlist_denied: AtomicU64,
    pub solve_histogram: SolveHistogram,
}

/// Latency histogram keyed by handler name. Bucket counts are cumulative
/// (each bucket includes all faster buckets). Plus per-handler sum + count
/// for Prometheus `_sum` and `_count` lines.
#[derive(Default)]
pub struct SolveHistogram {
    inner: Mutex<HashMap<String, HistEntry>>,
}

#[derive(Default, Clone)]
struct HistEntry {
    /// Counts per bucket in `SOLVE_BUCKETS_MS`; index N holds the
    /// observations that fell into `(SOLVE_BUCKETS_MS[N-1], SOLVE_BUCKETS_MS[N]]`
    /// (with index 0 being `(-inf, SOLVE_BUCKETS_MS[0]]`). Stored as
    /// per-bucket increments at observe time and converted to cumulative
    /// counts at expose time.
    increments: Vec<u64>,
    inf_count: u64,
    sum_ms: u64,
    count: u64,
}

impl HistEntry {
    fn new() -> Self {
        Self {
            increments: vec![0; SOLVE_BUCKETS_MS.len()],
            inf_count: 0,
            sum_ms: 0,
            count: 0,
        }
    }
}

impl SolveHistogram {
    /// Record a single solve latency for `handler`.
    pub fn observe(&self, handler: &str, solve_ms: u64) {
        let mut g = match self.inner.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        let entry = g.entry(handler.to_string()).or_insert_with(HistEntry::new);
        entry.count += 1;
        entry.sum_ms += solve_ms;
        let mut placed = false;
        for (i, &b) in SOLVE_BUCKETS_MS.iter().enumerate() {
            if solve_ms <= b {
                entry.increments[i] += 1;
                placed = true;
                break;
            }
        }
        if !placed {
            entry.inf_count += 1;
        }
    }

    /// Render Prometheus exposition lines into `out`. Caller already wrote
    /// the `# HELP` / `# TYPE` preamble (kept there so the metrics handler
    /// owns the canonical metric name).
    pub fn render(&self, metric: &str, out: &mut String) {
        let snapshot: Vec<(String, HistEntry)> = {
            let g = match self.inner.lock() {
                Ok(g) => g,
                Err(p) => p.into_inner(),
            };
            g.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        };
        for (handler, entry) in snapshot {
            let mut cum: u64 = 0;
            for (i, &b) in SOLVE_BUCKETS_MS.iter().enumerate() {
                cum += entry.increments[i];
                out.push_str(&format!(
                    "{metric}_bucket{{handler=\"{handler}\",le=\"{b}\"}} {cum}\n"
                ));
            }
            cum += entry.inf_count;
            out.push_str(&format!(
                "{metric}_bucket{{handler=\"{handler}\",le=\"+Inf\"}} {cum}\n"
            ));
            out.push_str(&format!(
                "{metric}_sum{{handler=\"{handler}\"}} {}\n",
                entry.sum_ms
            ));
            out.push_str(&format!(
                "{metric}_count{{handler=\"{handler}\"}} {}\n",
                entry.count
            ));
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn observe_places_in_correct_bucket() {
        let h = SolveHistogram::default();
        h.observe("perimeterx", 50);
        h.observe("perimeterx", 4500);
        h.observe("cloudflare", 10_500);
        h.observe("cloudflare", 35_000);
        let mut out = String::new();
        h.render("px_solve_ms", &mut out);
        // perimeterx 50ms -> le=100 cumulative 1
        assert!(
            out.contains("px_solve_ms_bucket{handler=\"perimeterx\",le=\"100\"} 1"),
            "out: {out}"
        );
        // perimeterx 4500ms -> in le=5000, cumulative up to le=5000 should be 2
        assert!(
            out.contains("px_solve_ms_bucket{handler=\"perimeterx\",le=\"5000\"} 2"),
            "out: {out}"
        );
        // cloudflare 10500ms -> le=15000 cumulative 1
        assert!(
            out.contains("px_solve_ms_bucket{handler=\"cloudflare\",le=\"15000\"} 1"),
            "out: {out}"
        );
        // cloudflare 35000ms -> overflows to +Inf, count at le=30000 still 1
        assert!(
            out.contains("px_solve_ms_bucket{handler=\"cloudflare\",le=\"30000\"} 1"),
            "out: {out}"
        );
        assert!(
            out.contains("px_solve_ms_bucket{handler=\"cloudflare\",le=\"+Inf\"} 2"),
            "out: {out}"
        );
        assert!(
            out.contains("px_solve_ms_sum{handler=\"perimeterx\"} 4550"),
            "out: {out}"
        );
        assert!(
            out.contains("px_solve_ms_count{handler=\"perimeterx\"} 2"),
            "out: {out}"
        );
    }
}
