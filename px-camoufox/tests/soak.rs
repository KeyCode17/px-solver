//! R5.9 — pedidosya Camoufox soak harness.
//!
//! Sequential harvests against a single URL via CamoufoxPool, measuring
//! per-harvest latency and CF-bypass cookie presence. Writes a
//! markdown evidence file to `docs/verification/<date>-pedidosya-camoufox-soak.md`
//! and asserts AC-1 (>= 95% success) and a relaxed AC-2 (median <= 30s
//! for Camoufox; the 6s production AC-2 budget does not apply to the
//! deep-stealth path — that's the open question this soak feeds into).
//!
//! Gated by env. Run with:
//!   DIAGNOSE_CAMOUFOX_SOAK=1 [DIAGNOSE_SOAK_N=10] [DIAGNOSE_URL=https://...] \
//!     cargo test -q -p px-camoufox --test soak -- --ignored --nocapture

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use px_camoufox::{CamoufoxConfig, CamoufoxPool};
use px_harvester::{HarvestRequest, Harvester};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

fn utc_iso() -> String {
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

fn utc_date() -> String {
    Command::new("date")
        .args(["-u", "+%Y-%m-%d"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn pedidosya_camoufox_soak() {
    if std::env::var("DIAGNOSE_CAMOUFOX_SOAK").ok().as_deref() != Some("1") {
        eprintln!("set DIAGNOSE_CAMOUFOX_SOAK=1 to run");
        return;
    }
    let url =
        std::env::var("DIAGNOSE_URL").unwrap_or_else(|_| "https://www.pedidosya.com.ar/".into());
    let n: usize = std::env::var("DIAGNOSE_SOAK_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    let mut cfg = CamoufoxConfig::from_env();
    cfg.headless = true;
    cfg.navigate_timeout = Duration::from_secs(60);
    cfg.validate().expect("camoufox binaries present");
    let pool = CamoufoxPool::new(cfg).expect("pool");

    let start_iso = utc_iso();
    let mut samples = Vec::with_capacity(n);
    let mut ok = 0usize;
    let mut fails: Vec<String> = Vec::new();

    for i in 0..n {
        let t0 = Instant::now();
        let req = HarvestRequest::new(&url);
        let outcome = tokio::time::timeout(Duration::from_secs(90), pool.harvest(req)).await;
        let elapsed_ms = t0.elapsed().as_millis() as u64;
        match outcome {
            Ok(Ok(r)) => {
                let has_cf = r.cookies.iter().any(|c| c.name == "cf_clearance");
                let has_px3 = r.cookies.iter().any(|c| c.name == "_px3");
                if has_cf && has_px3 {
                    ok += 1;
                    samples.push((elapsed_ms, true, r.cookies.len()));
                    eprintln!(
                        "[{i:>2}/{n}] ok elapsed={elapsed_ms}ms cookies={} cf=Y px3=Y",
                        r.cookies.len()
                    );
                } else {
                    samples.push((elapsed_ms, false, r.cookies.len()));
                    let reason = format!("missing_cookies cf={has_cf} px3={has_px3}");
                    fails.push(format!("#{i}: {reason}"));
                    eprintln!("[{i:>2}/{n}] FAIL {reason} elapsed={elapsed_ms}ms");
                }
            }
            Ok(Err(e)) => {
                samples.push((elapsed_ms, false, 0));
                fails.push(format!("#{i}: harvest_err: {e}"));
                eprintln!("[{i:>2}/{n}] ERR {e} elapsed={elapsed_ms}ms");
            }
            Err(_) => {
                samples.push((90_000, false, 0));
                fails.push(format!("#{i}: timeout 90s"));
                eprintln!("[{i:>2}/{n}] TIMEOUT 90s");
            }
        }
    }
    let end_iso = utc_iso();

    let mut sorted: Vec<u64> = samples.iter().map(|s| s.0).collect();
    sorted.sort_unstable();
    let total = samples.len();
    let median = if total == 0 {
        0
    } else if total % 2 == 1 {
        sorted[total / 2]
    } else {
        (sorted[total / 2 - 1] + sorted[total / 2]) / 2
    };
    let p95 = if total == 0 {
        0
    } else {
        sorted[((total as f64 * 0.95).ceil() as usize)
            .saturating_sub(1)
            .min(total - 1)]
    };
    let min = sorted.first().copied().unwrap_or(0);
    let max = sorted.last().copied().unwrap_or(0);
    let success = if total == 0 {
        0.0
    } else {
        ok as f64 / total as f64
    };

    let ac1 = if success >= 0.95 { "pass" } else { "fail" };
    // Camoufox AC-2 is RELAXED to 30s (vs production 6s) — this soak's
    // purpose is to gather data on whether even the relaxed budget holds.
    let ac2 = if median <= 30_000 { "pass" } else { "fail" };

    let date = utc_date();
    // Cargo runs tests with cwd = the package dir, so we resolve the
    // workspace root via CARGO_MANIFEST_DIR/.. to put evidence under the
    // canonical docs/verification/ tree at the repo root.
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let evidence = workspace_root
        .join("docs")
        .join("verification")
        .join(format!("{date}-pedidosya-camoufox-soak.md"));
    let _ = fs::create_dir_all(evidence.parent().unwrap());
    let mut body = String::new();
    body.push_str("# Camoufox soak — pedidosya\n\n");
    body.push_str(&format!("**Start:** {start_iso}\n"));
    body.push_str(&format!("**End:** {end_iso}\n"));
    body.push_str(&format!("**Target:** {url}\n"));
    body.push_str(&format!("**N requests:** {total}\n"));
    body.push_str("**Path:** Direct CamoufoxPool harvest (no HTTP layer, no cache)\n\n");
    body.push_str("## Counters\n\n");
    body.push_str("| Metric           | Value |\n");
    body.push_str("|------------------|-------|\n");
    body.push_str(&format!("| Total            | {total} |\n"));
    body.push_str(&format!("| Success (cf+px3) | {ok} |\n"));
    body.push_str(&format!("| Success ratio    | {success:.4} |\n"));
    body.push_str(&format!("| Median ms        | {median} |\n"));
    body.push_str(&format!("| p95 ms           | {p95} |\n"));
    body.push_str(&format!("| Min ms           | {min} |\n"));
    body.push_str(&format!("| Max ms           | {max} |\n\n"));
    body.push_str("## Per-sample\n\n");
    body.push_str("| # | ms | ok | cookies |\n");
    body.push_str("|---|----|----|---------|\n");
    for (i, (ms, sok, ck)) in samples.iter().enumerate() {
        body.push_str(&format!(
            "| {i} | {ms} | {} | {ck} |\n",
            if *sok { "Y" } else { "N" }
        ));
    }
    body.push_str("\n## Verdicts\n\n");
    body.push_str("| AC | Threshold (relaxed for Camoufox) | Observed | Verdict |\n");
    body.push_str("|----|-----------------------------------|----------|---------|\n");
    body.push_str(&format!(
        "| AC-1 | success_ratio >= 0.95 | {success:.4} | {ac1} |\n"
    ));
    body.push_str(&format!(
        "| AC-2 | median_ms <= 30000    | {median}    | {ac2} |\n\n"
    ));
    if !fails.is_empty() {
        body.push_str("## Failures\n\n");
        for f in &fails {
            body.push_str(&format!("- {f}\n"));
        }
    }
    fs::write(&evidence, body).expect("write evidence");
    eprintln!("[soak] wrote {}", evidence.display());
    eprintln!("[soak] AC-1={ac1} AC-2={ac2} success={success:.4} median={median}ms");

    assert_eq!(ac1, "pass", "AC-1 failed: success={success:.4}");
    assert_eq!(ac2, "pass", "AC-2 failed: median={median}ms");
}
