# ADR-0005: Parameterized Helm chart base over per-service manifest authoring

**Status:** accepted
**Date:** 2026-05-02
**Deciders:** Jason Anton

## Context

Every service deployed in a Stoopid Kubernetes cluster needs roughly the
same set of primitives: a `Deployment` with sane probe defaults, a
`Service`, a `HorizontalPodAutoscaler`, a `ServiceMonitor` for Prometheus,
a `NetworkPolicy`, a `PodDisruptionBudget`, ConfigMap and Secret wiring,
sidecar slots for OpenTelemetry, and a consistent label and annotation
scheme. Authoring these per service produces:

- Inconsistent probe defaults (some services miss readiness probes;
  others have wrong startup-probe budgets).
- Divergent label schemas (different keys for `app`, `service`,
  `version`), which break standard selectors and dashboards.
- Security-context drift (some services run as root, others not).
- One-off bugs in manifest templating that are caught only in production.
- Per-service redundancy that grows as the fleet grows.

A shared parameterized chart base eliminates that duplication. Services
override behavior via `values.yaml` rather than re-authoring manifests.
Bug fixes and security tightening (e.g. a CVE-driven default change)
become one-place changes that propagate by chart version bump.

The Helm ecosystem already supports this pattern via library/parent
charts; the repo's `charts/stoopid-service-base` directory is reserved
for it.

## Decision

`stoopid-commons` ships a parameterized Helm chart base named
`stoopid-service-base` under `charts/stoopid-service-base/`. Services
consume it via Helm dependency in their own `Chart.yaml` and override
behavior through `values.yaml`.

The base owns and templates:

- `Deployment` shape, including container spec, env wiring, and resource
  defaults.
- Probe defaults (liveness, readiness, startup) with sane budgets that
  consumers can override.
- A standard label schema, including OpenTelemetry resource attributes
  (`service.name`, `service.version`).
- Security context defaults (non-root, read-only root filesystem,
  dropped capabilities).
- `Service`.
- `ServiceMonitor` template (consumers opt in via values).
- Common annotations (e.g. for Prometheus scraping, OpenTelemetry
  instrumentation hints).

Consumers extend the base with their own templates only when they need
behavior the base does not cover. Forking the base is discouraged; if a
behavior is widely needed, it should be promoted into the base via a
chart version bump.

Distribution mechanism (GitHub Pages chart repository versus OCI
artifacts in ghcr.io) is **deliberately deferred** and is tracked as an
open question in [docs/README.md](../README.md). This ADR records the
architectural choice (parameterized base over per-service authoring),
not the transport.

## Consequences

### Positive

- New service Kubernetes setup becomes a `values.yaml` file. The
  developer-velocity gain is large.
- Probe, label, and security-context drift becomes near-zero across
  the fleet.
- One place to fix a CVE-driven security default. A chart bump
  propagates the fix.
- The label schema (especially OTel resource attributes) is enforced
  by template, which makes Grafana dashboards and Loki/Tempo
  correlation work without per-service tuning.

### Negative

- Base chart upgrades are coordinated migrations across all consuming
  services. SemVer discipline matters for chart versions just as much
  as for package versions.
- Values surface area must be designed carefully — too narrow and
  consumers fork; too wide and the chart becomes unmaintainable.
- Consumers must learn how Helm parent/library charts compose; the
  cognitive load is non-zero, even if smaller than per-service
  authoring.

### Neutral

- Distribution mechanism (chart repo via GitHub Pages versus OCI via
  ghcr.io) is deferred and does not affect the architectural choice
  recorded here.
- Cluster-level infrastructure (service mesh, ingress controller) is
  out of scope per `docs/README.md` non-goals; the base assumes those
  exist as cluster primitives.

## Alternatives Considered

- **Kustomize base + overlays** — rejected. Less ergonomic than Helm
  for value-driven configuration, and Stoopid's ecosystem is already
  Helm-centric.
- **Per-service hand-authored manifests** — rejected. This is the
  status-quo problem the chart base solves.
- **Cookiecutter / scaffold tool that generates per-service
  manifests** — rejected. Drift returns the moment the scaffold ages;
  generated artifacts are not propagated by chart version bump.

## References

- [docs/README.md](../README.md) — "Solution", "Key Components", and
  "Open Questions" sections.
- ADR-0002 — chart versions follow the same SemVer + release-please
  discipline as language packages.
