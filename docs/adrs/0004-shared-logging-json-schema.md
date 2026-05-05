# ADR-0004: Common JSON schema for structured logging across all language packages

**Status:** accepted
**Date:** 2026-05-02
**Deciders:** Jason Anton

## Context

The primary motivating problem for `stoopid-commons` is consistency across
polyglot services: a TrAICE cell composed of Python, TypeScript, and Rust
services should not produce three different log shapes that downstream
aggregators must reconcile. structlog (Python), pino (TypeScript), and the
`tracing` crate (Rust) each emit JSON natively, but their default field
names and types diverge. Without a shared contract, queries fragment along
language lines (`level` vs `severity`, `time` vs `timestamp`, `msg` vs
`message`), correlation across services becomes manual, and every backend
(Loki, Elasticsearch, CloudWatch, Grafana Cloud) needs custom parsing per
emitter.

A shared schema imposes a contract: every logging package emits objects that
conform to it. The schema is the single source of truth; per-language
packages are implementations of it. CI validates emitted output against the
schema, so divergence is caught before publication, not at log-aggregation
time.

This ADR records the existence and required-fields baseline of the schema.
Some specific field-placement questions (notably whether OpenTelemetry trace
and span identifiers belong as top-level fields or nested under a context
object) are deliberately left unresolved here so they can be decided
explicitly with their own tradeoff analysis.

## Decision

`stoopid-commons` defines a single JSON schema that every language's
structured-logging package emits against. The schema lives in the
repository (path determined during implementation, expected
`schemas/log-event.schema.json`) and is the contract.

**Required top-level fields (baseline):**

- `timestamp` — RFC 3339 / ISO 8601 UTC string.
- `level` — lowercase string, one of `debug`, `info`, `warn`, `error`,
  `fatal`.
- `message` — human-readable event description.
- `service` — service name (typically from environment, e.g.
  `OTEL_SERVICE_NAME`).
- `version` — service version.

**Structured context:** an object field carrying caller-supplied structured
fields (the conventional "extra" / "bindings" / fields passed by the
emitter).

**Validation:** each language package's CI suite runs an emit-and-validate
test that produces sample output and validates it against the schema. A
schema change requires synchronized updates to all three packages in the
same PR (atomic cross-language change — see ADR-0001).

**Explicitly deferred:** placement of OpenTelemetry trace and span
identifiers (top-level fields versus nested under the context object) is
deferred to a separate decision. The open question is tracked in
[docs/README.md](../README.md). Logging packages may emit them at whichever
location the follow-up decision specifies; this ADR does not pin it.

## Consequences

### Positive

- Cross-service log queries are uniform across languages: one query shape
  works whether the log came from a Python, TypeScript, or Rust service.
- Backend-agnostic: any standard log aggregator (Loki, Elasticsearch,
  CloudWatch Logs, Grafana Cloud) can consume the output without custom
  parsers.
- Schema is testable: CI catches schema drift before a package is
  published.
- New language packages added later (out of scope for v1) inherit the
  contract for free.

### Negative

- Adding or changing a required field is a coordinated breaking change
  across all three packages, with a corresponding major version bump on
  each package.
- Each language's idiomatic logging style bends slightly to fit the schema
  (e.g., structlog's bound-context model, pino's child-logger pattern, and
  the `tracing` crate's span-fields model must all serialize to the same
  shape).

### Neutral

- The schema is OpenTelemetry-compatible by construction (matching field
  semantics where they overlap) but is not a wholesale adoption of the
  OTel logs data model. A future ADR may revisit OTel logs adoption once
  it is generally available across all three SDKs.
- Trace/span ID placement is left open by design and is the subject of a
  separate decision.

## Alternatives Considered

- **Per-language schemas** — rejected. Defeats the cross-language
  consistency that motivates the project.
- **Wholesale adoption of the OpenTelemetry logs data model** —
  considered. Deferred until OTel logs are stable and well-supported in
  all three language SDKs. The chosen schema is OTel-compatible to keep
  this option open.
- **Loose convention without a machine-checkable schema** — rejected.
  Conventions drift without enforcement; the whole point of doing this in
  shared packages is to remove drift.

## References

- [docs/README.md](../README.md) — "Solution", "Key Components", and
  "Open Questions" sections.
- ADR-0001 — atomic cross-language changes are possible because of the
  monorepo structure.
- ADR-0002 — schema-bumping discipline rides on the SemVer enforcement.
- [OpenTelemetry semantic conventions](https://opentelemetry.io/docs/specs/semconv/).
