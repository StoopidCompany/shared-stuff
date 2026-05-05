import { RESERVED_TOP_LEVEL } from "./types.js";

/**
 * Pino `timestamp` function: returns a raw JSON fragment that pino concatenates
 * into the output line. Must include the leading comma and the quoted key.
 * Yields ISO 8601 UTC with millisecond precision and a `Z` suffix.
 */
export function timestampFormatter(): string {
  return `,"timestamp":"${new Date().toISOString()}"`;
}

/**
 * Pino `formatters.level`: lowercase string per the schema enum. Pino's
 * `trace` level is mapped to `debug` since the schema does not define it.
 */
export function levelFormatter(label: string): Record<string, string> {
  const mapped = label === "trace" ? "debug" : label;
  return { level: mapped };
}

/**
 * Pino `formatters.bindings`: strip pino's default `pid` and `hostname`. Any
 * other bindings (e.g. the `logger` field set by `getLogger(name)`) are
 * passed through unchanged.
 */
export function bindingsFormatter(
  bindings: Record<string, unknown> | undefined,
): Record<string, unknown> {
  if (!bindings) return {};
  const out: Record<string, unknown> = {};
  const context: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(bindings)) {
    if (key === "pid" || key === "hostname") continue;
    if (RESERVED_TOP_LEVEL.has(key)) {
      out[key] = value;
    } else {
      context[key] = value;
    }
  }
  if (Object.keys(context).length > 0) {
    out["context"] = context;
  }
  return out;
}

/**
 * Pino `formatters.log`: runs once per emitted event with the fully merged
 * object (bindings + per-call fields + mixin). Aggregates every non-reserved
 * key into a nested `context` object. If the user explicitly passed a
 * `context` field, its contents are merged in. The `context` key is dropped
 * from the output when empty.
 */
export function logFormatter(obj: Record<string, unknown>): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  const context: Record<string, unknown> = {};

  const explicit = obj["context"];
  if (explicit && typeof explicit === "object" && !Array.isArray(explicit)) {
    Object.assign(context, explicit as Record<string, unknown>);
  }

  for (const [key, value] of Object.entries(obj)) {
    if (key === "context") continue;
    if (RESERVED_TOP_LEVEL.has(key)) {
      out[key] = value;
    } else {
      context[key] = value;
    }
  }

  if (Object.keys(context).length > 0) {
    out["context"] = context;
  }
  return out;
}
