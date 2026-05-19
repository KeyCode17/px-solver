//! Minimal `Set-Cookie` parser. We accept the subset the PX servers
//! emit — `name=value; Domain=…; Path=…; …`. We do NOT honour
//! Expires/Max-Age here; cookie freshness is the caller's concern via
//! `PxCookieBundle::expires_at`.

use px_core::NamedCookie;
use reqwest::header::HeaderValue;

pub fn parse_set_cookies(values: &[&HeaderValue], default_domain: &str) -> Vec<NamedCookie> {
    values
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter_map(|s| parse_one(s, default_domain))
        .collect()
}

fn parse_one(raw: &str, default_domain: &str) -> Option<NamedCookie> {
    let mut parts = raw.split(';');
    let first = parts.next()?.trim();
    let (name, value) = first.split_once('=')?;
    let name = name.trim().to_owned();
    let value = value.trim().to_owned();
    if name.is_empty() {
        return None;
    }
    let mut domain = default_domain.to_owned();
    let mut path = "/".to_owned();
    for attr in parts {
        let attr = attr.trim();
        if let Some(rest) = strip_ci_prefix(attr, "domain=") {
            domain = rest.trim_start_matches('.').to_owned();
        } else if let Some(rest) = strip_ci_prefix(attr, "path=") {
            path = rest.to_owned();
        }
    }
    Some(NamedCookie {
        name,
        value,
        domain,
        path,
    })
}

fn strip_ci_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.len() >= prefix.len()
        && s.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes())
    {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn hv(s: &str) -> HeaderValue {
        HeaderValue::from_str(s).expect("hv")
    }

    #[test]
    fn parses_px_cookie_bundle_shape() {
        let v1 = hv("_px3=abc123; Domain=.pedidosya.com.ar; Path=/; Secure");
        let v2 = hv("pxhc=def; domain=.example.com; path=/api");
        let refs: Vec<&HeaderValue> = vec![&v1, &v2];
        let out = parse_set_cookies(&refs, "www.pedidosya.com.ar");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].name, "_px3");
        assert_eq!(out[0].value, "abc123");
        assert_eq!(out[0].domain, "pedidosya.com.ar");
        assert_eq!(out[0].path, "/");
        assert_eq!(out[1].domain, "example.com");
        assert_eq!(out[1].path, "/api");
    }

    #[test]
    fn falls_back_to_default_domain() {
        let v = hv("only=val");
        let out = parse_set_cookies(&[&v], "www.test.com");
        assert_eq!(out[0].domain, "www.test.com");
        assert_eq!(out[0].path, "/");
    }

    #[test]
    fn skips_empty_name() {
        let v = hv("=novalue");
        assert!(parse_set_cookies(&[&v], "test").is_empty());
    }
}
