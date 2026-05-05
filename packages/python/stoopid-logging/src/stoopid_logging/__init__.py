"""Structured JSON logging for Python services.

Conforms to the stoopid-commons shared log-event schema. See
``schemas/log-event.schema.json`` and ADRs 0004 and 0008 in the repository
root.
"""

from __future__ import annotations

from importlib.metadata import PackageNotFoundError, version

from stoopid_logging._config import configure_logging, get_logger

try:
    __version__ = version("stoopid-logging")
except PackageNotFoundError:
    __version__ = "0.0.0"

__all__ = ["__version__", "configure_logging", "get_logger"]
