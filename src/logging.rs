//! Structured logging setup for dooz-code

use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Log level configuration
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    pub level: LogLevel,
    pub json_output: bool,
    pub include_target: bool,
    pub include_file: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            json_output: false,
            include_target: true,
            include_file: false,
        }
    }
}

impl LogConfig {
    pub fn development() -> Self {
        Self {
            level: LogLevel::Debug,
            json_output: false,
            include_target: true,
            include_file: true,
        }
    }
    
    pub fn production() -> Self {
        Self {
            level: LogLevel::Info,
            json_output: true,
            include_target: false,
            include_file: false,
        }
    }
}

/// Initialize logging with configuration
pub fn init_logging(config: &LogConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            let level = match config.level {
                LogLevel::Trace => "trace",
                LogLevel::Debug => "debug",
                LogLevel::Info => "info",
                LogLevel::Warn => "warn",
                LogLevel::Error => "error",
            };
            EnvFilter::new(format!("dooz_code={},dooz_worker={}", level, level))
        });

    if config.json_output {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_target(config.include_target)
                    .with_file(config.include_file)
                    .with_line_number(config.include_file)
            )
            .init();
    }
    
    tracing::info!("Logging initialized");
}

/// Initialize with defaults from environment
pub fn init_from_env() {
    let is_prod = std::env::var("DOOZ_ENV")
        .map(|v| v == "production")
        .unwrap_or(false);
    
    let config = if is_prod {
        LogConfig::production()
    } else {
        LogConfig::development()
    };
    
    init_logging(&config);
}

/// Structured log helpers
#[macro_export]
macro_rules! log_task {
    ($task_id:expr, $msg:expr) => {
        tracing::info!(task_id = %$task_id, $msg)
    };
    ($task_id:expr, $msg:expr, $($field:tt)*) => {
        tracing::info!(task_id = %$task_id, $($field)*, $msg)
    };
}

#[macro_export]
macro_rules! log_mode {
    ($mode:expr, $msg:expr) => {
        tracing::info!(mode = %$mode, $msg)
    };
}

#[macro_export]
macro_rules! log_perf {
    ($operation:expr, $duration_ms:expr) => {
        tracing::info!(
            operation = $operation,
            duration_ms = $duration_ms,
            "Performance metric"
        )
    };
}
