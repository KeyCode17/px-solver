//! Long-lived Camoufox session that survives across multiple
//! `/v1/fetch` calls. Spawning geckodriver + Camoufox costs ~13s on
//! each fresh harvest, and rapid repeats trip PerimeterX's
//! "tráfico inusual" rate limit — so the fetcher reuses one warm
//! browser per allowed domain.

use crate::domain::config::CamoufoxConfig;
use crate::infrastructure::caps::{build_capabilities, pick_free_port, wait_for_geckodriver};
use crate::infrastructure::synthetic_user::SyntheticUser;
use fantoccini::ClientBuilder;
use px_errors::AppError;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::Mutex;

/// A live Camoufox/geckodriver pair plus the webdriver client.
/// Concurrent callers serialize on `inner` so a single browser only
/// handles one fetch at a time (sharing it across threads would
/// race on URL navigation).
pub(crate) struct PersistentSession {
    pub(crate) created_at: Instant,
    pub(crate) inner: Mutex<Inner>,
    pub(crate) user: SyntheticUser,
}

pub(crate) struct Inner {
    pub(crate) client: fantoccini::Client,
    /// Held only for its `kill_on_drop(true)` semantics. Never read.
    #[allow(dead_code)]
    pub(crate) child: tokio::process::Child,
    pub(crate) last_used: Instant,
    pub(crate) fetch_count: u64,
    /// `true` after the first homepage navigation has happened, so
    /// subsequent fetches skip the warm-up step.
    pub(crate) warmed: bool,
    /// The URL the webdriver is currently parked on. Used to skip a
    /// redundant referer navigation when the new request would land
    /// on the same page anyway.
    pub(crate) last_visited: Option<String>,
}

impl PersistentSession {
    pub(crate) async fn spawn(
        config: &CamoufoxConfig,
        domain: &str,
        proxy: Option<&str>,
        user: SyntheticUser,
    ) -> Result<Self, AppError> {
        let port = pick_free_port().await?;
        let child = Command::new(&config.geckodriver_bin)
            .arg("--port")
            .arg(port.to_string())
            .arg("--binary")
            .arg(&config.camoufox_bin)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AppError::InternalError(format!("spawn geckodriver: {e}")))?;
        wait_for_geckodriver(port, Duration::from_secs(15)).await?;
        // Per-user override of the config-level locale: the synthetic
        // user's locale beats the global PX_CAMOUFOX_LOCALE so each
        // session carries the matching Accept-Language chain.
        let mut user_config = config.clone();
        user_config.locale = user.locale.clone();
        let caps = build_capabilities(&user_config, proxy);
        let endpoint = format!("http://127.0.0.1:{port}");
        let client = ClientBuilder::native()
            .capabilities(caps)
            .connect(&endpoint)
            .await
            .map_err(|e| AppError::InternalError(format!("webdriver connect: {e}")))?;
        // Resize to the user's viewport so the screen fingerprint
        // varies per synthetic identity.
        let (w, h) = user.viewport;
        let _ = client.set_window_size(w, h).await;
        tracing::info!(
            domain = %domain,
            port,
            proxy = proxy.unwrap_or("direct"),
            user = %user.id,
            locale = %user.locale,
            viewport = format!("{w}x{h}"),
            "persistent Camoufox session spawned"
        );
        Ok(Self {
            created_at: Instant::now(),
            inner: Mutex::new(Inner {
                client,
                child,
                last_used: Instant::now(),
                fetch_count: 0,
                warmed: false,
                last_visited: None,
            }),
            user,
        })
    }

    /// Best-effort age check. Doesn't peek inside the lock — callers
    /// outside the mutex use this only to short-circuit when the
    /// session is obviously stale.
    pub(crate) fn is_aged_out(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
}
