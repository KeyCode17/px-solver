//! Field-grammar calibration: compare a captured `[{t,d},…]` batch
//! against what [`super::default_batch`] would synthesise, and emit a
//! diff (missing tags, extra tags, per-tag field overlap).
//!
//! This is the consumer of ground-truth JSON produced by
//! `px-camoufox::capture_sensor` (ADR-0024 v1.8.0 P2). Use it to drive
//! P3 — closing the gap between the synthetic batch and what the
//! runtime actually emits.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::events::model::SensorEvent;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalibrationReport {
    /// Tags the runtime emitted but `default_batch` does not.
    pub missing_tags: Vec<String>,
    /// Tags `default_batch` emits but the runtime did not in this capture.
    pub extra_tags: Vec<String>,
    /// Per-tag breakdown: { tag → { observed_keys, synthesised_keys, missing_keys } }.
    pub per_tag: BTreeMap<String, TagDiff>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TagDiff {
    pub observed_keys: Vec<String>,
    pub synthesised_keys: Vec<String>,
    pub missing_keys: Vec<String>,
}

pub fn calibrate(observed: &[SensorEvent], synthesised: &[SensorEvent]) -> CalibrationReport {
    let observed_by_tag = group_by_tag(observed);
    let synth_by_tag = group_by_tag(synthesised);

    let observed_tags: BTreeSet<&String> = observed_by_tag.keys().collect();
    let synth_tags: BTreeSet<&String> = synth_by_tag.keys().collect();

    let mut report = CalibrationReport::default();
    for t in observed_tags.difference(&synth_tags) {
        report.missing_tags.push((*t).clone());
    }
    for t in synth_tags.difference(&observed_tags) {
        report.extra_tags.push((*t).clone());
    }
    for tag in observed_tags.intersection(&synth_tags) {
        let obs_keys = union_keys(observed_by_tag.get(*tag).unwrap_or(&Vec::new()));
        let syn_keys = union_keys(synth_by_tag.get(*tag).unwrap_or(&Vec::new()));
        let mut missing: Vec<String> = obs_keys.difference(&syn_keys).cloned().collect();
        missing.sort();
        report.per_tag.insert(
            (*tag).clone(),
            TagDiff {
                observed_keys: sorted(obs_keys),
                synthesised_keys: sorted(syn_keys),
                missing_keys: missing,
            },
        );
    }
    report
}

fn group_by_tag(events: &[SensorEvent]) -> BTreeMap<String, Vec<&SensorEvent>> {
    let mut by_tag: BTreeMap<String, Vec<&SensorEvent>> = BTreeMap::new();
    for ev in events {
        by_tag.entry(ev.t.clone()).or_default().push(ev);
    }
    by_tag
}

fn union_keys(events: &[&SensorEvent]) -> BTreeSet<String> {
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for ev in events {
        for k in ev.d.keys() {
            keys.insert(k.clone());
        }
    }
    keys
}

fn sorted(s: BTreeSet<String>) -> Vec<String> {
    s.into_iter().collect()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::events::SensorEvent;

    #[test]
    fn empty_captures_yield_empty_report() {
        let r = calibrate(&[], &[]);
        assert!(r.missing_tags.is_empty());
        assert!(r.extra_tags.is_empty());
    }

    #[test]
    fn reports_missing_and_extra_tags() {
        let obs = vec![SensorEvent::new("PXobs").with("k1", "v")];
        let syn = vec![SensorEvent::new("PXsyn").with("k2", "v")];
        let r = calibrate(&obs, &syn);
        assert_eq!(r.missing_tags, vec!["PXobs"]);
        assert_eq!(r.extra_tags, vec!["PXsyn"]);
    }

    #[test]
    fn flags_missing_keys_within_shared_tag() {
        let obs = vec![
            SensorEvent::new("PX561")
                .with("AzNweUZUfEs=", 1u64)
                .with("EwNgCVZlZDw=", "ua")
                .with("MISSING_KEY", "x"),
        ];
        let syn = vec![
            SensorEvent::new("PX561")
                .with("AzNweUZUfEs=", 1u64)
                .with("EwNgCVZlZDw=", "ua"),
        ];
        let r = calibrate(&obs, &syn);
        let diff = r.per_tag.get("PX561").expect("PX561 entry");
        assert_eq!(diff.missing_keys, vec!["MISSING_KEY"]);
    }
}
