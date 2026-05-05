//! Custom `tracing_subscriber::Layer` that emits JSON conforming to the
//! shared log-event schema.
//!
//! `tracing_subscriber`'s built-in JSON formatter emits a flat shape that
//! does not match the schema (uppercase level, `target`, flattened span
//! fields), so we build the event object ourselves.

use std::fmt;
use std::io::Write;
use std::sync::Arc;

use serde_json::{Map, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

const RESERVED_AT_TOP_LEVEL_BY_VISITOR: &[&str] = &["message"];

/// JSON-emitting `tracing_subscriber::Layer`. Public for testing-only use via
/// the `_testing` module. End users should configure logging via
/// [`crate::init`] / [`crate::builder`] rather than constructing this directly.
pub struct JsonLayer<W> {
    make_writer: W,
    service: Arc<str>,
    version: Arc<str>,
}

impl<W> JsonLayer<W> {
    /// Construct a layer with the given writer factory, service, and version.
    pub fn new(make_writer: W, service: String, version: String) -> Self {
        Self {
            make_writer,
            service: Arc::from(service),
            version: Arc::from(version),
        }
    }
}

#[derive(Default)]
struct JsonFields(Map<String, Value>);

#[derive(Default)]
struct JsonVisitor {
    fields: Map<String, Value>,
}

impl Visit for JsonVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .insert(field.name().to_string(), Value::String(value.to_string()));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .insert(field.name().to_string(), Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .insert(field.name().to_string(), Value::Number(value.into()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .insert(field.name().to_string(), Value::Bool(value));
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        let v = serde_json::Number::from_f64(value).map_or(Value::Null, Value::Number);
        self.fields.insert(field.name().to_string(), v);
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.fields.insert(
            field.name().to_string(),
            Value::String(format!("{value:?}")),
        );
    }
}

fn level_to_str(level: Level) -> &'static str {
    match level {
        Level::TRACE | Level::DEBUG => "debug",
        Level::INFO => "info",
        Level::WARN => "warn",
        Level::ERROR => "error",
    }
}

impl<S, W> Layer<S> for JsonLayer<W>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    W: for<'writer> MakeWriter<'writer> + 'static,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else { return };
        let mut visitor = JsonVisitor::default();
        attrs.record(&mut visitor);
        span.extensions_mut().insert(JsonFields(visitor.fields));
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(id) else { return };
        let mut extensions = span.extensions_mut();
        let fields = extensions.get_mut::<JsonFields>();
        let Some(JsonFields(map)) = fields else {
            return;
        };
        let mut visitor = JsonVisitor {
            fields: std::mem::take(map),
        };
        values.record(&mut visitor);
        *map = visitor.fields;
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let mut out = Map::new();

        let timestamp = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
        // Force millisecond precision: trim subseconds beyond ms and ensure Z.
        out.insert(
            "timestamp".to_string(),
            Value::String(normalize_ts(&timestamp)),
        );
        out.insert(
            "level".to_string(),
            Value::String(level_to_str(*event.metadata().level()).to_string()),
        );
        out.insert(
            "service".to_string(),
            Value::String(self.service.to_string()),
        );
        out.insert(
            "version".to_string(),
            Value::String(self.version.to_string()),
        );

        let mut event_visitor = JsonVisitor::default();
        event.record(&mut event_visitor);

        // The `message` field is stored at top level; everything else from the
        // event becomes part of `context`.
        if let Some(msg) = event_visitor.fields.remove("message") {
            out.insert("message".to_string(), msg);
        } else {
            out.insert("message".to_string(), Value::String(String::new()));
        }

        let mut context = Map::new();

        // Walk span tree root → leaf so leaf fields override on collisions.
        if let Some(scope) = ctx.event_scope(event) {
            for span in scope.from_root() {
                let extensions = span.extensions();
                if let Some(JsonFields(fields)) = extensions.get::<JsonFields>() {
                    for (k, v) in fields {
                        if RESERVED_AT_TOP_LEVEL_BY_VISITOR.contains(&k.as_str()) {
                            continue;
                        }
                        context.insert(k.clone(), v.clone());
                    }
                }
            }
        }

        // Event fields (innermost) override span fields on collisions.
        for (k, v) in event_visitor.fields {
            context.insert(k, v);
        }

        #[cfg(feature = "otel")]
        if let Some((trace_id, span_id)) = crate::otel::extract_trace_ids(&ctx, event) {
            out.insert("trace_id".to_string(), Value::String(trace_id));
            out.insert("span_id".to_string(), Value::String(span_id));
        }

        if !context.is_empty() {
            out.insert("context".to_string(), Value::Object(context));
        }

        let Ok(mut line) = serde_json::to_string(&Value::Object(out)) else {
            return;
        };
        line.push('\n');

        let mut writer = self.make_writer.make_writer();
        let _ = writer.write_all(line.as_bytes());
    }
}

fn normalize_ts(rfc3339: &str) -> String {
    // time's Rfc3339 emits e.g. "2026-05-05T17:42:11.123456789Z" or "...+00:00".
    // Schema requires ms precision and Z suffix. Truncate or pad to 3 sub-digits.
    let (date_time_part, _tz) = match rfc3339.find(['Z', '+']) {
        Some(idx) => (&rfc3339[..idx], &rfc3339[idx..]),
        None => return rfc3339.to_string(),
    };

    let trimmed = if let Some(dot_idx) = date_time_part.find('.') {
        let (head, frac) = date_time_part.split_at(dot_idx);
        let frac = &frac[1..]; // drop the dot
        let mut frac_buf = String::with_capacity(3);
        for c in frac.chars().take(3) {
            frac_buf.push(c);
        }
        while frac_buf.len() < 3 {
            frac_buf.push('0');
        }
        format!("{head}.{frac_buf}")
    } else {
        format!("{date_time_part}.000")
    };

    format!("{trimmed}Z")
}
