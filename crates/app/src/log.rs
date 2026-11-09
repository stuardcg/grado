//! Starts logging (ADR 0012): every `tracing` event the level lets through goes to stdout, in the
//! configured format.

use crate::config::{LogConfig, LogFormat};

pub fn start(config: LogConfig) {
    let logs = tracing_subscriber::fmt().with_env_filter(config.level);
    match config.format {
        LogFormat::Json => logs.json().init(),
        LogFormat::Pretty => logs.pretty().init(),
    }
}
