#[derive(Debug, Clone)]
pub struct PageHtml {
    pub url: String,
    pub html: String,
}

impl PageHtml {
    pub fn new(url: impl Into<String>, html: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            html: html.into(),
        }
    }
}
