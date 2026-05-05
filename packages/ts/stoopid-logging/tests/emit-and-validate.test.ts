import { Writable } from "node:stream";

import { configureLogging, getLogger, _resetForTests } from "../src/config.js";
import { logEventSchema, validate } from "./setup.js";

class CaptureStream extends Writable {
  public lines: string[] = [];

  override _write(chunk: Buffer, _enc: string, cb: () => void): void {
    const text = chunk.toString("utf-8");
    for (const line of text.split("\n")) {
      if (line.length > 0) this.lines.push(line);
    }
    cb();
  }
}

function withCapture(setup: () => void): Record<string, unknown>[] {
  const stream = new CaptureStream();
  _resetForTests();
  configureLogging({ destination: stream as unknown as NodeJS.WritableStream });
  setup();
  return stream.lines.map((line) => JSON.parse(line) as Record<string, unknown>);
}

beforeEach(() => {
  _resetForTests();
  process.env["OTEL_SERVICE_NAME"] = "test-service";
  process.env["SERVICE_VERSION"] = "0.0.0-test";
  delete process.env["LOG_LEVEL"];
});

describe("schema self-checks", () => {
  it("schema's own examples validate against the schema", () => {
    for (const example of logEventSchema.examples ?? []) {
      const ok = validate(example);
      if (!ok) console.error(validate.errors);
      expect(ok).toBe(true);
    }
  });
});

describe("emit-and-validate", () => {
  it("emits a minimal valid event", () => {
    const events = withCapture(() => {
      getLogger().info("service started");
    });
    expect(events).toHaveLength(1);
    expect(validate(events[0])).toBe(true);
    expect(events[0]).toMatchObject({
      level: "info",
      message: "service started",
      service: "test-service",
      version: "0.0.0-test",
    });
    expect(events[0]).not.toHaveProperty("context");
  });

  it("places named-logger fields at the top level", () => {
    const events = withCapture(() => {
      getLogger("orders.api").info("request");
    });
    expect(events).toHaveLength(1);
    expect(validate(events[0])).toBe(true);
    expect(events[0]?.["logger"]).toBe("orders.api");
  });

  it("places per-call object fields under context", () => {
    const events = withCapture(() => {
      getLogger("orders.api").info({ requestId: "req_42", durationMs: 18 }, "processed");
    });
    expect(events).toHaveLength(1);
    expect(validate(events[0])).toBe(true);
    expect(events[0]?.["context"]).toEqual({ requestId: "req_42", durationMs: 18 });
    expect(events[0]?.["logger"]).toBe("orders.api");
  });

  it("filters below the configured level", () => {
    const stream = new CaptureStream();
    _resetForTests();
    configureLogging({ level: "warn", destination: stream as unknown as NodeJS.WritableStream });
    const log = getLogger();
    log.info("dropped");
    log.warn("kept");
    log.error("kept");
    const events = stream.lines.map((line) => JSON.parse(line) as Record<string, unknown>);
    expect(events.map((e) => e["level"])).toEqual(["warn", "error"]);
  });

  it("rejects an unknown level at configure time", () => {
    expect(() => configureLogging({ level: "trace" })).toThrow(/Unknown log level/);
  });

  it("child logger non-reserved bindings land under context", () => {
    const events = withCapture(() => {
      const log = getLogger("orders.api");
      const bound = log.child({ requestId: "req_42" });
      bound.info("processed");
    });
    expect(events).toHaveLength(1);
    expect(validate(events[0])).toBe(true);
    expect(events[0]?.["logger"]).toBe("orders.api");
    expect(events[0]?.["context"]).toEqual({ requestId: "req_42" });
  });

  it("merges explicit context with sibling fields", () => {
    const events = withCapture(() => {
      getLogger().info({ context: { a: 1 }, b: 2 }, "merge");
    });
    expect(events).toHaveLength(1);
    expect(validate(events[0])).toBe(true);
    expect(events[0]?.["context"]).toEqual({ a: 1, b: 2 });
  });
});
