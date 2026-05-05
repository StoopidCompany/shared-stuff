# stoopid-logging (Python)

Structured JSON logging for Python services. Emits log events conforming to
the [stoopid-commons shared log-event schema](../../../schemas/log-event.schema.json).

Built on [structlog](https://www.structlog.org/). Optionally correlates with
OpenTelemetry traces when `opentelemetry-api` is installed.

## Install

```sh
pip install stoopid-logging
# with OpenTelemetry trace correlation:
pip install "stoopid-logging[otel]"
```

## Usage

```python
from stoopid_logging import configure_logging, get_logger

configure_logging()  # reads LOG_LEVEL env var (default "info")
log = get_logger(__name__)

log.info("service started")
log.warning("downstream slow", downstream="billing", elapsed_ms=842)

bound = log.bind(request_id="req_42")
bound.info("processed request")
```

Each call emits a single line of JSON to stdout, e.g.:

```json
{
  "timestamp": "2026-05-05T17:42:11.123Z",
  "level": "info",
  "message": "service started",
  "service": "orders",
  "version": "1.4.2",
  "logger": "orders.api"
}
```

## Configuration

The package reads these environment variables:

| Variable            | Purpose                                                          | Fallback                                               |
| ------------------- | ---------------------------------------------------------------- | ------------------------------------------------------ |
| `LOG_LEVEL`         | Minimum severity to emit (`debug`/`info`/`warn`/`error`/`fatal`) | `info`                                                 |
| `OTEL_SERVICE_NAME` | Service name                                                     | `SERVICE_NAME`, then `"unknown_service"`               |
| `SERVICE_VERSION`   | Service version                                                  | parsed from `OTEL_RESOURCE_ATTRIBUTES`, then `"0.0.0"` |

User-supplied keyword arguments to log calls (and fields bound via
`log.bind(...)` or `structlog.contextvars.bound_contextvars(...)`) appear
nested under the `context` field of the emitted event. Top-level reserved
field names (`timestamp`, `level`, `message`, `service`, `version`,
`trace_id`, `span_id`, `logger`, `error`, `context`) are protected from
collision because user fields never reach the top level.

## License

MIT
