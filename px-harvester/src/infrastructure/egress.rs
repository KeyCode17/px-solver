//! Egress-proxy handling shared by every [`crate::Harvester`] implementation.
//!
//! A harvest leaves through the proxy named on its [`crate::HarvestRequest`]
//! and no other. Rotation across a pool belongs to long-lived browser
//! sessions, not to solving: a `_px3` bundle is bound to the IP that earned
//! it, so a caller who did not name an egress could not use a bundle
//! harvested through a rotating one.

/// Drop `user:pass@` from a proxy URL, warning that the credentials are
/// being discarded.
///
/// Neither browser engine can answer a proxy `407` from userinfo in the
/// URL: geckodriver's W3C `proxy` capability has no credential field, and
/// Chromium's `--proxy-server` ignores them without a CDP
/// `Fetch.authRequired` handler. Passing them through anyway fails the
/// whole harvest with no visible cause, so an authenticated upstream has
/// to be fronted by a local unauthenticated relay (gost, 3proxy).
pub fn strip_credentials(proxy: String) -> String {
    let Some((scheme, rest)) = proxy.split_once("://") else {
        return proxy;
    };
    let Some((_userinfo, host_port)) = rest.rsplit_once('@') else {
        return proxy;
    };
    let sanitized = format!("{scheme}://{host_port}");
    tracing::warn!(
        proxy = %sanitized,
        "proxy credentials discarded: no browser engine can authenticate them; front the upstream with a local unauthenticated relay"
    );
    sanitized
}

/// Normalize a proxy URL into a Chromium `--proxy-server` spec.
///
/// Chromium understands `http`, `https`, `socks4` and `socks5` — but not
/// `socks5h`, which is a curl convention that geckodriver's capability
/// layer accepts. Chromium *silently ignores* a spec it cannot parse and
/// goes direct, which is the failure this whole path exists to remove, so
/// the unknown scheme is rewritten rather than passed through.
pub fn chromium_proxy_spec(proxy: &str) -> String {
    match proxy.split_once("://") {
        Some(("socks5h", host_port)) => format!("socks5://{host_port}"),
        _ => proxy.to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn drops_userinfo_from_http_and_socks_urls() {
        assert_eq!(
            strip_credentials("http://u:p@rotating.example:8080".into()),
            "http://rotating.example:8080"
        );
        assert_eq!(
            strip_credentials("socks5://a:b@x.example:1080".into()),
            "socks5://x.example:1080"
        );
    }

    #[test]
    fn leaves_unauthenticated_urls_alone() {
        assert_eq!(
            strip_credentials("http://plain.example:8080".into()),
            "http://plain.example:8080"
        );
        assert_eq!(strip_credentials("host:8080".into()), "host:8080");
    }

    #[test]
    fn chromium_spec_rewrites_socks5h_which_chromium_cannot_parse() {
        assert_eq!(
            chromium_proxy_spec("socks5h://x.example:1080"),
            "socks5://x.example:1080"
        );
    }

    #[test]
    fn chromium_spec_passes_supported_schemes_through() {
        for proxy in [
            "http://x.example:8080",
            "https://x.example:8443",
            "socks4://x.example:1080",
            "socks5://x.example:1080",
            "x.example:8080",
        ] {
            assert_eq!(chromium_proxy_spec(proxy), proxy);
        }
    }
}
