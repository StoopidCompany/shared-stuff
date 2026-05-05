"""Quickstart example for stoopid-logging (Python).

Run from the example's directory:

    uv run main.py
"""

from __future__ import annotations

import structlog
from stoopid_logging import configure_logging, get_logger


def main() -> None:
    configure_logging(level="info")

    log = get_logger("orders.api")
    log.info("service started")
    log.warning(
        "downstream slow",
        downstream="billing",
        elapsed_ms=842,
    )

    bound = log.bind(request_id="req_42")
    bound.info("processed request")

    with structlog.contextvars.bound_contextvars(tenant="acme"):
        log.info("tenant-scoped event")


if __name__ == "__main__":
    main()
