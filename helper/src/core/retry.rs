// use std::fmt::Debug;
// use std::future::Future;
// use std::time::Duration;
// use tokio::time::sleep;

// // 定义重试策略的 trait
// pub trait RetryStrategy {
//     fn next_delay(&mut self) -> Option<Duration>;
// }

// // 自定义间隔重试策略
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

// // 通用的重试函数
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
// //     // 定义自定义间隔重试策略
// //     let retry_strategy = RetryIntervals::new(vec![
// //         Duration::from_secs(2),
// //         Duration::from_secs(60),
// //         Duration::from_secs(300),
// //         Duration::from_secs(1800),
// //     ]);

// //     // 定义需要重试的操作
// //     let operation = || async {
// //         println!("Attempting operation...");
// //         // 模拟一个可能失败的操作
// //         Err::<(), &str>("Operation failed")
// //     };

// //     // 使用重试策略执行操作
// //     let result = retry_async(operation, retry_strategy).await;

// //     match result {
// //         Ok(_) => println!("Operation succeeded"),
// //         Err(e) => println!("Operation failed after retries: {:?}", e),
// //     }
// // }
