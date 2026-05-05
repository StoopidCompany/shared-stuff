# Rust logging quickstart

A minimal end-to-end demo of [stoopid-logging](../../packages/rust/stoopid-logging).

## Run

```sh
cargo run -p rust-logging-quickstart
```

## What it shows

- `stoopid_logging::init()` — install the global subscriber.
- `tracing::info!`/`warn!` macros — the standard `tracing` macros work
  unchanged; output flows through the JSON layer.
- Field syntax (`user_id = "u_1"`) — event fields land inside `context`.
- `tracing::info_span!(...)` — span fields propagate into `context` for
  every event emitted within the span.

Each line of output is a single JSON object conforming to
[`schemas/log-event.schema.json`](../../schemas/log-event.schema.json).
