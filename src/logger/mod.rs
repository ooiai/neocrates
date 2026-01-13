use chrono::Local;
use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub use tracing::*;

#[derive(Default, Clone, Copy)]
struct LocalTime;

impl tracing_subscriber::fmt::time::FormatTime for LocalTime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    #[serde(rename = "rust-log")]
    pub log: LogSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct LogSettings {
    #[serde(default = "default_level")]
    pub level: String,
    #[serde(default = "default_true")]
    pub target: bool,
    #[serde(default = "default_true")]
    pub thread_ids: bool,
    #[serde(default = "default_true")]
    pub line_number: bool,
    #[serde(default = "default_true")]
    pub file: bool,
    #[serde(default = "default_true")]
    pub pretty: bool,
    // 是否打印 SQL
    #[serde(default)]
    pub sql_log: Option<bool>,
}

impl LogConfig {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let f = std::fs::File::open(path)?;
        let config: LogConfig = serde_yaml::from_reader(f)?;
        Ok(config)
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log: LogSettings::default(),
        }
    }
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            level: default_level(),
            target: true,
            thread_ids: true,
            line_number: true,
            file: true,
            pretty: true,
            sql_log: Some(true),
        }
    }
}

fn default_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

pub fn init(config: LogConfig) {
    let config = config.log;
    if let Some(on) = config.sql_log {
        crate::dieselhelper::logging::set_sql_logging(on);
    }
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

    let builder = FmtSubscriber::builder()
        .with_timer(LocalTime)
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::CLOSE)
        .with_target(config.target)
        .with_thread_ids(config.thread_ids)
        .with_line_number(config.line_number)
        .with_file(config.file);

    if config.pretty {
        let subscriber = builder.pretty().finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    } else {
        let subscriber = builder.finish();
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    }
}

pub async fn run() {
    init(LogConfig::default());
}
