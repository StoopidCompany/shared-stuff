# ADR-0008: Resolve log-event trace_id/span_id placement and structured-context shape

**Status:** accepted
**Date:** 2026-05-05
**Deciders:** Jason Anton

## Context

[ADR-0004](0004-shared-logging-json-schema.md) defined the shared logging
JSON schema and pinned five required fields (`timestamp`, `level`,
`message`, `service`, `version`) plus an undefined "structured context"
object for caller-supplied fields. ADR-0004 deliberately deferred two
specific design questions, calling them out as open in
[docs/README.md](../README.md):

1. **Trace/span ID placement.** Should OpenTelemetry `trace_id` and
   `span_id` live as top-level fields on every log event, or be nested
   under a context/tracing object?
2. **Structured-context shape.** Should the caller-supplied "bindings"
   from structlog (Python), child-logger fields (pino, TypeScript), and
   span fields (`tracing` crate, Rust) flatten at the top level — risking
   collisions with reserved field names — or nest under a single object?

The first cut of the three logging packages (`stoopid-logging-py`,
`stoopid-logging-ts`, `stoopid-logging-rs`) needs both questions
answered; v0.1.0 cannot ship with the shape unspecified, because once
published to public registries, the schema is effectively frozen
([ADR-0007](0007-package-naming-convention.md), and the SemVer discipline
in [ADR-0002](0002-conventional-commits-and-semver.md)).

This ADR resolves both questions.

## Decision

**Trace/span IDs are top-level fields.**

- `trace_id` is a 32-character lowercase hexadecimal string (W3C Trace
  Context trace-id format).
- `span_id` is a 16-character lowercase hexadecimal string (W3C Trace
  Context span-id format).
- Both are auto-populated from the ambient OpenTelemetry context when an
  OTel SDK is loaded and a span is active; both are **omitted entirely**
  when no context is present (not emitted as `null`, not as empty
  string).
- Neither field is required by the schema.

**User-supplied structured fields nest under `context`.**

- All caller-supplied "bindings" / span fields / extras land inside a
  single nested `context: { ... }` object.
- The top-level schema is **closed**: `additionalProperties: false`.
- The `context` object is **open**: `additionalProperties: true`.
- User input cannot collide with reserved top-level field names because
  it never reaches the top level.
- When `context` would be empty, it is omitted from the emitted event
  rather than serialized as `{}`.

The schema artifact codifying both decisions lives at
`schemas/log-event.schema.json`.

## Consequences

### Positive

- Top-level `trace_id`/`span_id` aligns with the OpenTelemetry logs data
  model and the Elastic Common Schema (ECS). Backends (Loki, Tempo,
  Grafana Cloud, Elasticsearch, CloudWatch) correlate logs and traces
  out of the box, with no per-emitter parsing.
- A closed top-level shape gives reserved-name protection by
  construction — no runtime collision-detection logic is needed in the
  language packages.
- Each language's idiomatic binding model (structlog `bind`, pino child
  loggers, `tracing` span fields) maps cleanly onto a single nested
  object; the per-language code paths converge on the same serialization
  step.

### Negative

- Adding any new top-level field is a breaking change for any consumer
  that asserts unknown-field rejection against the schema. Future schema
  growth (e.g. an `error` object, `host` metadata) requires deliberate
  major-version bumps coordinated across all three packages
  ([ADR-0001](0001-polyglot-monorepo-layout.md),
  [ADR-0002](0002-conventional-commits-and-semver.md)).
- Per-language packages must include best-effort OpenTelemetry context
  extraction — even though the OTel SDK is an optional/peer dependency.
  This is small (~20 LOC per language) but real.

### Neutral

- The `context` field is omitted when empty rather than emitted as `{}`.
  Consumers querying for `context.user_id` must tolerate the field being
  absent on logs with no bindings; this is the standard JSON query
  shape, but worth noting.
- Trace ID and span ID formats follow W3C Trace Context strictly. Any
  non-W3C tracer (e.g. legacy 64-bit trace IDs) cannot populate these
  fields without padding to 32 hex chars; in practice OTel SDKs already
  emit W3C-compliant IDs.

## Alternatives Considered

- **Trace/span IDs nested under `context`** — rejected. Backend
  correlation queries become verbose (`context.trace_id` instead of
  `trace_id`), and most log aggregators have built-in trace correlation
  that expects the IDs at top level. Misaligns with the OTel logs data
  model and ECS, both of which we want to remain compatible with per
  ADR-0004.
- **Open top-level schema with a reserved-name list** — rejected. Field
  collisions become a runtime concern (rename? drop? raise?), each
  language must implement collision logic, and consumers cannot
  schema-validate emitted events without a custom validator that knows
  the reserved set. Closed top-level removes the entire class of
  problems.
- **Flat schema, all fields including user bindings at top level** —
  rejected. User payload pollutes the top-level namespace; new reserved
  fields cannot be added later without breaking existing consumers'
  payloads. Loses the ability to evolve the schema safely.
- **Defer to v0.2** — rejected by the user during planning. Trace
  correlation is the primary value-add of structured logging in the
  TrAICE platform; shipping v0.1.0 without it would force every consumer
  to adopt a v0.1 → v0.2 migration on day two.

## References

- [ADR-0004](0004-shared-logging-json-schema.md) — the parent decision
  that explicitly deferred these questions. Not superseded; this ADR
  resolves the open items it left for follow-up.
- [ADR-0001](0001-polyglot-monorepo-layout.md) — atomic cross-language
  changes (this schema-defining decision ships with all three
  implementations in a single PR).
- [ADR-0002](0002-conventional-commits-and-semver.md) — schema-changing
  releases ride on SemVer discipline.
- [W3C Trace Context](https://www.w3.org/TR/trace-context/) — trace-id
  and span-id format specification.
- [OpenTelemetry Logs Data Model](https://opentelemetry.io/docs/specs/otel/logs/data-model/).
- [Elastic Common Schema](https://www.elastic.co/guide/en/ecs/current/index.html).
- `schemas/log-event.schema.json` — the codified schema this ADR governs.
