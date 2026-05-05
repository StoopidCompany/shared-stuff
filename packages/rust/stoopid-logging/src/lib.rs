//! Structured JSON logging for Rust services.
//!
//! Conforms to the stoopid-commons shared log-event schema. See
//! `schemas/log-event.schema.json` and ADRs 0004 and 0008 in the repository
//! root.
//!
//! ```no_run
//! stoopid_logging::init().expect("install subscriber");
//! tracing::info!(user_id = "u_1", "hello world");
//! ```

mod config;
mod layer;
#[cfg(feature = "otel")]
mod otel;

use std::io;

use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

pub use config::InitError;

/// Builder for the global subscriber. Use [`builder`] to construct one.
pub struct Builder<W = fn() -> io::Stdout> {
    level: Option<String>,
    make_writer: W,
}

impl Default for Builder<fn() -> io::Stdout> {
    fn default() -> Self {
        Self {
            level: None,
            make_writer: io::stdout,
        }
    }
}

impl<W> Builder<W> {
    /// Override the level. Accepts schema enum values and stdlib aliases.
    /// When `None`, falls back to `LOG_LEVEL` env, then `info`.
    #[must_use]
    pub fn level(mut self, level: impl Into<String>) -> Self {
        self.level = Some(level.into());
        self
    }

    /// Override the writer. Tests pass a custom writer to capture output.
    #[must_use]
    pub fn with_writer<W2>(self, make_writer: W2) -> Builder<W2>
    where
        W2: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
    {
        Builder {
            level: self.level,
            make_writer,
        }
    }
}

impl<W> Builder<W>
where
    W: for<'writer> MakeWriter<'writer> + Send + Sync + 'static,
{
    /// Install the configured subscriber as the global default. Returns an
    /// error if a global subscriber is already installed.
    pub fn try_init(self) -> Result<(), InitError> {
        let level = config::resolve_level(self.level.as_deref())?;
        let service = config::resolve_service_name();
        let version = config::resolve_service_version();
        let json_layer = layer::JsonLayer::new(self.make_writer, service, version);

        Registry::default()
            .with(level)
            .with(json_layer)
            .try_init()
            .map_err(|_| InitError::AlreadyInitialized)
    }
}

/// Build a fresh subscriber configuration.
#[must_use]
pub fn builder() -> Builder {
    Builder::default()
}

/// Install the default subscriber: JSON to stdout, level from `LOG_LEVEL`
/// env (default `info`), service and version resolved from environment
/// variables. Returns an error if a global subscriber is already installed.
pub fn init() -> Result<(), InitError> {
    builder().try_init()
}

/// Like [`init`] but never returns `AlreadyInitialized` — silently no-ops if
/// another subscriber is already installed. Useful in test binaries that
/// share a process.
pub fn try_init() -> Result<(), InitError> {
    match init() {
        Ok(()) | Err(InitError::AlreadyInitialized) => Ok(()),
        Err(e) => Err(e),
    }
}

#[doc(hidden)]
pub mod _testing {
    pub use crate::layer::JsonLayer;
}
