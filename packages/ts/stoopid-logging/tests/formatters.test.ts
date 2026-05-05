import {
  bindingsFormatter,
  levelFormatter,
  logFormatter,
  timestampFormatter,
} from "../src/formatters.js";

describe("timestampFormatter", () => {
  it("returns a JSON fragment with a leading comma and quoted timestamp key", () => {
    const out = timestampFormatter();
    expect(out).toMatch(/^,"timestamp":"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z"$/);
  });
});

describe("levelFormatter", () => {
  it.each(["debug", "info", "warn", "error", "fatal"])("passes %s through", (label) => {
    expect(levelFormatter(label)).toEqual({ level: label });
  });

  it("maps trace to debug", () => {
    expect(levelFormatter("trace")).toEqual({ level: "debug" });
  });
});

describe("bindingsFormatter", () => {
  it("strips pid and hostname, passes reserved fields through", () => {
    expect(bindingsFormatter({ pid: 1, hostname: "x", logger: "foo" })).toEqual({
      logger: "foo",
    });
  });

  it("aggregates non-reserved bindings into context", () => {
    expect(bindingsFormatter({ pid: 1, logger: "x", requestId: "r_1" })).toEqual({
      logger: "x",
      context: { requestId: "r_1" },
    });
  });

  it("omits context when no non-reserved bindings", () => {
    expect(bindingsFormatter({ pid: 1, hostname: "x" })).toEqual({});
  });

  it("tolerates undefined bindings", () => {
    expect(bindingsFormatter(undefined)).toEqual({});
  });
});

describe("logFormatter", () => {
  it("aggregates non-reserved fields into context", () => {
    const out = logFormatter({
      level: "info",
      message: "x",
      service: "s",
      version: "v",
      user_id: "u_1",
      request_method: "GET",
    });
    expect(out["context"]).toEqual({ user_id: "u_1", request_method: "GET" });
    expect(out["user_id"]).toBeUndefined();
  });

  it("preserves all reserved top-level fields", () => {
    const out = logFormatter({
      level: "info",
      message: "x",
      service: "s",
      version: "v",
      trace_id: "a".repeat(32),
      span_id: "b".repeat(16),
      logger: "foo.bar",
      user_id: "u_1",
    });
    expect(out["trace_id"]).toBe("a".repeat(32));
    expect(out["span_id"]).toBe("b".repeat(16));
    expect(out["logger"]).toBe("foo.bar");
    expect(out["context"]).toEqual({ user_id: "u_1" });
  });

  it("drops empty context", () => {
    const out = logFormatter({ level: "info", message: "x", service: "s", version: "v" });
    expect(out["context"]).toBeUndefined();
  });

  it("merges existing context with non-reserved fields", () => {
    const out = logFormatter({ level: "info", context: { a: 1 }, b: 2 });
    expect(out["context"]).toEqual({ a: 1, b: 2 });
  });
});
