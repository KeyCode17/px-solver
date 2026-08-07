//! Per-proxy `reqwest` clients for the native sensor path.
//!
//! A `Client` owns its connection pool, and the proxy is fixed at build
//! time, so honouring a per-request egress means one client per proxy.
//! They are cached because rebuilding one per solve would throw away the
//! pooled TLS connection to the sensor endpoint.
//!
//! Unlike the browser paths, `reqwest` does implement proxy
//! authentication, so `user:pass@` in the URL is honoured here.

use std::collections::HashMap;
use std::sync::Mutex;

use px_errors::AppError;
use reqwest::{Client, Proxy};

pub struct ProxyClients {
    direct: Client,
    proxied: Mutex<HashMap<String, Client>>,
}

impl ProxyClients {
    pub fn new(direct: Client) -> Self {
        Self {
            direct,
            proxied: Mutex::new(HashMap::new()),
        }
    }

    /// Client whose egress is `proxy`, or the direct one when `None`.
    pub fn for_proxy(&self, proxy: Option<&str>) -> Result<Client, AppError> {
        let Some(proxy) = proxy else {
            return Ok(self.direct.clone());
        };
        let mut cache = self
            .proxied
            .lock()
            .map_err(|e| AppError::InternalError(format!("proxy client cache poisoned: {e}")))?;
        if let Some(client) = cache.get(proxy) {
            return Ok(client.clone());
        }
        let built = Client::builder()
            .proxy(
                Proxy::all(proxy)
                    .map_err(|e| AppError::BadRequest(format!("invalid proxy url: {e}")))?,
            )
            .build()
            .map_err(|e| AppError::InternalError(format!("build proxied client: {e}")))?;
        cache.insert(proxy.to_string(), built.clone());
        Ok(built)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn clients() -> ProxyClients {
        ProxyClients::new(Client::builder().build().expect("build direct client"))
    }

    #[test]
    fn no_proxy_yields_the_direct_client() {
        assert!(clients().for_proxy(None).is_ok());
    }

    #[test]
    fn a_proxy_url_builds_and_caches_one_client() {
        let clients = clients();
        assert!(clients.for_proxy(Some("http://127.0.0.1:8080")).is_ok());
        assert!(clients.for_proxy(Some("http://127.0.0.1:8080")).is_ok());
        assert_eq!(clients.proxied.lock().expect("cache not poisoned").len(), 1);
    }

    #[test]
    fn a_malformed_proxy_url_is_a_bad_request() {
        let err = clients()
            .for_proxy(Some("not a url"))
            .expect_err("malformed proxy must fail");
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
