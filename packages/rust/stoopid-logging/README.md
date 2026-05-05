# stoopid-logging (Rust)

Structured JSON logging for Rust services. Emits log events conforming to the
[stoopid-commons shared log-event schema](../../../schemas/log-event.schema.json).

Built on [`tracing`](https://docs.rs/tracing) and
[`tracing-subscriber`](https://docs.rs/tracing-subscriber). Optional
OpenTelemetry trace correlation via the `otel` feature.

## Install

```toml
[dependencies]
stoopid-logging = "0.1"
tracing = "0.1"
# with OpenTelemetry trace correlation:
stoopid-logging = { version = "0.1", features = ["otel"] }
```

## Usage

```rust
fn main() {
    stoopid_logging::init().expect("install subscriber");

    tracing::info!("service started");
    tracing::warn!(downstream = "billing", elapsed_ms = 842, "downstream slow");

    let span = tracing::info_span!("request", request_id = "req_42");
    let _enter = span.enter();
    tracing::info!("inside span");
}
```

Each call emits a single line of JSON to stdout, e.g.:

```json
{
  "timestamp": "2026-05-05T17:42:11.123Z",
  "level": "info",
  "message": "service started",
  "service": "orders",
  "version": "1.4.2"
}
```

## Configuration

| Variable            | Purpose                                                  | Fallback                                               |
| ------------------- | -------------------------------------------------------- | ------------------------------------------------------ |
| `LOG_LEVEL`         | Minimum severity (`debug`/`info`/`warn`/`error`/`fatal`) | `info`                                                 |
| `OTEL_SERVICE_NAME` | Service name                                             | `SERVICE_NAME`, then `"unknown_service"`               |
| `SERVICE_VERSION`   | Service version                                          | parsed from `OTEL_RESOURCE_ATTRIBUTES`, then `"0.0.0"` |

User-supplied event fields (e.g. `tracing::info!(user_id = "u_1", "msg")`)
and span fields land inside the nested `context` object on the emitted
event. Innermost span fields win on key collisions across nested spans.

`tracing::Level::TRACE` is mapped to schema `debug` since the schema's
level enum does not define a `trace` value.

## License

MIT
