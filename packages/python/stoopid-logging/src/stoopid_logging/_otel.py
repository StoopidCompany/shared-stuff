"""OpenTelemetry trace/span ID injection.

Soft dependency on ``opentelemetry-api``. When the SDK is not installed,
``inject_otel_ids`` is a no-op.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from structlog.typing import EventDict, WrappedLogger

try:
    from opentelemetry import trace as _otel_trace
except ImportError:  # pragma: no cover - exercised only when OTel is absent
    _otel_trace = None  # type: ignore[assignment]


def inject_otel_ids(
    _logger: WrappedLogger,
    _name: str,
    event_dict: EventDict,
) -> EventDict:
    """Attach top-level ``trace_id`` and ``span_id`` from the active OTel span.

    Both fields are 32/16-char lowercase hex per W3C Trace Context. When no
    SDK is loaded or no span is active, neither field is added.
    """
    if _otel_trace is None:
        return event_dict
    span_ctx = _otel_trace.get_current_span().get_span_context()
    if not span_ctx.is_valid:
        return event_dict
    event_dict["trace_id"] = format(span_ctx.trace_id, "032x")
    event_dict["span_id"] = format(span_ctx.span_id, "016x")
    return event_dict
