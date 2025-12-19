use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const EPOCH: u64 = 1609459200000; // 2021-01-01 00:00:00 UTC in milliseconds
const WORKER_ID_BITS: u64 = 5;
const DATA_CENTER_ID_BITS: u64 = 5;
const SEQUENCE_BITS: u64 = 12;

const MAX_WORKER_ID: u64 = (1 << WORKER_ID_BITS) - 1;
const MAX_DATA_CENTER_ID: u64 = (1 << DATA_CENTER_ID_BITS) - 1;
const SEQUENCE_MASK: u64 = (1 << SEQUENCE_BITS) - 1;

const WORKER_ID_SHIFT: u64 = SEQUENCE_BITS;
const DATA_CENTER_ID_SHIFT: u64 = SEQUENCE_BITS + WORKER_ID_BITS;
const TIMESTAMP_SHIFT: u64 = SEQUENCE_BITS + WORKER_ID_BITS + DATA_CENTER_ID_BITS;

pub struct SnowflakeIdGenerator {
    worker_id: u64,
    data_center_id: u64,
    sequence: u64,
    last_timestamp: u64,
}

impl SnowflakeIdGenerator {
    pub fn new(worker_id: u64, data_center_id: u64) -> Self {
        if worker_id > MAX_WORKER_ID {
            panic!("worker_id can't be greater than {}", MAX_WORKER_ID);
        }
        if data_center_id > MAX_DATA_CENTER_ID {
            panic!(
                "data_center_id can't be greater than {}",
                MAX_DATA_CENTER_ID
            );
        }
        SnowflakeIdGenerator {
            worker_id,
            data_center_id,
            sequence: 0,
            last_timestamp: 0,
        }
    }

    pub fn generate(&mut self) -> u64 {
        let mut timestamp = current_time_millis();

        if timestamp < self.last_timestamp {
            timestamp = self.last_timestamp;
        }

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & SEQUENCE_MASK;
            if self.sequence == 0 {
                timestamp = self.wait_for_next_millis(self.last_timestamp);
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        let time_part = timestamp.saturating_sub(EPOCH);

        (time_part << TIMESTAMP_SHIFT)
            | (self.data_center_id << DATA_CENTER_ID_SHIFT)
            | (self.worker_id << WORKER_ID_SHIFT)
            | self.sequence
    }

    fn wait_for_next_millis(&self, last_timestamp: u64) -> u64 {
        let timestamp = current_time_millis();
        if timestamp <= last_timestamp {
            last_timestamp + 1
        } else {
            timestamp
        }
    }
}

fn current_time_millis() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis() as u64
}

use once_cell::sync::Lazy;

static ID_GENERATOR: Lazy<Mutex<SnowflakeIdGenerator>> = Lazy::new(|| {
    let generator = SnowflakeIdGenerator::new(1, 1);
    Mutex::new(generator)
});

pub fn generate_snowflake_uid() -> u64 {
    let mut generator = ID_GENERATOR.lock().expect("Failed to lock ID generator");
    generator.generate()
}

pub fn generate_snowflake_id() -> i64 {
    let mut generator = ID_GENERATOR.lock().expect("Failed to lock ID generator");
    generator.generate() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn snowflake_monotonic_and_unique() {
        let mut prev = generate_snowflake_uid();
        for _ in 0..50_000 {
            let id = generate_snowflake_uid();
            assert!(
                id > prev,
                "not strictly increasing: prev={}, curr={}",
                prev,
                id
            );
            prev = id;
        }

        let threads = 8;
        let per_thread = 3_000;
        let barrier = Arc::new(Barrier::new(threads));
        let handles: Vec<_> = (0..threads)
            .map(|_| {
                let b = barrier.clone();
                thread::spawn(move || {
                    // 让各线程尽量同时开始
                    b.wait();
                    let mut v = Vec::with_capacity(per_thread);
                    for _ in 0..per_thread {
                        v.push(generate_snowflake_uid());
                    }
                    v
                })
            })
            .collect();

        let mut all = Vec::with_capacity(threads * per_thread);
        for h in handles {
            let mut v = h.join().expect("thread panicked");
            all.append(&mut v);
        }

        let mut set = HashSet::with_capacity(all.len());
        for id in &all {
            assert!(set.insert(*id), "duplicate id {}", id);
        }

        all.sort_unstable();
        for i in 1..all.len() {
            assert!(
                all[i] > all[i - 1],
                "sorted not strictly increasing at {}: {} <= {}",
                i,
                all[i],
                all[i - 1]
            );
        }
    }
}
