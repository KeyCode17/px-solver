//! Inputs to the event collector that come from the synthetic-user
//! pool (see `px-camoufox/src/infrastructure/synthetic_user.rs`).
//! Kept as a standalone struct so px-native doesn't depend on the
//! camoufox crate.

#[derive(Debug, Clone)]
pub struct SyntheticIdentity {
    pub user_agent: String,
    pub locale: String,
    pub timezone: String,
    pub viewport: (u32, u32),
    pub ga_client_id: String,
    pub session_count: u32,
    /// Visit-history depth in days. Drives the `first_visit_days_ago`
    /// timestamp the runtime sometimes attaches.
    pub first_visit_days_ago: u32,
}

impl SyntheticIdentity {
    /// Default identity for use in unit tests. Real production
    /// identities flow in from the camoufox `SyntheticUserPool`.
    pub fn test_default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (X11; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0"
                .to_owned(),
            locale: "es-AR".to_owned(),
            timezone: "America/Argentina/Buenos_Aires".to_owned(),
            viewport: (1366, 768),
            ga_client_id: "GA1.1.000000000.0000000000".to_owned(),
            session_count: 3,
            first_visit_days_ago: 14,
        }
    }
}
