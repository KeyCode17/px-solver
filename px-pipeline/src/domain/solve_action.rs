//! What a [`crate::ChallengeHandler`] is asked to solve.
//!
//! Detection only ever needs the page, but solving also needs the egress
//! the operator picked for this request, so the action carries both. It is
//! built at the infrastructure edge (the HTTP handler) and passed inward;
//! handlers never read transport types.

use crate::domain::page_html::PageHtml;

#[derive(Debug, Clone)]
pub struct SolveAction {
    pub page: PageHtml,
    pub proxy: Option<String>,
}

impl SolveAction {
    pub fn new(page: PageHtml) -> Self {
        Self { page, proxy: None }
    }

    /// Pin this solve to one egress proxy. `None` leaves the choice to the
    /// harvester's rotation.
    #[must_use]
    pub fn with_proxy(mut self, proxy: Option<String>) -> Self {
        self.proxy = proxy;
        self
    }

    pub fn url(&self) -> &str {
        &self.page.url
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_no_proxy() {
        let action = SolveAction::new(PageHtml::new("https://example.com", ""));
        assert_eq!(action.url(), "https://example.com");
        assert!(action.proxy.is_none());
    }

    #[test]
    fn carries_the_requested_proxy() {
        let action = SolveAction::new(PageHtml::new("https://example.com", ""))
            .with_proxy(Some("socks5://127.0.0.1:9050".into()));
        assert_eq!(action.proxy.as_deref(), Some("socks5://127.0.0.1:9050"));
    }
}
