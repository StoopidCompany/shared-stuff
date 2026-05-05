//! OpenTelemetry trace/span ID extraction (feature-gated).

use opentelemetry::trace::{SpanContext, TraceContextExt};
use tracing::{Event, Subscriber};
use tracing_opentelemetry::OtelData;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

pub(crate) fn extract_trace_ids<S>(
    ctx: &Context<'_, S>,
    event: &Event<'_>,
) -> Option<(String, String)>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let scope = ctx.event_scope(event)?;
    for span in scope.from_root() {
        let extensions = span.extensions();
        if let Some(otel_data) = extensions.get::<OtelData>() {
            let parent_span = otel_data.parent_cx.span();
            let span_ctx: &SpanContext = parent_span.span_context();
            if span_ctx.is_valid() {
                let trace_id = format!("{:032x}", span_ctx.trace_id());
                let span_id = format!("{:016x}", span_ctx.span_id());
                return Some((trace_id, span_id));
            }
        }
    }
    None
}
