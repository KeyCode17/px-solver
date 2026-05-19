//! `px-cli calibrate` — compare a Camoufox sensor capture against
//! `px-native`'s synthetic baseline batch and print the diff. Drives
//! ADR-0024 v1.8.0 P3 (field grammar calibration).

use std::fs;

use anyhow::{Context, Result};
use px_native::events::{SensorEvent, SyntheticIdentity, calibrate, default_batch};
use serde::Deserialize;

use crate::cli::CalibrateArgs;

#[derive(Deserialize)]
struct Capture {
    plaintext_events: Vec<Vec<SensorEvent>>,
}

pub async fn run(args: CalibrateArgs) -> Result<()> {
    let raw = fs::read_to_string(&args.capture)
        .with_context(|| format!("read capture {}", args.capture.display()))?;
    let capture: Capture = serde_json::from_str(&raw)
        .with_context(|| format!("parse capture {}", args.capture.display()))?;

    let observed: Vec<SensorEvent> = capture.plaintext_events.into_iter().flatten().collect();
    let synth = default_batch(&SyntheticIdentity::test_default(), 1_700_000_000_000);
    let report = calibrate(&observed, &synth);

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).context("serialize report")?
        );
        return Ok(());
    }

    println!("Observed events: {}", observed.len());
    println!("Synthetic events: {}\n", synth.len());

    if !report.missing_tags.is_empty() {
        println!("Missing tags (runtime emits, default_batch does not):");
        for t in &report.missing_tags {
            println!("  - {t}");
        }
        println!();
    }
    if !report.extra_tags.is_empty() {
        println!("Extra tags (default_batch emits, runtime did not):");
        for t in &report.extra_tags {
            println!("  - {t}");
        }
        println!();
    }
    for (tag, diff) in &report.per_tag {
        let obs = diff.observed_keys.len();
        let syn = diff.synthesised_keys.len();
        let miss = diff.missing_keys.len();
        println!("[{tag}] observed={obs} synth={syn} missing={miss}");
        for k in &diff.missing_keys {
            println!("  + need: {k}");
        }
    }
    Ok(())
}
