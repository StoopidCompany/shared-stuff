"""Unit tests for individual processors."""

from __future__ import annotations

import re

import pytest
from stoopid_logging._processors import (
    RESERVED_TOP_LEVEL,
    add_timestamp,
    aggregate_to_context,
    make_inject_service_version,
    normalize_level,
)


def test_normalize_level_maps_warning_to_warn() -> None:
    event = {"level": "warning"}
    result = normalize_level(None, "info", event)  # type: ignore[arg-type]
    assert result["level"] == "warn"


def test_normalize_level_maps_critical_to_fatal() -> None:
    event = {"level": "critical"}
    result = normalize_level(None, "info", event)  # type: ignore[arg-type]
    assert result["level"] == "fatal"


def test_normalize_level_passes_through_known_levels() -> None:
    for level in ("debug", "info", "warn", "error", "fatal"):
        event = {"level": level}
        result = normalize_level(None, "info", event)  # type: ignore[arg-type]
        assert result["level"] == level


def test_add_timestamp_format_iso_z_ms() -> None:
    event: dict[str, object] = {}
    result = add_timestamp(None, "info", event)  # type: ignore[arg-type]
    timestamp = result["timestamp"]
    assert isinstance(timestamp, str)
    iso_z_ms = r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$"
    assert re.match(iso_z_ms, timestamp), timestamp


def test_inject_service_version_reads_otel_service_name(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("OTEL_SERVICE_NAME", "billing")
    monkeypatch.setenv("SERVICE_VERSION", "2.5.0")
    inject = make_inject_service_version()
    result = inject(None, "info", {})  # type: ignore[arg-type]
    assert result["service"] == "billing"
    assert result["version"] == "2.5.0"


def test_inject_service_version_falls_back_to_unknown(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("OTEL_SERVICE_NAME", raising=False)
    monkeypatch.delenv("SERVICE_NAME", raising=False)
    monkeypatch.delenv("SERVICE_VERSION", raising=False)
    monkeypatch.delenv("OTEL_RESOURCE_ATTRIBUTES", raising=False)
    inject = make_inject_service_version()
    result = inject(None, "info", {})  # type: ignore[arg-type]
    assert result["service"] == "unknown_service"
    assert result["version"] == "0.0.0"


def test_inject_service_version_parses_otel_resource_attributes(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("SERVICE_VERSION", raising=False)
    monkeypatch.setenv(
        "OTEL_RESOURCE_ATTRIBUTES",
        "deployment.environment=prod,service.version=3.1.4,service.namespace=traice",
    )
    inject = make_inject_service_version()
    result = inject(None, "info", {})  # type: ignore[arg-type]
    assert result["version"] == "3.1.4"


def test_aggregate_to_context_moves_user_fields() -> None:
    event = {
        "level": "info",
        "message": "x",
        "service": "s",
        "version": "v",
        "timestamp": "2026-01-01T00:00:00.000Z",
        "user_id": "u_1",
        "request_method": "GET",
    }
    result = aggregate_to_context(None, "info", event)  # type: ignore[arg-type]
    assert result["context"] == {"user_id": "u_1", "request_method": "GET"}
    assert "user_id" not in result
    assert "request_method" not in result


def test_aggregate_to_context_preserves_reserved_top_level_keys() -> None:
    event = {
        "level": "info",
        "trace_id": "a" * 32,
        "span_id": "b" * 16,
        "logger": "foo.bar",
        "user_id": "u_1",
    }
    result = aggregate_to_context(None, "info", event)  # type: ignore[arg-type]
    assert result["trace_id"] == "a" * 32
    assert result["span_id"] == "b" * 16
    assert result["logger"] == "foo.bar"
    assert result["context"] == {"user_id": "u_1"}


def test_aggregate_to_context_drops_empty_context() -> None:
    event = {"level": "info", "message": "x", "service": "s", "version": "v"}
    result = aggregate_to_context(None, "info", event)  # type: ignore[arg-type]
    assert "context" not in result


def test_aggregate_to_context_merges_existing_context() -> None:
    event = {
        "level": "info",
        "context": {"a": 1},
        "b": 2,
    }
    result = aggregate_to_context(None, "info", event)  # type: ignore[arg-type]
    assert result["context"] == {"a": 1, "b": 2}


def test_aggregate_to_context_keeps_event_key_for_renamer() -> None:
    event = {"level": "info", "event": "the message"}
    result = aggregate_to_context(None, "info", event)  # type: ignore[arg-type]
    assert result["event"] == "the message"


def test_reserved_set_contains_all_schema_top_level_fields() -> None:
    expected = {
        "timestamp",
        "level",
        "message",
        "service",
        "version",
        "trace_id",
        "span_id",
        "logger",
        "error",
        "context",
    }
    assert expected == RESERVED_TOP_LEVEL
