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

pub fn to_offset_limit(current: usize, size: usize) -> (usize, usize, i64, i64) {
    let current = if current == 0 { 1 } else { current };
    let size = if size == 0 { 10 } else { size };
    let offset = ((current - 1) * size) as i64;
    let limit = size as i64;
    (current, size, offset, limit)
}
