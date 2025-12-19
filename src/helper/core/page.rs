use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PageParams {
    pub current: Option<i64>,
    pub size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct PageResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub current: i64,
    pub size: i64,
}
