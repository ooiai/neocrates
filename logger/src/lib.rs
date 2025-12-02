use chrono::{DateTime, Local, Utc};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub use tracing::*;

#[derive(Default, Clone, Copy)]
struct LocalTime;

impl tracing_subscriber::fmt::time::FormatTime for LocalTime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = Utc::now();
        let local_now: DateTime<Local> = now.with_timezone(&Local);
        write!(w, "{}", local_now.format("%Y-%m-%d %H:%M:%S"))
    }
}

pub async fn run() {
    // let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| panic!("RUST_LOG must be set!"));
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        .with_timer(LocalTime)
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::CLOSE)
        // .with_max_level(Level::DEBUG)
        // .with_max_level(Level::ERROR)
        // .with_max_level(Level::WARN)
        // .with_max_level(Level::TRACE)
        // completes the builder.
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
