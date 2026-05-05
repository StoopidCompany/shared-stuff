# Python logging quickstart

A minimal end-to-end demo of [stoopid-logging](../../packages/python/stoopid-logging).

## Run

```sh
uv run --project . main.py
```

## What it shows

- `configure_logging(level="info")` — pipeline setup
- `get_logger("orders.api")` — the `logger` field appears at the top level of
  every event from this logger.
- `log.warning(..., downstream=..., elapsed_ms=...)` — keyword arguments land
  under `context`.
- `log.bind(...)` — fields bound on the logger itself.
- `structlog.contextvars.bound_contextvars(...)` — fields bound via
  `contextvars` (request-scoped).

Each line of output is a single JSON object conforming to
[`schemas/log-event.schema.json`](../../schemas/log-event.schema.json).
