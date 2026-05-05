/**
 * Field names that live at the top level of an emitted event. Anything else in
 * the event object is moved into the nested `context` object by the log
 * formatter. Mirrored in the Python and Rust packages.
 */
export const RESERVED_TOP_LEVEL = new Set<string>([
  "timestamp",
  "level",
  "message",
  "service",
  "version",
  "trace_id",
  "span_id",
  "logger",
  "error",
  "context",
]);

export type Level = "debug" | "info" | "warn" | "error" | "fatal";

export interface ConfigureOptions {
  level?: string;
  /** Override the destination stream. Used by tests. */
  destination?: NodeJS.WritableStream;
}
