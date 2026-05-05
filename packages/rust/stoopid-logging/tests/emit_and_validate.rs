//! End-to-end test: emit log events and validate them against the shared schema.
//!
//! This is the canonical test required by ADR-0004: a per-language
//! emit-and-validate suite that catches schema drift before publication.

use std::io::Write;
use std::sync::{Arc, Mutex};

use jsonschema::JSONSchema;
use serde_json::Value;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use stoopid_logging::_testing::JsonLayer;

#[derive(Clone, Default)]
struct VecWriter {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl VecWriter {
    fn lines(&self) -> Vec<Value> {
        let buf = self.buf.lock().expect("lock poisoned");
        let text = std::str::from_utf8(&buf).expect("utf-8");
        text.lines()
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_str(line).expect("parse json line"))
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
        let mut guard = self.buf.lock().expect("lock poisoned");
        guard.extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn assert_validates(schema: &JSONSchema, value: &Value) {
    if let Err(errors) = schema.validate(value) {
        for e in errors {
            eprintln!("validation error: {e}");
        }
        panic!("schema validation failed for: {value}");
    }
}

fn load_schema() -> JSONSchema {
    let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../schemas/log-event.schema.json");
    let bytes = std::fs::read(&schema_path)
        .unwrap_or_else(|_| panic!("schema at {}", schema_path.display()));
    let schema: Value = serde_json::from_slice(&bytes).expect("schema json");
    JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&schema)
        .expect("compile schema")
}

fn install(writer: VecWriter) -> tracing::subscriber::DefaultGuard {
    // Use a fresh service/version directly so the test doesn't depend on env.
    std::env::set_var("OTEL_SERVICE_NAME", "test-service");
    std::env::set_var("SERVICE_VERSION", "0.0.0-test");
    std::env::remove_var("LOG_LEVEL");
    let layer = JsonLayer::new(writer, "test-service".to_string(), "0.0.0-test".to_string());
    let subscriber = Registry::default().with(layer);
    tracing::subscriber::set_default(subscriber)
}

#[test]
fn schema_examples_self_validate() {
    let schema_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../schemas/log-event.schema.json");
    let schema_value: Value =
        serde_json::from_slice(&std::fs::read(&schema_path).expect("read schema"))
            .expect("parse schema");
    let validator = JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&schema_value)
        .expect("compile schema");
    let examples = schema_value
        .get("examples")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!examples.is_empty(), "schema has no examples");
    for ex in &examples {
        let result = validator.validate(ex);
        if let Err(errors) = result {
            for e in errors {
                eprintln!("validation error: {e}");
            }
            panic!("example failed schema validation: {ex}");
        }
    }
}

#[test]
fn minimal_event_validates() {
    let schema = load_schema();
    let writer = VecWriter::default();

    {
        let _guard = install(writer.clone());
        tracing::info!("service started");
    }

    let events = writer.lines();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    if let Err(errors) = schema.validate(event) {
        for e in errors {
            eprintln!("validation error: {e}");
        }
        panic!("event failed schema validation: {event}");
    }
    assert_eq!(event["level"], Value::String("info".to_string()));
    assert_eq!(
        event["message"],
        Value::String("service started".to_string())
    );
    assert_eq!(event["service"], Value::String("test-service".to_string()));
    assert!(!event.as_object().unwrap().contains_key("context"));
}

#[test]
fn event_fields_land_in_context() {
    let schema = load_schema();
    let writer = VecWriter::default();

    {
        let _guard = install(writer.clone());
        tracing::info!(user_id = "u_1", request_method = "GET", "processed");
    }

    let events = writer.lines();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_validates(&schema, event);
    let ctx = event
        .get("context")
        .expect("context object")
        .as_object()
        .unwrap();
    assert_eq!(ctx["user_id"], Value::String("u_1".to_string()));
    assert_eq!(ctx["request_method"], Value::String("GET".to_string()));
}

#[test]
fn span_fields_merge_into_context_with_innermost_wins() {
    let schema = load_schema();
    let writer = VecWriter::default();

    {
        let _guard = install(writer.clone());
        let outer = tracing::info_span!("outer", request_id = "req_42", layer = "outer");
        let _o = outer.enter();
        let inner = tracing::info_span!("inner", layer = "inner");
        let _i = inner.enter();
        tracing::info!(extra = 1, "in span");
    }

    let events = writer.lines();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_validates(&schema, event);
    let ctx = event.get("context").expect("context").as_object().unwrap();
    assert_eq!(ctx["request_id"], Value::String("req_42".to_string()));
    assert_eq!(ctx["layer"], Value::String("inner".to_string()));
    assert_eq!(ctx["extra"], Value::Number(1.into()));
}

#[test]
fn warn_level_emitted_as_warn() {
    let schema = load_schema();
    let writer = VecWriter::default();

    {
        let _guard = install(writer.clone());
        tracing::warn!("downstream slow");
    }

    let events = writer.lines();
    assert_eq!(events.len(), 1);
    assert_validates(&schema, &events[0]);
    assert_eq!(events[0]["level"], Value::String("warn".to_string()));
}

#[test]
fn timestamp_format_is_iso_z_ms() {
    let writer = VecWriter::default();
    {
        let _guard = install(writer.clone());
        tracing::info!("ts");
    }
    let events = writer.lines();
    let ts = events[0]["timestamp"].as_str().unwrap();
    // YYYY-MM-DDTHH:MM:SS.mmmZ
    let re_ok = ts.len() == 24
        && ts.ends_with('Z')
        && ts.chars().nth(10) == Some('T')
        && ts.chars().nth(19) == Some('.');
    assert!(re_ok, "unexpected timestamp shape: {ts}");
}
