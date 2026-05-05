# TypeScript logging quickstart

A minimal end-to-end demo of [stoopid-logging](../../packages/ts/stoopid-logging).

## Run

```sh
pnpm install
pnpm --filter ts-logging-quickstart start
```

## What it shows

- `configureLogging({ level: "info" })` — pipeline setup
- `getLogger("orders.api")` — the `logger` field appears at the top level of
  every event from this logger.
- `log.warn({ downstream, elapsedMs }, "msg")` — first-arg object fields land
  under `context`.
- `log.child({ requestId })` — pino child loggers bind fields once; they
  flow through `context` on every event from the child.

Each line of output is a single JSON object conforming to
[`schemas/log-event.schema.json`](../../schemas/log-event.schema.json).
