use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SolveRequestDto {
    pub url: String,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SolveResponseDto {
    pub user_agent: String,
    pub solve_ms: u64,
    pub cache_hit: bool,
    pub handler: String,
    pub cookies: Vec<CookieDto>,
    pub expires_at: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CookieDto {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthDto {
    pub status: &'static str,
    pub build_sha: &'static str,
    pub uptime_secs: u64,
}
