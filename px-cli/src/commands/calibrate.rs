//! `px-cli calibrate` — compare a Camoufox sensor capture against
//! `px-native`'s synthetic baseline batch and print the diff. Drives
//! ADR-0024 v1.8.0 P3 (field grammar calibration).
//!
//! When the capture has no plaintext events (the runtime serialised
//! via the `hY` array-join path that bypasses our JSON.stringify
//! hook), we fall back to lossy-decrypting the `payload=` bodies in
//! `xhr_sends`. The recovered prefix is enough to learn tag and
//! field-key vocabulary even when the full event tree is truncated.

use std::fs;

use anyhow::{Context, Result};
use px_native::cipher::decrypt_payload_lossy;
use px_native::events::{SensorEvent, SyntheticIdentity, calibrate, default_batch};
use serde::Deserialize;

use crate::cli::CalibrateArgs;

#[derive(Deserialize)]
struct Capture {
    #[serde(default)]
    plaintext_events: Vec<String>,
    #[serde(default)]
    xhr_sends: Vec<CaptureXhr>,
}

#[derive(Deserialize)]
struct CaptureXhr {
    body: Option<String>,
}

pub async fn run(args: CalibrateArgs) -> Result<()> {
    let raw = fs::read_to_string(&args.capture)
        .with_context(|| format!("read capture {}", args.capture.display()))?;
    let capture: Capture = serde_json::from_str(&raw)
        .with_context(|| format!("parse capture {}", args.capture.display()))?;

    let mut observed: Vec<SensorEvent> = Vec::new();
    for entry in &capture.plaintext_events {
        match serde_json::from_str::<Vec<SensorEvent>>(entry) {
            Ok(events) => observed.extend(events),
            Err(e) => eprintln!("warn: skipping unparseable batch ({e})"),
        }
    }
    if observed.is_empty() {
        for xhr in &capture.xhr_sends {
            let Some(body) = xhr.body.as_deref() else {
                continue;
            };
            let Some(payload) = body.strip_prefix("payload=") else {
                continue;
            };
            let plaintext = decrypt_payload_lossy(payload);
            if let Some(events) = parse_partial_events(&plaintext) {
                observed.extend(events);
            }
        }
    }
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

/// Best-effort parse: try strict JSON; if it fails (the lossy decrypt
/// usually produces corrupted JSON past the first salt insertion), fall
/// back to **regex vocab extraction** — every `"t":"…"` becomes a
/// stub event whose `d` carries the field keys we can identify.
fn parse_partial_events(text: &str) -> Option<Vec<SensorEvent>> {
    if let Ok(events) = serde_json::from_str::<Vec<SensorEvent>>(text)
        && !events.is_empty()
    {
        return Some(events);
    }
    let mut events = Vec::new();
    let mut last_tag: Option<String> = None;
    let mut current_keys: Vec<String> = Vec::new();
    for (i, _) in text.match_indices('"').collect::<Vec<_>>() {
        // Look for "t":"…"
        if text.get(i..i + 5) == Some("\"t\":\"") {
            // Finalise previous event.
            if let Some(tag) = last_tag.take() {
                push_event(&mut events, tag, std::mem::take(&mut current_keys));
            }
            let rest = &text[i + 5..];
            if let Some(end) = rest.find('"') {
                last_tag = Some(rest[..end].to_owned());
            }
        }
        // Look for "<key>":  where key is a base64-ish blob ≥6 chars.
        if let Some(rest) = text.get(i + 1..)
            && let Some(close) = rest.find('"')
        {
            let candidate = &rest[..close];
            if (6..=40).contains(&close)
                && candidate
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
                && rest.as_bytes().get(close + 1) == Some(&b':')
                && !current_keys.iter().any(|k| k == candidate)
            {
                current_keys.push(candidate.to_owned());
            }
        }
    }
    if let Some(tag) = last_tag {
        push_event(&mut events, tag, current_keys);
    }
    if events.is_empty() {
        None
    } else {
        Some(events)
    }
}

fn push_event(out: &mut Vec<SensorEvent>, tag: String, keys: Vec<String>) {
    let mut ev = SensorEvent::new(tag);
    for k in keys {
        ev = ev.with(k, "(observed)");
    }
    out.push(ev);
}
