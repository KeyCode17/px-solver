//! Data model for a single sensor event. The runtime emits arrays of
//! these and the encryptor JSON-stringifies the array before XOR.
//!
//! Field values are deliberately constrained to the JSON value kinds
//! we have actually observed inside `o.d[…]` assignments in the
//! deobfuscated init.js (line 7706-7738): strings, integers, booleans,
//! the JSON `null`, and one level of nested objects.

use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum EventField {
    Null,
    Bool(bool),
    Int(i64),
    String(String),
    Object(BTreeMap<String, EventField>),
}

impl From<bool> for EventField {
    fn from(b: bool) -> Self {
        EventField::Bool(b)
    }
}

impl From<i64> for EventField {
    fn from(n: i64) -> Self {
        EventField::Int(n)
    }
}

impl From<u64> for EventField {
    fn from(n: u64) -> Self {
        EventField::Int(n as i64)
    }
}

impl From<&str> for EventField {
    fn from(s: &str) -> Self {
        EventField::String(s.to_owned())
    }
}

impl From<String> for EventField {
    fn from(s: String) -> Self {
        EventField::String(s)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SensorEvent {
    pub t: String,
    pub d: BTreeMap<String, EventField>,
}

impl SensorEvent {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            t: tag.into(),
            d: BTreeMap::new(),
        }
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<EventField>) -> Self {
        self.d.insert(key.into(), value.into());
        self
    }
}
