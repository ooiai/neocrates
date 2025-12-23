// use std::fmt::Debug;
// use std::future::Future;
// use std::time::Duration;
// use tokio::time::sleep;

// // Define the retry strategy trait
// pub trait RetryStrategy {
//     fn next_delay(&mut self) -> Option<Duration>;
// }

// // Custom interval-based retry strategy
// pub struct RetryIntervals {
//     intervals: Vec<Duration>,
//     current: usize,
// }

// impl RetryIntervals {
//     pub fn new(intervals: Vec<Duration>) -> Self {
//         Self {
//             intervals,
//             current: 0,
//         }
//     }
// }

// impl RetryStrategy for RetryIntervals {
//     fn next_delay(&mut self) -> Option<Duration> {
//         if self.current < self.intervals.len() {
//             let delay = self.intervals[self.current];
//             self.current += 1;
//             Some(delay)
//         } else {
//             None
//         }
//     }
// }

// // Generic retry function
// pub async fn retry_async<F, Fut, T, E, S>(mut operation: F, mut strategy: S) -> Result<T, E>
// where
//     F: FnMut() -> Fut,
//     Fut: Future<Output = Result<T, E>>,
//     E: Debug,
//     S: RetryStrategy,
// {
//     while let Some(delay) = strategy.next_delay() {
//         match operation().await {
//             Ok(result) => return Ok(result),
//             Err(e) => {
//                 println!("Operation failed: {:?}. Retrying in {:?}", e, delay);
//                 sleep(delay).await;
//             }
//         }
//     }
//     operation().await
// }

// // #[tokio::main]
// // async fn main() {
// //     // Define a custom interval retry strategy
// //     let retry_strategy = RetryIntervals::new(vec![
// //         Duration::from_secs(2),
// //         Duration::from_secs(60),
// //         Duration::from_secs(300),
// //         Duration::from_secs(1800),
// //     ]);

// //     // Define the operation that may need retries
// //     let operation = || async {
// //         println!("Attempting operation...");
// //         // Simulate an operation that may fail
// //         Err::<(), &str>("Operation failed")
// //     };

// //     // Execute the operation with the retry strategy
// //     let result = retry_async(operation, retry_strategy).await;

// //     match result {
// //         Ok(_) => println!("Operation succeeded"),
// //         Err(e) => println!("Operation failed after retries: {:?}", e),
// //     }
// // }
