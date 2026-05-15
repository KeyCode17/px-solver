use serde::{Deserialize, Serialize};

#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SingleResponse<T> {
    pub data: T,
    pub message: String,
}

impl<T> SingleResponse<T> {
    pub fn new(data: T, message: impl Into<String>) -> Self {
        Self {
            data,
            message: message.into(),
        }
    }
}

#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ListResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
}

impl<T> ListResponse<T> {
    pub fn new(data: Vec<T>, pagination: PaginationMeta) -> Self {
        Self { data, pagination }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct PaginationMeta {
    pub page: u32,
    pub size: u32,
    pub total: u64,
}

impl PaginationMeta {
    pub fn new(page: u32, size: u32, total: u64) -> Self {
        Self { page, size, total }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn single_response_serializes() {
        let r = SingleResponse::new(42_i32, "ok");
        let json = serde_json::to_string(&r).expect("serialize");
        assert!(json.contains("\"data\":42"));
        assert!(json.contains("\"message\":\"ok\""));
    }

    #[test]
    fn list_response_carries_pagination() {
        let r = ListResponse::new(vec![1_i32, 2, 3], PaginationMeta::new(0, 3, 3));
        assert_eq!(r.data.len(), 3);
        assert_eq!(r.pagination.total, 3);
    }
}
