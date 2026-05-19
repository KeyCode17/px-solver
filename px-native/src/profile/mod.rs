//! Tenant profile: per-`appId` parameters that select the right
//! cipher constants, sensor endpoint, and fallback secret. Loaded
//! from a TOML file at startup so adding a tenant is a config change,
//! not a code change (ADR-0024 §N5).

pub mod schema;

pub use schema::{TenantProfile, XorKeys};
