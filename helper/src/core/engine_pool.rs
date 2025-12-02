use std::sync::atomic::{AtomicUsize, Ordering};

pub struct EnginePool {
    urls: Vec<String>,
    idx: AtomicUsize,
}

impl EnginePool {
    pub fn new(urls: Vec<String>) -> Self {
        Self {
            urls,
            idx: AtomicUsize::new(0),
        }
    }

    pub fn next_url(&self) -> Option<String> {
        let len = self.urls.len();
        if len == 0 {
            return None;
        }
        let i = self.idx.fetch_add(1, Ordering::Relaxed) % len;
        Some(self.urls[i].clone())
    }
}

// let pool = Arc::new(EnginePool::new(vec![
//     "http://a".to_string(),
//     "http://b".to_string(),
//     "http://c".to_string(),
// ]));

// let url = pool.next_url();
