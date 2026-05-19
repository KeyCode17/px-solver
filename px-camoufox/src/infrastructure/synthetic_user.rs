//! Synthetic-user pool for Camoufox sessions.
//!
//! Each `PersistentSession` is bound to one `SyntheticUser` for its
//! lifetime. The user carries the kind of "lived-in browser" signals
//! that PerimeterX scores positively: long-running analytics IDs (GA,
//! Facebook), prior pageview counters in `localStorage`, a stable
//! viewport + locale that matches the egress IP geo.
//!
//! Fingerprints already vary per Camoufox process (canvas / WebGL /
//! audio); what we add here is the *persistent* layer those engines
//! don't synthesize: account-like cookies and storage that look like a
//! returning visitor.

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone, Debug)]
pub(crate) struct SyntheticUser {
    pub id: String,
    pub locale: String,
    /// Built once at construction so callers don't recompute it; not
    /// currently injected into the caps (locale alone is enough for
    /// the Accept-Language header today).
    #[allow(dead_code)]
    pub accept_languages: String,
    pub viewport: (u32, u32),
    /// Future use: pass into Firefox prefs once Camoufox supports a
    /// timezone override beyond the OS default.
    #[allow(dead_code)]
    pub timezone: String,
    pub ga_client_id: String,
    pub gid: String,
    pub fbp: String,
    /// Days-ago this synthetic visitor "first visited" pedidosya.
    pub first_visit_days_ago: u32,
    /// Synthetic visit count (used to seed `peya:sessions` etc.).
    pub session_count: u32,
}

impl SyntheticUser {
    /// Build a deterministic-but-varied user from an index. Same index
    /// always yields the same user so session reuse keeps the persona
    /// stable across requests.
    pub fn from_index(idx: usize) -> Self {
        let viewports = [
            (1280, 720),
            (1366, 768),
            (1440, 900),
            (1536, 864),
            (1600, 900),
            (1920, 1080),
        ];
        let locales = ["es-AR", "es-AR", "es-AR", "es-AR", "es-MX", "es-AR"];
        let timezones = [
            "America/Argentina/Buenos_Aires",
            "America/Argentina/Cordoba",
            "America/Argentina/Mendoza",
        ];
        let v = viewports[idx % viewports.len()];
        let locale = locales[idx % locales.len()].to_string();
        let tz = timezones[idx % timezones.len()].to_string();
        Self {
            id: format!("syn-{idx:04}"),
            locale: locale.clone(),
            accept_languages: accept_language_chain(&locale),
            viewport: v,
            timezone: tz,
            ga_client_id: format!(
                "{}.{}",
                100_000_000 + idx * 7919,
                1_700_000_000 - idx as u64 * 86_400
            ),
            gid: format!(
                "GA1.2.{}.{}",
                1_000_000_000 + idx * 31,
                1_750_000_000 - idx as u64 * 1024
            ),
            fbp: format!(
                "fb.1.{}.{}",
                1_700_000_000 - idx as u64 * 600,
                idx * 314_159 % 999_999
            ),
            first_visit_days_ago: 3 + (idx as u32 % 27),
            session_count: 2 + (idx as u32 % 18),
        }
    }
}

fn accept_language_chain(locale: &str) -> String {
    match locale {
        "es-AR" => "es-AR,es;q=0.9,en-US;q=0.7,en;q=0.5".into(),
        "es-MX" => "es-MX,es;q=0.9,en-US;q=0.7,en;q=0.5".into(),
        other => other.into(),
    }
}

pub(crate) struct SyntheticUserPool {
    users: Vec<SyntheticUser>,
    cursor: AtomicUsize,
}

impl SyntheticUserPool {
    pub fn new(size: usize) -> Self {
        let size = size.max(1);
        Self {
            users: (0..size).map(SyntheticUser::from_index).collect(),
            cursor: AtomicUsize::new(0),
        }
    }

    pub fn next(&self) -> SyntheticUser {
        let idx = self.cursor.fetch_add(1, Ordering::Relaxed) % self.users.len();
        self.users[idx].clone()
    }

    pub fn len(&self) -> usize {
        self.users.len()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_from_index() {
        let a = SyntheticUser::from_index(7);
        let b = SyntheticUser::from_index(7);
        assert_eq!(a.id, b.id);
        assert_eq!(a.ga_client_id, b.ga_client_id);
    }

    #[test]
    fn pool_round_robins() {
        let pool = SyntheticUserPool::new(3);
        let ids: Vec<_> = (0..6).map(|_| pool.next().id).collect();
        assert_eq!(ids[0], ids[3]);
        assert_eq!(ids[1], ids[4]);
        assert_eq!(ids[2], ids[5]);
    }
}
