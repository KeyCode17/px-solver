use px_harvester::HarvestedCookie;

const PX_COOKIE_PREFIXES: &[&str] = &["_px", "_pxvid", "_pxhd", "_pxde"];

pub fn is_px_cookie(name: &str) -> bool {
    PX_COOKIE_PREFIXES.iter().any(|p| name.starts_with(p))
}

pub fn extract_px_cookies(cookies: &[HarvestedCookie]) -> Vec<HarvestedCookie> {
    cookies
        .iter()
        .filter(|c| is_px_cookie(&c.name))
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
    fn keeps_only_px_cookies() {
        let input = vec![
            cookie("_px3"),
            cookie("_pxhd"),
            cookie("_pxvid"),
            cookie("_pxde"),
            cookie("session_id"),
            cookie("dwsid"),
        ];
        let out = extract_px_cookies(&input);
        assert_eq!(out.len(), 4);
        assert!(out.iter().all(|c| is_px_cookie(&c.name)));
    }
}
