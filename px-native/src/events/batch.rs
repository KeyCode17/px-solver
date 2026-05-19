//! Compose a default event batch from a `SyntheticIdentity`.
//!
//! The PX-tag → data-field mapping is the part of the sensor protocol
//! we have NOT fully reversed yet (ADR-0024 N3 follow-up). What this
//! module does is emit a small batch matching the **shapes** we have
//! seen in the deobfuscated runtime — enough for the encryptor to
//! produce bytes the wire will accept the format of, even if a
//! production-grade trust score requires more events to be added
//! once we capture ground truth.

use crate::events::identity::SyntheticIdentity;
use crate::events::model::SensorEvent;

/// Build a baseline event batch with the well-known PX-tag set we have
/// surfaced from the eT15wiaE init.js. Each event carries the data
/// fields we can synthesise locally.
pub fn default_batch(identity: &SyntheticIdentity, now_ms: u64) -> Vec<SensorEvent> {
    vec![
        // Page-load / first emit. The runtime always sends one of these
        // at boot, before any user interaction has happened.
        SensorEvent::new("PX561")
            .with("AzNweUZUfEs=", now_ms)
            .with("EwNgCVZlZDw=", identity.user_agent.as_str())
            .with("HCgvIllLKRA=", identity.locale.as_str()),
        // Pixel-counter / visit-stats event. Carries the session count
        // and a derived visit-age timestamp.
        SensorEvent::new("PX11978")
            .with("AzNweUVSfU0=", i64::from(identity.session_count))
            .with(
                "XGhvYhkOb1g=",
                visit_age_ms(now_ms, identity.first_visit_days_ago),
            ),
        // Fingerprint container. We don't yet populate full WebGL /
        // canvas vectors here; that's the next iteration. The shape
        // is enough for the encryptor.
        SensorEvent::new("PX12457").with("dWFGKzACQxw=", identity.timezone.as_str()),
    ]
}

fn visit_age_ms(now_ms: u64, days_ago: u32) -> i64 {
    const MS_PER_DAY: i64 = 86_400_000;
    (now_ms as i64).saturating_sub(MS_PER_DAY.saturating_mul(i64::from(days_ago)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_batch_has_three_events() {
        let id = SyntheticIdentity::test_default();
        let batch = default_batch(&id, 1_716_192_345_678);
        assert_eq!(batch.len(), 3);
        assert_eq!(batch[0].t, "PX561");
        assert!(batch[0].d.contains_key("AzNweUZUfEs="));
    }

    #[test]
    fn visit_age_is_before_now() {
        let now: u64 = 1_716_192_345_678;
        let age = visit_age_ms(now, 14);
        assert!((age as u64) < now);
    }
}
