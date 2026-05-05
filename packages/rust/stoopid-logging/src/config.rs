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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn clear_env() {
        for k in [
            "LOG_LEVEL",
            "OTEL_SERVICE_NAME",
            "SERVICE_NAME",
            "SERVICE_VERSION",
            "OTEL_RESOURCE_ATTRIBUTES",
        ] {
            env::remove_var(k);
        }
    }

    #[test]
    #[serial]
    fn resolve_level_explicit_takes_precedence_over_env() {
        clear_env();
        env::set_var("LOG_LEVEL", "warn");
        assert_eq!(resolve_level(Some("debug")).unwrap(), LevelFilter::DEBUG);
        clear_env();
    }

    #[test]
    #[serial]
    fn resolve_level_reads_log_level_env_when_no_explicit() {
        clear_env();
        env::set_var("LOG_LEVEL", "warn");
        assert_eq!(resolve_level(None).unwrap(), LevelFilter::WARN);
        clear_env();
    }

    #[test]
    #[serial]
    fn resolve_level_defaults_to_info() {
        clear_env();
        assert_eq!(resolve_level(None).unwrap(), LevelFilter::INFO);
    }

    #[test]
    #[serial]
    fn resolve_level_accepts_aliases_and_is_case_insensitive() {
        clear_env();
        assert_eq!(resolve_level(Some("WARNING")).unwrap(), LevelFilter::WARN);
        assert_eq!(resolve_level(Some("FATAL")).unwrap(), LevelFilter::ERROR);
        assert_eq!(resolve_level(Some("critical")).unwrap(), LevelFilter::ERROR);
        assert_eq!(
            resolve_level(Some("  debug  ")).unwrap(),
            LevelFilter::DEBUG
        );
    }

    #[test]
    #[serial]
    fn resolve_level_unknown_returns_error() {
        clear_env();
        let err = resolve_level(Some("loud")).unwrap_err();
        match err {
            InitError::UnknownLevel(s) => assert_eq!(s, "loud"),
            other => panic!("expected UnknownLevel, got {other:?}"),
        }
    }

    #[test]
    #[serial]
    fn resolve_service_name_prefers_otel_service_name() {
        clear_env();
        env::set_var("OTEL_SERVICE_NAME", "billing");
        env::set_var("SERVICE_NAME", "ignored");
        assert_eq!(resolve_service_name(), "billing");
        clear_env();
    }

    #[test]
    #[serial]
    fn resolve_service_name_falls_back_to_service_name() {
        clear_env();
        env::set_var("SERVICE_NAME", "checkout");
        assert_eq!(resolve_service_name(), "checkout");
        clear_env();
    }

    #[test]
    #[serial]
    fn resolve_service_name_default_when_unset() {
        clear_env();
        assert_eq!(resolve_service_name(), "unknown_service");
    }

    #[test]
    #[serial]
    fn resolve_service_version_prefers_explicit() {
        clear_env();
        env::set_var("SERVICE_VERSION", "1.2.3");
        env::set_var("OTEL_RESOURCE_ATTRIBUTES", "service.version=ignored");
        assert_eq!(resolve_service_version(), "1.2.3");
        clear_env();
    }

    #[test]
    #[serial]
    fn resolve_service_version_parses_otel_resource_attributes() {
        clear_env();
        env::set_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            "deployment.environment=prod,service.version=4.5.6,team=platform",
        );
        assert_eq!(resolve_service_version(), "4.5.6");
        clear_env();
    }

    #[test]
    #[serial]
    fn resolve_service_version_ignores_malformed_pairs() {
        clear_env();
        env::set_var(
            "OTEL_RESOURCE_ATTRIBUTES",
            "no_equals_sign,still_no=,service.version=7.8.9",
        );
        assert_eq!(resolve_service_version(), "7.8.9");
        clear_env();
    }

    #[test]
    #[serial]
    fn resolve_service_version_defaults_when_unset() {
        clear_env();
        assert_eq!(resolve_service_version(), "0.0.0");
    }

    #[test]
    fn init_error_display() {
        let already = InitError::AlreadyInitialized;
        assert!(already.to_string().contains("already installed"));
        let unknown = InitError::UnknownLevel("nope".to_string());
        assert!(unknown.to_string().contains("nope"));
    }
}
