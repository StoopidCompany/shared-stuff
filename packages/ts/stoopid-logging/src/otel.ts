/**
 * OpenTelemetry trace/span ID extraction.
 *
 * Soft dependency on `@opentelemetry/api`. When the package is not installed
 * (or no span is active), `getTraceContext` returns an empty object.
 */

type OtelApi = {
  trace: {
    getActiveSpan: () => { spanContext: () => SpanContext } | undefined;
  };
};

interface SpanContext {
  traceId: string;
  spanId: string;
  traceFlags: number;
}

let cachedApi: OtelApi | null | undefined;

function loadApi(): OtelApi | null {
  if (cachedApi !== undefined) return cachedApi;
  try {
    // Use a runtime require to keep @opentelemetry/api a true optional peer.
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const mod = require("@opentelemetry/api") as OtelApi;
    cachedApi = mod;
  } catch {
    cachedApi = null;
  }
  return cachedApi;
}

export function getTraceContext(): { trace_id?: string; span_id?: string } {
  const api = loadApi();
  if (!api) return {};
  const span = api.trace.getActiveSpan();
  if (!span) return {};
  const ctx = span.spanContext();
  if (!isValidTraceId(ctx.traceId) || !isValidSpanId(ctx.spanId)) return {};
  return { trace_id: ctx.traceId.toLowerCase(), span_id: ctx.spanId.toLowerCase() };
}

function isValidTraceId(id: string): boolean {
  return /^[0-9a-f]{32}$/i.test(id) && id !== "0".repeat(32);
}

function isValidSpanId(id: string): boolean {
  return /^[0-9a-f]{16}$/i.test(id) && id !== "0".repeat(16);
}

/** Reset the cached api lookup. Test-only. */
export function _resetOtelCache(): void {
  cachedApi = undefined;
}
