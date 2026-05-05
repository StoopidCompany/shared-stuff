"""OTel trace/span ID injection tests.

Uses opentelemetry-sdk's in-memory tracer so we don't need a running
collector. The dependency is in the package's dev group.
"""

from __future__ import annotations

import json
from typing import Any

import pytest
from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from stoopid_logging import configure_logging, get_logger
from stoopid_logging._otel import inject_otel_ids


@pytest.fixture
def tracer() -> trace.Tracer:
    provider = TracerProvider()
    trace.set_tracer_provider(provider)
    return trace.get_tracer("stoopid-logging-tests")


def test_inject_otel_ids_no_active_span_omits_fields() -> None:
    result = inject_otel_ids(None, "info", {})  # type: ignore[arg-type]
    assert "trace_id" not in result
    assert "span_id" not in result


def test_inject_otel_ids_attaches_when_span_active(tracer: trace.Tracer) -> None:
    with tracer.start_as_current_span("test-span"):
        result = inject_otel_ids(None, "info", {})  # type: ignore[arg-type]
    assert isinstance(result.get("trace_id"), str)
    assert isinstance(result.get("span_id"), str)
    assert len(result["trace_id"]) == 32
    assert len(result["span_id"]) == 16
    assert all(c in "0123456789abcdef" for c in result["trace_id"])
    assert all(c in "0123456789abcdef" for c in result["span_id"])


def test_emit_with_active_span_includes_trace_ids(
    tracer: trace.Tracer,
    capsys: pytest.CaptureFixture[str],
) -> None:
    configure_logging(level="debug")
    log = get_logger("test")
    with tracer.start_as_current_span("outer"):
        log.info("inside-span")
    captured = capsys.readouterr().out.strip().splitlines()
    assert len(captured) == 1
    event: dict[str, Any] = json.loads(captured[0])
    assert "trace_id" in event
    assert "span_id" in event
    assert len(event["trace_id"]) == 32
    assert len(event["span_id"]) == 16
