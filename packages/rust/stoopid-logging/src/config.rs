//! Environment-driven configuration for `stoopid_logging`.

use std::env;

use thiserror::Error;
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("a global tracing subscriber is already installed")]
    AlreadyInitialized,

    #[error("unknown LOG_LEVEL value: {0}")]
    UnknownLevel(String),
}

pub(crate) fn resolve_level(explicit: Option<&str>) -> Result<LevelFilter, InitError> {
    let raw = explicit
        .map(str::to_string)
        .or_else(|| env::var("LOG_LEVEL").ok())
        .unwrap_or_else(|| "info".to_string())
        .trim()
        .to_lowercase();

    let filter = match raw.as_str() {
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" | "warning" => LevelFilter::WARN,
        // `tracing` has no FATAL level; it's an alias for ERROR in our schema.
        "error" | "fatal" | "critical" => LevelFilter::ERROR,
        _ => return Err(InitError::UnknownLevel(raw)),
    };
    Ok(filter)
}

pub(crate) fn resolve_service_name() -> String {
    env::var("OTEL_SERVICE_NAME")
        .or_else(|_| env::var("SERVICE_NAME"))
        .unwrap_or_else(|_| "unknown_service".to_string())
}

pub(crate) fn resolve_service_version() -> String {
    if let Ok(explicit) = env::var("SERVICE_VERSION") {
        return explicit;
    }
    if let Ok(attrs) = env::var("OTEL_RESOURCE_ATTRIBUTES") {
        for pair in attrs.split(',') {
            if let Some((key, value)) = pair.split_once('=') {
                if key.trim() == "service.version" {
                    return value.trim().to_string();
                }
            }
        }
    }
    "0.0.0".to_string()
}
