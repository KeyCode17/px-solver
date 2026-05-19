//! Cipher core for the eT15wiaE px-3 sensor (ADR-0024, N1).
//!
//! ```text
//! events ── JSON.stringify ── jw(·, IS=50) ── h_p ── v_q(salt, ·, v_n(len, secret)) ── url-encoded payload
//! ```

pub mod b64;
pub mod offsets;
pub mod remap;
pub mod secret;
pub mod splice;
pub mod xor;

pub use b64::h_p;
pub use offsets::v_n;
pub use remap::v_m;
pub use secret::v_l;
pub use splice::v_q;
pub use xor::{IS, VJ, jw};
