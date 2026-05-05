import pino from "pino";

import {
  bindingsFormatter,
  levelFormatter,
  logFormatter,
  timestampFormatter,
} from "./formatters.js";
import { getTraceContext } from "./otel.js";
import type { ConfigureOptions } from "./types.js";

const VALID_LEVELS = new Set(["debug", "info", "warn", "warning", "error", "fatal", "critical"]);

function resolveLevel(level: string | undefined): string {
  const raw = (level ?? process.env["LOG_LEVEL"] ?? "info").trim().toLowerCase();
  if (!VALID_LEVELS.has(raw)) {
    throw new Error(
      `Unknown log level ${JSON.stringify(raw)}; must be one of debug, info, warn, error, fatal`,
    );
  }
  // Pino accepts "warn"/"fatal"; map "warning"/"critical" aliases.
  if (raw === "warning") return "warn";
  if (raw === "critical") return "fatal";
  return raw;
}

function resolveServiceName(): string {
  return process.env["OTEL_SERVICE_NAME"] ?? process.env["SERVICE_NAME"] ?? "unknown_service";
}

function resolveServiceVersion(): string {
  const explicit = process.env["SERVICE_VERSION"];
  if (explicit) return explicit;
  const attrs = process.env["OTEL_RESOURCE_ATTRIBUTES"] ?? "";
  for (const pair of attrs.split(",")) {
    const idx = pair.indexOf("=");
    if (idx < 0) continue;
    const key = pair.slice(0, idx).trim();
    const value = pair.slice(idx + 1).trim();
    if (key === "service.version") return value;
  }
  return "0.0.0";
}

let rootLogger: pino.Logger | null = null;

export function configureLogging(opts: ConfigureOptions = {}): pino.Logger {
  const level = resolveLevel(opts.level);
  const service = resolveServiceName();
  const version = resolveServiceVersion();

  const logger = pino(
    {
      level,
      messageKey: "message",
      timestamp: timestampFormatter,
      formatters: {
        level: levelFormatter,
        bindings: bindingsFormatter,
        log: logFormatter,
      },
      mixin: () => ({ service, version, ...getTraceContext() }),
      base: undefined,
    },
    opts.destination ?? pino.destination({ dest: 1, sync: true }),
  );

  rootLogger = wrapChildToPreserveFormatter(logger);
  return rootLogger;
}

/**
 * Force every `child()` call on this logger (and its descendants) to pass an
 * options object. Pino swaps `formatters.bindings` for an identity function
 * on `child(bindings)` calls without options (perf optimization in
 * pino@9.14), which would bypass our context aggregation. Wrapping ensures
 * the formatter runs for every child binding.
 */
/**
 * Reference to pino's prototype `child` method, captured once. We invoke it
 * directly with `.call(logger, ...)` rather than going through the override
 * chain, because each child inherits its parent's overridden `.child` via
 * the prototype chain, which would create infinite indirection if we used
 * `logger.child.bind(logger)`.
 */
const PROTO_CHILD: (
  this: pino.Logger,
  bindings: pino.Bindings,
  options?: pino.ChildLoggerOptions,
) => pino.Logger = (() => {
  const sample = pino({ enabled: false });
  const proto = Object.getPrototypeOf(sample) as { child: typeof PROTO_CHILD };
  return proto.child;
})();

/**
 * Pino swaps `formatters.bindings` for an identity function on every
 * `child()` call unless the new formatter is passed in `options.formatters`.
 * (The `child()` slow path always reaches the `buildFormatters(...,
 * resetChildingsFormatter, ...)` branch when options does not contain
 * `formatters`.) This wrapper forces our formatter on every descendant.
 */
function wrapChildToPreserveFormatter(logger: pino.Logger): pino.Logger {
  const wrappedChild = (b: pino.Bindings, opts?: pino.ChildLoggerOptions) => {
    const merged = {
      ...(opts ?? {}),
      formatters: {
        bindings: bindingsFormatter,
        ...(opts?.formatters ?? {}),
      },
    } as pino.ChildLoggerOptions;
    const child = PROTO_CHILD.call(logger, b, merged);
    return wrapChildToPreserveFormatter(child);
  };
  Object.defineProperty(logger, "child", {
    value: wrappedChild,
    writable: true,
    configurable: true,
  });
  return logger;
}

export function getLogger(name?: string): pino.Logger {
  if (!rootLogger) {
    configureLogging();
  }
  const root = rootLogger as pino.Logger;
  return name ? root.child({ logger: name }, {}) : root;
}

/** Reset module state. Test-only. */
export function _resetForTests(): void {
  rootLogger = null;
}
