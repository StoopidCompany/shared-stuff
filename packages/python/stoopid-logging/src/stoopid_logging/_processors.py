"""structlog processor functions used by the stoopid-logging pipeline.

Each processor mutates and returns the structlog ``event_dict``. They are
deliberately small, side-effect free (except for env reads cached at import),
and unit-tested individually.
"""

from __future__ import annotations

import os
from datetime import UTC, datetime
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from structlog.typing import EventDict, WrappedLogger

# Field names that live at the top level of an emitted event. Anything else
# in the event dict is moved into the nested ``context`` object by
# ``aggregate_to_context``. Mirrored in the TS and Rust packages.
RESERVED_TOP_LEVEL: frozenset[str] = frozenset(
    {
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
    },
)

_LEVEL_ALIASES: dict[str, str] = {
    "warning": "warn",
    "critical": "fatal",
    "exception": "error",
}


def normalize_level(
    _logger: WrappedLogger,
    _name: str,
    event_dict: EventDict,
) -> EventDict:
    """Map structlog/stdlib level names to the schema's enum.

    ``warning`` -> ``warn``, ``critical`` -> ``fatal``, ``exception`` ->
    ``error``. Unknown levels pass through and will fail schema validation,
    which is the desired behavior.
    """
    level = event_dict.get("level")
    if isinstance(level, str):
        event_dict["level"] = _LEVEL_ALIASES.get(level, level)
    return event_dict


def add_timestamp(
    _logger: WrappedLogger,
    _name: str,
    event_dict: EventDict,
) -> EventDict:
    """Insert a UTC ISO 8601 timestamp with millisecond precision and ``Z`` suffix."""
    now = datetime.now(UTC)
    # isoformat with timespec="milliseconds" yields "...+00:00"; replace with "Z".
    event_dict["timestamp"] = now.isoformat(timespec="milliseconds").replace(
        "+00:00",
        "Z",
    )
    return event_dict


def _resolve_service_name() -> str:
    return (
        os.environ.get("OTEL_SERVICE_NAME")
        or os.environ.get("SERVICE_NAME")
        or "unknown_service"
    )


def _resolve_service_version() -> str:
    explicit = os.environ.get("SERVICE_VERSION")
    if explicit:
        return explicit
    attrs = os.environ.get("OTEL_RESOURCE_ATTRIBUTES", "")
    for pair in attrs.split(","):
        if "=" not in pair:
            continue
        key, _, value = pair.partition("=")
        if key.strip() == "service.version":
            return value.strip()
    return "0.0.0"


def make_inject_service_version() -> Any:
    """Return a processor that injects cached ``service`` and ``version``.

    Env reads happen once at processor construction; rebuild the pipeline if
    the environment changes mid-process (this is not a supported runtime
    case, but is what tests do via ``monkeypatch``).
    """
    service = _resolve_service_name()
    version_ = _resolve_service_version()

    def _inject(
        _logger: WrappedLogger,
        _name: str,
        event_dict: EventDict,
    ) -> EventDict:
        event_dict.setdefault("service", service)
        event_dict.setdefault("version", version_)
        return event_dict

    return _inject


def aggregate_to_context(
    _logger: WrappedLogger,
    _name: str,
    event_dict: EventDict,
) -> EventDict:
    """Move every non-reserved key into a nested ``context`` object.

    Runs near the end of the pipeline, after every reserved-field injector.
    If ``context`` is empty after aggregation, drop it from the event dict.
    structlog's ``event`` key is left in place and will be renamed to
    ``message`` by ``EventRenamer`` later.
    """
    existing_context = event_dict.pop("context", None)
    context: dict[str, Any] = {}
    if isinstance(existing_context, dict):
        context.update(existing_context)  # type: ignore[arg-type]

    for key in list(event_dict.keys()):
        # ``event`` is structlog's payload key; preserve it for EventRenamer.
        if key in RESERVED_TOP_LEVEL or key == "event":
            continue
        context[key] = event_dict.pop(key)

    if context:
        event_dict["context"] = context
    return event_dict
