use std::fmt::Debug;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Define the retry strategy trait
pub trait RetryStrategy {
    fn next_delay(&mut self) -> Option<Duration>;
}

/// Custom interval-based retry strategy
pub struct RetryIntervals {
    intervals: Vec<Duration>,
    current: usize,
}

impl RetryIntervals {
    pub fn new(intervals: Vec<Duration>) -> Self {
        Self {
            intervals,
            current: 0,
        }
    }

    /// Create a fixed interval strategy
    /// count: number of retries (excluding the initial attempt)
    /// duration: wait time between attempts
    pub fn fixed(count: usize, duration: Duration) -> Self {
        Self {
            intervals: vec![duration; count],
            current: 0,
        }
    }
}

impl RetryStrategy for RetryIntervals {
    fn next_delay(&mut self) -> Option<Duration> {
        if self.current < self.intervals.len() {
            let delay = self.intervals[self.current];
            self.current += 1;
            Some(delay)
        } else {
            None
        }
    }
}

/// Generic retry function
///
/// # Arguments
/// * `operation` - A closure that returns a Future. The Future must return a Result.
/// * `strategy` - The retry strategy (e.g., RetryIntervals).
///
/// # Example
/// ```rust,ignore
/// use std::time::Duration;
/// use neocrates::helper::core::retry::{retry_async, RetryIntervals};
///
/// async fn do_something() -> Result<(), String> {
///     // ...
///     Err("fail".to_string())
/// }
///
/// async fn main() {
///     // Retry 3 times with 1 second interval
///     let strategy = RetryIntervals::fixed(3, Duration::from_secs(1));
///
///     let result = retry_async(|| do_something(), strategy).await;
/// }
/// ```
pub async fn retry_async<F, Fut, T, E, S>(mut operation: F, mut strategy: S) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Debug,
    S: RetryStrategy,
{
    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => match strategy.next_delay() {
                Some(delay) => {
                    tracing::warn!("Operation failed: {:?}. Retrying in {:?}", e, delay);
                    sleep(delay).await;
                }
                None => return Err(e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_retry_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let operation = || async {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err("fail")
            } else {
                Ok("success")
            }
        };

        // Retry 3 times, should succeed on 3rd attempt (index 2)
        let strategy = RetryIntervals::fixed(3, Duration::from_millis(10));
        let result = retry_async(operation, strategy).await;

        assert_eq!(result, Ok("success"));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fail() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let operation = || async {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err::<(), &str>("always fail")
        };

        // Retry 2 times (total 3 attempts)
        let strategy = RetryIntervals::fixed(2, Duration::from_millis(10));
        let result = retry_async(operation, strategy).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }
}
