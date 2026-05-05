"""Public configuration entry points: ``configure_logging`` and ``get_logger``."""

from __future__ import annotations

import logging
import os
from typing import TYPE_CHECKING

import structlog

from stoopid_logging._otel import inject_otel_ids
from stoopid_logging._processors import (
    add_timestamp,
    aggregate_to_context,
    make_inject_service_version,
    normalize_level,
)

if TYPE_CHECKING:
    from structlog.stdlib import BoundLogger

# Map our schema's level names (and a couple of common aliases) to the stdlib
# integer levels that structlog's filtering wrapper understands.
_LEVEL_TO_INT: dict[str, int] = {
    "debug": logging.DEBUG,
    "info": logging.INFO,
    "warn": logging.WARNING,
    "warning": logging.WARNING,
    "error": logging.ERROR,
    "fatal": logging.CRITICAL,
    "critical": logging.CRITICAL,
}

_configured: bool = False


def _resolve_level(level: str | None) -> int:
    name = (level or os.environ.get("LOG_LEVEL") or "info").strip().lower()
    if name not in _LEVEL_TO_INT:
        msg = (
            f"Unknown log level {name!r}; must be one of "
            f"{sorted(set(_LEVEL_TO_INT) - {'warning', 'critical'})}"
        )
        raise ValueError(msg)
    return _LEVEL_TO_INT[name]


def configure_logging(level: str | None = None) -> None:
    """Configure structlog to emit JSON conforming to the shared schema.

    Idempotent. The first call installs the pipeline; subsequent calls with
    the same ``level`` are no-ops. Calling with a different ``level`` will
    re-install the pipeline.

    ``level`` accepts schema enum values (``debug``/``info``/``warn``/
    ``error``/``fatal``) and the stdlib aliases (``warning``/``critical``).
    When ``None``, reads ``LOG_LEVEL`` from the environment, defaulting to
    ``info``.
    """
    global _configured
    level_int = _resolve_level(level)

    structlog.configure(
        processors=[
            structlog.contextvars.merge_contextvars,
            structlog.processors.add_log_level,
            normalize_level,
            add_timestamp,
            make_inject_service_version(),
            inject_otel_ids,
            structlog.processors.format_exc_info,
            aggregate_to_context,
            structlog.processors.EventRenamer(to="message"),
            structlog.processors.JSONRenderer(sort_keys=False),
        ],
        wrapper_class=structlog.make_filtering_bound_logger(level_int),
        logger_factory=structlog.PrintLoggerFactory(),
        cache_logger_on_first_use=True,
    )
    _configured = True


def get_logger(name: str | None = None) -> BoundLogger:
    """Return a logger ready for use.

    Lazily configures logging on the first call so that consumers who never
    invoke :func:`configure_logging` directly still get correct output.
    Passing ``name`` binds it as the top-level ``logger`` field on every
    event from this logger.
    """
    if not _configured:
        configure_logging()
    logger = structlog.get_logger(name)
    if name is not None:
        logger = logger.bind(logger=name)
    return logger
