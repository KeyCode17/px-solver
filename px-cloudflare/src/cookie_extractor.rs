use px_harvester::HarvestedCookie;

/// Cookie name prefixes the CloudflareHandler considers session-relevant.
///
/// Includes CF Bot Management (`cf_clearance`, `__cf_bm`) plus the PX family
/// because pedidosya-style CF→PX cascades set both during a single bypass.
const SESSION_COOKIE_PREFIXES: &[&str] = &[
    "cf_clearance",
    "__cf_bm",
    "_px",
    "_pxvid",
    "_pxhd",
    "_pxde",
    "pxcts",
];

pub fn is_session_cookie(name: &str) -> bool {
    SESSION_COOKIE_PREFIXES.iter().any(|p| name.starts_with(p))
}

pub fn extract_session_cookies(cookies: &[HarvestedCookie]) -> Vec<HarvestedCookie> {
    cookies
        .iter()
        .filter(|c| is_session_cookie(&c.name))
        .cloned()
        .collect()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    fn cookie(name: &str) -> HarvestedCookie {
        HarvestedCookie {
            name: name.into(),
            value: "v".into(),
            domain: "x.com".into(),
            path: "/".into(),
        }
    }

    #[test]
    fn keeps_cf_and_px_families() {
        let input = vec![
            cookie("cf_clearance"),
            cookie("__cf_bm"),
            cookie("_px3"),
            cookie("_pxhd"),
            cookie("_pxvid"),
            cookie("pxcts"),
            cookie("_pxde"),
            cookie("peyas.sid"),
            cookie("dhhPerseusEnvironment"),
        ];
        let out = extract_session_cookies(&input);
        let names: Vec<&str> = out.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(out.len(), 7);
        assert!(names.contains(&"cf_clearance"));
        assert!(names.contains(&"__cf_bm"));
        assert!(names.contains(&"_px3"));
        assert!(names.contains(&"pxcts"));
        assert!(!names.contains(&"peyas.sid"));
        assert!(!names.contains(&"dhhPerseusEnvironment"));
    }
}
