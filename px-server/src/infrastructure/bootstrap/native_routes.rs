//! Wire `SensorNativeSolver` into the solve dispatcher (ADR-0024, v1.8.0).
//!
//! Routes are loaded from `PX_NATIVE_PROFILES`, a comma-separated list of
//! `domain=path/to/profile.toml` pairs. For each entry, we instantiate a
//! [`SensorNativeSolver`] from the profile and decorate the existing
//! per-domain handler with [`NativeFirstHandler`] so the native path is
//! tried first and the Camoufox/Chromium harvester remains the fallback.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use px_core::PxAppId;
use px_native::infrastructure::{NativeFirstHandler, NativePxHandler};
use px_native::profile::TenantProfile;
use px_native::{NativeSolver, SensorNativeSolver};
use px_pipeline::ChallengeHandler;
use reqwest::Client;

use crate::application::routing::RoutingDispatcher;

pub struct NativeRoute {
    pub domain: String,
    pub profile_path: PathBuf,
}

pub fn parse_native_routes(raw: Option<&str>) -> Vec<NativeRoute> {
    raw.unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|spec| {
            let (domain, path) = spec.split_once('=')?;
            Some(NativeRoute {
                domain: domain.trim().to_lowercase(),
                profile_path: PathBuf::from(path.trim()),
            })
        })
        .collect()
}

pub fn apply_native_overlay(
    mut router: RoutingDispatcher,
    routes: Vec<NativeRoute>,
) -> Result<RoutingDispatcher> {
    if routes.is_empty() {
        return Ok(router);
    }
    let client = Client::builder()
        .build()
        .context("build reqwest client for native handler")?;
    for route in routes {
        let profile = TenantProfile::load(&route.profile_path).with_context(|| {
            format!(
                "load tenant profile {} for {}",
                route.profile_path.display(),
                route.domain
            )
        })?;
        let app_id = PxAppId::new(&profile.app_id).map_err(|e| {
            anyhow::anyhow!(
                "profile {} has invalid app_id: {e}",
                route.profile_path.display()
            )
        })?;
        let solver: Arc<dyn NativeSolver> =
            Arc::new(SensorNativeSolver::new(client.clone(), Arc::new(profile)));
        let native: Arc<dyn ChallengeHandler> = Arc::new(NativePxHandler::new(solver, app_id));
        let fallback = router
            .handler_for(&route.domain)
            .cloned()
            .unwrap_or_else(|| Arc::clone(router.default_handler()));
        let wrapped: Arc<dyn ChallengeHandler> =
            Arc::new(NativeFirstHandler::new(native, fallback));
        router = router.with_route(route.domain.clone(), wrapped);
        tracing::info!(domain = %route.domain, "native PX solver overlaid (native-first)");
    }
    Ok(router)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_pairs() {
        let r = parse_native_routes(Some(
            " pedidosya.com.ar=profiles/eT15wiaE.toml , foo.com=/x.toml ",
        ));
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].domain, "pedidosya.com.ar");
        assert_eq!(r[0].profile_path, PathBuf::from("profiles/eT15wiaE.toml"));
        assert_eq!(r[1].domain, "foo.com");
    }

    #[test]
    fn parse_unset_is_empty() {
        assert!(parse_native_routes(None).is_empty());
        assert!(parse_native_routes(Some("")).is_empty());
    }

    #[test]
    fn ignores_malformed_entries() {
        let r = parse_native_routes(Some("nopair, ok=p"));
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].domain, "ok");
    }
}
