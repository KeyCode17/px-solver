//! Synthetic event collector. Produces the JSON event batch the
//! sensor encryptor consumes.
//!
//! The exact mapping from PX event tags to data fields is still
//! partially opaque (it needs a captured XHR-hook trace to confirm —
//! see ADR-0024 N3 follow-up). What we model here is the **shape**:
//! a `Vec<SensorEvent>` where each event carries a `t` tag string and
//! a `BTreeMap` of base64-keyed fields. The encoder needs only that
//! structure; per-tenant population is a profile concern (N5).

pub mod batch;
pub mod identity;
pub mod model;

pub use batch::default_batch;
pub use identity::SyntheticIdentity;
pub use model::{EventField, SensorEvent};
