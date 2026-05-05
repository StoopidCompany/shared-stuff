"""End-to-end test: emit log events and validate them against the shared schema.

This is the canonical test required by ADR-0004: a per-language emit-and-
validate suite that catches schema drift before publication.
"""

from __future__ import annotations

import json
from typing import Any

import pytest
from jsonschema import Draft202012Validator
from stoopid_logging import configure_logging, get_logger


def _emit_lines(capsys: pytest.CaptureFixture[str]) -> list[dict[str, Any]]:
    captured = capsys.readouterr().out.strip().splitlines()
    return [json.loads(line) for line in captured]


def test_schema_is_self_valid(log_event_schema: dict[str, Any]) -> None:
    """The schema must validate against the JSON Schema 2020-12 meta-schema."""
    Draft202012Validator.check_schema(log_event_schema)


def test_schema_examples_validate(log_event_schema: dict[str, Any]) -> None:
    """The canonical examples in the schema must validate against the schema."""
    validator = Draft202012Validator(log_event_schema)
    for example in log_event_schema.get("examples", []):
        validator.validate(example)


def test_minimal_emit_validates(
    log_event_schema: dict[str, Any],
    capsys: pytest.CaptureFixture[str],
) -> None:
    configure_logging(level="info")
    log = get_logger()
    log.info("service started")
    events = _emit_lines(capsys)
    assert len(events) == 1
    Draft202012Validator(log_event_schema).validate(events[0])
    assert events[0]["level"] == "info"
    assert events[0]["message"] == "service started"
    assert events[0]["service"] == "test-service"
    assert events[0]["version"] == "0.0.0-test"
    assert "context" not in events[0]


def test_bound_fields_land_in_context(
    log_event_schema: dict[str, Any],
    capsys: pytest.CaptureFixture[str],
) -> None:
    configure_logging(level="info")
    log = get_logger("orders.api").bind(request_id="req_42")
    log.info("processed", duration_ms=18)
    events = _emit_lines(capsys)
    assert len(events) == 1
    Draft202012Validator(log_event_schema).validate(events[0])
    assert events[0]["logger"] == "orders.api"
    assert events[0]["context"] == {"request_id": "req_42", "duration_ms": 18}


def test_level_filter_drops_below_threshold(
    capsys: pytest.CaptureFixture[str],
) -> None:
    configure_logging(level="warn")
    log = get_logger()
    log.info("not emitted")
    log.warning("emitted")
    log.error("also emitted")
    events = _emit_lines(capsys)
    assert len(events) == 2
    assert events[0]["level"] == "warn"
    assert events[1]["level"] == "error"


def test_warning_method_emits_warn_level(
    log_event_schema: dict[str, Any],
    capsys: pytest.CaptureFixture[str],
) -> None:
    configure_logging(level="info")
    log = get_logger()
    log.warning("downstream slow")
    events = _emit_lines(capsys)
    Draft202012Validator(log_event_schema).validate(events[0])
    assert events[0]["level"] == "warn"


def test_critical_method_emits_fatal_level(
    log_event_schema: dict[str, Any],
    capsys: pytest.CaptureFixture[str],
) -> None:
    configure_logging(level="info")
    log = get_logger()
    log.critical("oom")
    events = _emit_lines(capsys)
    Draft202012Validator(log_event_schema).validate(events[0])
    assert events[0]["level"] == "fatal"


def test_explicit_user_context_dict_merges(
    log_event_schema: dict[str, Any],
    capsys: pytest.CaptureFixture[str],
) -> None:
    configure_logging(level="info")
    log = get_logger().bind(context={"a": 1})
    log.info("merge", b=2)
    events = _emit_lines(capsys)
    Draft202012Validator(log_event_schema).validate(events[0])
    assert events[0]["context"] == {"a": 1, "b": 2}


def test_unknown_level_rejected_at_configure() -> None:
    with pytest.raises(ValueError, match="Unknown log level"):
        configure_logging(level="trace")


def test_log_level_env_var_drives_default(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    monkeypatch.setenv("LOG_LEVEL", "error")
    configure_logging()
    log = get_logger()
    log.warning("dropped")
    log.error("kept")
    events = _emit_lines(capsys)
    assert len(events) == 1
    assert events[0]["level"] == "error"
