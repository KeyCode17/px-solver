//! `ChallengeHandler` adapter for [`SensorNativeSolver`] so it can slot
//! into the existing routing dispatcher. Pair this with
//! [`super::native_first::NativeFirstHandler`] to get the
//! "native first, browser on failure" wiring.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use px_core::{CookieJarDelta, Fingerprint, PxAppId};
use px_errors::AppError;
use px_pipeline::{ChallengeHandler, HandlerMetrics, HandlerName, HandlerOutcome, PageHtml};

use crate::domain::native_solver::{NativeSolver, SolveContext};

pub struct NativePxHandler {
    solver: Arc<dyn NativeSolver>,
    app_id: PxAppId,
    name: HandlerName,
}

impl NativePxHandler {
    pub fn new(solver: Arc<dyn NativeSolver>, app_id: PxAppId) -> Self {
        Self {
            solver,
            app_id,
            name: "perimeterx-native",
        }
    }
}

#[async_trait]
impl ChallengeHandler for NativePxHandler {
    fn name(&self) -> HandlerName {
        self.name
    }

    async fn detects(&self, _page: &PageHtml) -> Result<bool, AppError> {
        Ok(true)
    }

    async fn solve(&self, page: &PageHtml) -> Result<HandlerOutcome, AppError> {
        let started = Instant::now();
        let ctx = SolveContext::new(page.url.clone(), self.app_id.clone(), default_fingerprint());
        let bundle = self.solver.solve(&ctx).await?;
        let metrics = HandlerMetrics {
            solve_ms: started.elapsed().as_millis() as u64,
            ..Default::default()
        };
        Ok(HandlerOutcome::solved_with_ua(
            self.name,
            CookieJarDelta {
                set: bundle.cookies,
                removed: Vec::new(),
            },
            Vec::new(),
            metrics,
            bundle.user_agent,
        ))
    }
}

fn default_fingerprint() -> Fingerprint {
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

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;
    use px_core::{NamedCookie, PxCookieBundle};
    use px_pipeline::HandlerStatus;
    use std::time::{Duration, SystemTime};

    struct AlwaysOkSolver;

    #[async_trait]
    impl NativeSolver for AlwaysOkSolver {
        async fn solve(&self, _ctx: &SolveContext) -> Result<PxCookieBundle, AppError> {
            Ok(PxCookieBundle::new(
                vec![NamedCookie {
                    name: "_px3".into(),
                    value: "native".into(),
                    domain: "example.com".into(),
                    path: "/".into(),
                }],
                "ua",
                SystemTime::now(),
                Duration::from_secs(60),
            ))
        }
    }

    fn app_id() -> PxAppId {
        PxAppId::new("PXeT15wiaE").expect("valid app id")
    }

    #[tokio::test]
    async fn handler_reports_solved_status() {
        let handler =
            NativePxHandler::new(Arc::new(AlwaysOkSolver) as Arc<dyn NativeSolver>, app_id());
        let page = PageHtml::new("https://www.pedidosya.com.ar/", "");
        let out = handler.solve(&page).await.expect("solve");
        assert_eq!(out.status, HandlerStatus::Solved);
        assert_eq!(out.cookies.set.len(), 1);
        assert_eq!(out.user_agent.as_deref(), Some("ua"));
    }
}
