"""Shared pytest fixtures for stoopid-logging tests."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

import pytest
import stoopid_logging._config as _config
import structlog

SCHEMA_PATH = Path(__file__).resolve().parents[4] / "schemas" / "log-event.schema.json"


@pytest.fixture(scope="session")
def log_event_schema() -> dict[str, Any]:
    """Load the shared log-event JSON schema once per test session."""
    return json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))


@pytest.fixture(autouse=True)
def _reset_structlog_state(monkeypatch: pytest.MonkeyPatch) -> None:
    """Reset the structlog defaults and the package's configure latch.

    structlog caches loggers on first use, which leaks across tests. This
    fixture wipes that state so each test starts from a clean configuration.
    """
    structlog.reset_defaults()
    monkeypatch.setattr(_config, "_configured", False)
    # Default service env to a known value for tests that don't override it.
    monkeypatch.setenv("OTEL_SERVICE_NAME", "test-service")
    monkeypatch.setenv("SERVICE_VERSION", "0.0.0-test")
    monkeypatch.delenv("LOG_LEVEL", raising=False)
