//! Native `NativeSolver` implementation that builds a sensor payload
//! locally and POSTs it to `${origin}/${app_id_tag}${sensor_path}`,
//! then parses the `Set-Cookie` response into a [`PxCookieBundle`].

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use px_core::{NamedCookie, PxCookieBundle};
use px_errors::AppError;
use reqwest::Client;
use reqwest::header::{
    ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, HeaderValue, ORIGIN, REFERER, SET_COOKIE, USER_AGENT,
};
use url::Url;

use crate::cipher::encrypt_sensor;
use crate::domain::native_solver::{NativeSolver, SolveContext};
use crate::events::{SyntheticIdentity, default_batch};
use crate::infrastructure::cookies::parse_set_cookies;
use crate::profile::TenantProfile;

/// HTTP-backed native solver bound to a single tenant profile.
pub struct SensorNativeSolver {
    client: Client,
    profile: Arc<TenantProfile>,
    cookie_ttl: Duration,
}

impl SensorNativeSolver {
    pub fn new(client: Client, profile: Arc<TenantProfile>) -> Self {
        Self {
            client,
            profile,
            cookie_ttl: Duration::from_secs(300),
        }
    }

    pub fn with_cookie_ttl(mut self, ttl: Duration) -> Self {
        self.cookie_ttl = ttl;
        self
    }

    /// Build the wire-format payload for the given context.
    pub fn build_payload(&self, ctx: &SolveContext) -> Result<Vec<u8>, AppError> {
        let identity = identity_from_fingerprint(ctx);
        let now_ms = epoch_ms_now();
        let batch = default_batch(&identity, now_ms);
        let json = serde_json::to_vec(&batch)
            .map_err(|e| AppError::InternalError(format!("serialize sensor batch: {e}")))?;
        let pf = if ctx.url.is_empty() {
            self.profile.pf_fallback.as_bytes().to_vec()
        } else {
            ctx.url.as_bytes().to_vec()
        };
        let cu = uuid::Uuid::now_v1(&[0u8; 6]).to_string();
        encrypt_sensor(&json, &pf, cu.as_bytes())
    }
}

#[async_trait]
impl NativeSolver for SensorNativeSolver {
    async fn solve(&self, ctx: &SolveContext) -> Result<PxCookieBundle, AppError> {
        let url =
            Url::parse(&ctx.url).map_err(|e| AppError::BadRequest(format!("invalid url: {e}")))?;
        let origin = format!(
            "{}://{}",
            url.scheme(),
            url.host_str()
                .ok_or_else(|| AppError::BadRequest("url has no host".into()))?,
        );
        let sensor_url = self.profile.sensor_url(&origin);
        let payload = self.build_payload(ctx)?;

        let resp = self
            .client
            .post(&sensor_url)
            .header(USER_AGENT, header(&ctx.fingerprint.user_agent)?)
            .header(ACCEPT, HeaderValue::from_static("*/*"))
            .header(
                ACCEPT_LANGUAGE,
                header(&ctx.fingerprint.accept_language.join(","))?,
            )
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            )
            .header(ORIGIN, header(&origin)?)
            .header(REFERER, header(&ctx.url)?)
            .body(payload)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("sensor POST: {e}")))?;

        if !resp.status().is_success() {
            return Err(AppError::InternalError(format!(
                "sensor POST returned status {}",
                resp.status()
            )));
        }

        let set_cookie_values: Vec<&HeaderValue> =
            resp.headers().get_all(SET_COOKIE).iter().collect();
        let cookies: Vec<NamedCookie> =
            parse_set_cookies(&set_cookie_values, url.host_str().unwrap_or(""));
        if cookies.is_empty() {
            return Err(AppError::InternalError(
                "sensor POST returned no Set-Cookie headers".into(),
            ));
        }
        tracing::info!(target: "px_native", url = %sensor_url, count = cookies.len(), "native sensor solved");
        Ok(PxCookieBundle::new(
            cookies,
            ctx.fingerprint.user_agent.clone(),
            SystemTime::now(),
            self.cookie_ttl,
        ))
    }
}

fn header(value: &str) -> Result<HeaderValue, AppError> {
    HeaderValue::from_str(value).map_err(|e| AppError::BadRequest(format!("bad header value: {e}")))
}

fn identity_from_fingerprint(ctx: &SolveContext) -> SyntheticIdentity {
    SyntheticIdentity {
        user_agent: ctx.fingerprint.user_agent.clone(),
        locale: ctx
            .fingerprint
            .accept_language
            .first()
            .cloned()
            .unwrap_or_else(|| "en-US".into()),
        timezone: ctx.fingerprint.timezone.clone(),
        viewport: (ctx.fingerprint.screen_width, ctx.fingerprint.screen_height),
        ga_client_id: format!("GA1.1.{}", ctx.fingerprint.key_hash()),
        session_count: 1,
        first_visit_days_ago: 7,
    }
}

fn epoch_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
