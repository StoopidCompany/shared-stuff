//! Integration coverage for the public `Builder` / `init` / `try_init` surface.
//!
//! `try_init()` installs the global subscriber, which can succeed exactly once
//! per test binary. Cargo runs each integration test file as a separate
//! binary, so this file is dedicated to exercising the install flow.

use std::io::Write;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use serial_test::serial;
use stoopid_logging::InitError;
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
struct VecWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl VecWriter {
    fn lines(&self) -> Vec<Value> {
        let buf = self.buf.lock().expect("lock");
        std::str::from_utf8(&buf)
            .expect("utf8")
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| serde_json::from_str(l).expect("json line"))
            .collect()
    }
}

impl<'a> MakeWriter<'a> for VecWriter {
    type Writer = VecWriterGuard;
    fn make_writer(&'a self) -> Self::Writer {
        VecWriterGuard {
            buf: Arc::clone(&self.buf),
        }
    }
}

struct VecWriterGuard {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl Write for VecWriterGuard {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buf.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Drives the full happy path: builder().level().with_writer().try_init() →
/// log emission honors level filtering → second try_init returns
/// AlreadyInitialized → top-level try_init() swallows that error.
///
/// All assertions live in one function because there is exactly one global
/// subscriber slot per test binary.
#[test]
#[serial]
fn builder_init_lifecycle() {
    std::env::set_var("OTEL_SERVICE_NAME", "builder-test");
    std::env::set_var("SERVICE_VERSION", "9.9.9");
    std::env::remove_var("LOG_LEVEL");

    let writer = VecWriter::default();

    stoopid_logging::builder()
        .level("warn")
        .with_writer(writer.clone())
        .try_init()
        .expect("first try_init must succeed");

    tracing::info!("filtered out by level");
    tracing::warn!(reason = "downstream slow", "service degraded");
    tracing::error!("hard fail");

    let events = writer.lines();
    assert_eq!(
        events.len(),
        2,
        "info should be filtered out, got {events:?}"
    );
    assert_eq!(events[0]["level"], Value::String("warn".into()));
    assert_eq!(events[0]["service"], Value::String("builder-test".into()));
    assert_eq!(events[0]["version"], Value::String("9.9.9".into()));
    assert_eq!(
        events[0]["message"],
        Value::String("service degraded".into())
    );
    assert_eq!(
        events[0]["context"]["reason"],
        Value::String("downstream slow".into())
    );
    assert_eq!(events[1]["level"], Value::String("error".into()));

    // Second try_init must report AlreadyInitialized.
    let again = stoopid_logging::builder()
        .with_writer(VecWriter::default())
        .try_init();
    assert!(matches!(again, Err(InitError::AlreadyInitialized)));

    // Top-level try_init() swallows AlreadyInitialized.
    stoopid_logging::try_init().expect("top-level try_init should swallow AlreadyInitialized");

    // init() (no-arg, stdout) also surfaces AlreadyInitialized.
    let stdout_again = stoopid_logging::init();
    assert!(matches!(stdout_again, Err(InitError::AlreadyInitialized)));
}

/// `Builder::try_init` rejects an unknown LOG_LEVEL value before installing
/// any subscriber. Lives in its own test binary so the global slot stays free.
#[test]
fn unknown_level_returns_error_without_installing() {
    let result = stoopid_logging::builder()
        .level("loud")
        .with_writer(VecWriter::default())
        .try_init();
    match result {
        Err(InitError::UnknownLevel(s)) => assert_eq!(s, "loud"),
        other => panic!("expected UnknownLevel, got {other:?}"),
    }
}
