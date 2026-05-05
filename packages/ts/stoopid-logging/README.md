# stoopid-logging (TypeScript)

Structured JSON logging for Node services. Emits log events conforming to the
[stoopid-commons shared log-event schema](../../../schemas/log-event.schema.json).

Built on [pino](https://getpino.io). Optionally correlates with
OpenTelemetry traces when `@opentelemetry/api` is installed.

## Install

```sh
pnpm add stoopid-logging
# with OpenTelemetry trace correlation:
pnpm add stoopid-logging @opentelemetry/api
```

## Usage

```ts
import { configureLogging, getLogger } from "stoopid-logging";

configureLogging(); // reads LOG_LEVEL env var (default "info")
const log = getLogger("orders.api");

log.info("service started");
log.warn({ downstream: "billing", elapsedMs: 842 }, "downstream slow");

const bound = log.child({ requestId: "req_42" });
bound.info("processed request");
```

Each call emits a single line of JSON to stdout, e.g.:

```json
{"timestamp":"2026-05-05T17:42:11.123Z","level":"info","message":"service started","service":"orders","version":"1.4.2","logger":"orders.api"}
```

## Configuration

| Variable | Purpose | Fallback |
| --- | --- | --- |
| `LOG_LEVEL` | Minimum severity (`debug`/`info`/`warn`/`error`/`fatal`) | `info` |
| `OTEL_SERVICE_NAME` | Service name | `SERVICE_NAME`, then `"unknown_service"` |
| `SERVICE_VERSION` | Service version | parsed from `OTEL_RESOURCE_ATTRIBUTES`, then `"0.0.0"` |

User-supplied object fields (passed as the first argument to log methods or
via `child()`) appear nested under the `context` field of the emitted event.
Top-level reserved field names are protected from collision because user
fields never reach the top level.

Pino's `trace` level is mapped to the schema's `debug` (the schema's level
enum permits only `debug`/`info`/`warn`/`error`/`fatal`).

## License

MIT
