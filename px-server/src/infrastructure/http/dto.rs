use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct SolveRequestDto {
    pub url: String,
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FetchRequestDto {
    pub url: String,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FetchResponseDto {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub duration_ms: u64,
    pub handler: String,
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
