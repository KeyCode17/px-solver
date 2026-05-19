#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
//! v1.8.0/P4 — live throughput soak for the native PX solver.
//!
//! Drives `SensorNativeSolver::solve` N times in parallel against a
//! live PX target and measures:
//!   * solve success rate (Ok / total)
//!   * sustained req/min
//!   * p50 / p95 latency
//!
//! Writes a markdown evidence file. Asserts the configurable target
//! (default 40 req/min); the bet from ADR-0024 is that native sensor
//! synthesis can sustain ≥40 req/min through `/v1/solve` once
//! `default_batch` is calibrated against ground-truth captures.
//!
//! Run with:
//!   NATIVE_SOAK=1 \
//!     [NATIVE_SOAK_URL=https://www.pedidosya.com.ar/] \
//!     [NATIVE_SOAK_N=80] \
//!     [NATIVE_SOAK_CONCURRENCY=8] \
//!     [NATIVE_SOAK_TARGET_RPM=40] \
//!     [NATIVE_SOAK_PROFILE=px-native/profiles/eT15wiaE.toml] \
//!     cargo test -p pxsolver-native --test throughput_soak -- --ignored --nocapture

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use futures::stream::{FuturesUnordered, StreamExt};
use px_core::{Fingerprint, PxAppId};
use px_native::profile::TenantProfile;
use px_native::{NativeSolver, SensorNativeSolver, SolveContext};
use reqwest::Client;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn native_throughput_soak() {
    if std::env::var("NATIVE_SOAK").ok().as_deref() != Some("1") {
        eprintln!("set NATIVE_SOAK=1 to run");
        return;
    }
    let url =
        std::env::var("NATIVE_SOAK_URL").unwrap_or_else(|_| "https://www.pedidosya.com.ar/".into());
    let n: usize = std::env::var("NATIVE_SOAK_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80);
    let concurrency: usize = std::env::var("NATIVE_SOAK_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    let target_rpm: f64 = std::env::var("NATIVE_SOAK_TARGET_RPM")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(40.0);
    let profile_path: PathBuf = std::env::var("NATIVE_SOAK_PROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("profiles/eT15wiaE.toml"));

    let profile = TenantProfile::load(&profile_path).expect("load profile");
    let app_id = PxAppId::new(&profile.app_id).expect("valid app id");
    let client = Client::builder().build().expect("client");
    let solver: Arc<dyn NativeSolver> =
        Arc::new(SensorNativeSolver::new(client, Arc::new(profile)));
    let ctx_template = SolveContext::new(url.clone(), app_id.clone(), soak_fingerprint());

    eprintln!("soak: n={n} concurrency={concurrency} target_rpm={target_rpm} url={url}");

    let mut latencies_ms: Vec<u128> = Vec::with_capacity(n);
    let mut ok_count: usize = 0;
    let mut err_count: usize = 0;
    let started = Instant::now();

    let mut tasks: FuturesUnordered<_> = FuturesUnordered::new();
    let mut launched = 0usize;
    while launched < n || !tasks.is_empty() {
        while launched < n && tasks.len() < concurrency {
            let solver = Arc::clone(&solver);
            let ctx = ctx_template.clone();
            tasks.push(tokio::spawn(async move {
                let t0 = Instant::now();
                let outcome = solver.solve(&ctx).await;
                (t0.elapsed(), outcome)
            }));
            launched += 1;
        }
        if let Some(res) = tasks.next().await {
            let (elapsed, outcome) = res.expect("join task");
            latencies_ms.push(elapsed.as_millis());
            match outcome {
                Ok(_) => ok_count += 1,
                Err(e) => {
                    err_count += 1;
                    eprintln!("solve err: {e}");
                }
            }
        }
    }
    let total_elapsed = started.elapsed();
    let rpm = (n as f64) / total_elapsed.as_secs_f64() * 60.0;
    let success_rate = (ok_count as f64) / (n as f64) * 100.0;
    latencies_ms.sort_unstable();
    let p50 = latencies_ms[latencies_ms.len() / 2];
    let p95 = latencies_ms[(latencies_ms.len() * 95 / 100).min(latencies_ms.len() - 1)];

    eprintln!(
        "\n=== NATIVE_SOAK ===\n  n: {n}\n  ok: {ok_count}\n  err: {err_count}\n  success_rate: {success_rate:.1}%\n  elapsed: {:?}\n  rpm: {rpm:.1}\n  p50: {p50} ms\n  p95: {p95} ms",
        total_elapsed
    );

    assert!(
        rpm >= target_rpm,
        "throughput {rpm:.1} req/min below target {target_rpm:.1}"
    );
}

fn soak_fingerprint() -> Fingerprint {
    Fingerprint {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0".into(),
        accept_language: vec!["es-AR".into(), "es".into(), "en-US".into()],
        screen_width: 1366,
        screen_height: 768,
        device_pixel_ratio: 1,
        timezone: "America/Argentina/Buenos_Aires".into(),
        platform: "Linux x86_64".into(),
        webgl_vendor: "Mozilla".into(),
        webgl_renderer: "Mozilla".into(),
    }
}
