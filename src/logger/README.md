# Logger Module

The `logger` module bootstraps `tracing-subscriber` for Neocrates applications. It provides local-time formatting, optional pretty output, and a small YAML-friendly config structure.

See also: [root README](../../README.md)

---

## Feature

Enable with:

```toml
neocrates = { version = "0.1", default-features = false, features = ["logger"] }
```

---

## What you get

- `LogConfig` and `LogSettings` for structured configuration
- `LogConfig::load(path)` to read YAML
- `init(config)` to install the global tracing subscriber
- `run()` to install the default config
- `pub use tracing::*` so downstream code can use `info!`, `warn!`, `error!`, and friends

Important behavior:

- `RUST_LOG` overrides the configured level via `EnvFilter`
- `sql_log` toggles **Diesel SQL logging** when the `diesel` feature is enabled
- output is written through `tracing-subscriber` formatting; there is no built-in file rotation layer

---

## Quick start

```rust
#[tokio::main]
async fn main() {
    neocrates::logger::run().await;
    neocrates::logger::info!("neocrates logger ready");
}
```

---

## Step-by-step tutorial

## 1. Start with the built-in default

```rust
#[tokio::main]
async fn main() {
    neocrates::logger::run().await;
}
```

This uses:

- level: `info`
- pretty output: enabled
- target/thread-id/file/line output: enabled

## 2. Load settings from YAML

`LogConfig` is serde-friendly:

```yaml
rust-log:
  level: info
  target: true
  thread-ids: true
  line-number: true
  file: true
  pretty: true
  sql-log: true
```

```rust
use neocrates::logger::{LogConfig, init};

fn main() -> neocrates::anyhow::Result<()> {
    let cfg = LogConfig::load("log.yml")?;
    init(cfg);
    Ok(())
}
```

## 3. Let `RUST_LOG` override the config file

```bash
RUST_LOG=debug cargo run
```

This is useful when you want more logs without editing the YAML file.

---

## Key points and gotchas

- `init()` installs a **global** subscriber and will panic if you call it twice in the same process.
- `LocalTime` uses the system’s local timezone for formatting.
- `sql_log` currently controls Diesel query logging only. SQLx logging is configured separately through `sqlxhelper::logging::set_sql_logging(...)`.
- The module configures output formatting, not file sinks or rotation.

---

## Roadmap

Potential improvements:

1. Add JSON output mode for structured log shipping.
2. Add explicit integration knobs for SQLx logging.
3. Offer a builder-style API alongside the YAML config structs.
4. Provide documented patterns for file appenders and rotation.
