# stoopid-commons

**Status:** draft
**Last reviewed:** 2026-05-02
**Owner:** Jason Anton

## Table of Contents

- [stoopid-commons](#stoopid-commons)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Goals and Non-goals](#goals-and-non-goals)
    - [Goals](#goals)
    - [Non-goals](#non-goals)
  - [Current State](#current-state)
  - [Solution](#solution)
    - [Key components](#key-components)
    - [Significant design decisions](#significant-design-decisions)
  - [Constraints and Assumptions](#constraints-and-assumptions)
    - [Technical](#technical)
    - [Regulatory and organizational](#regulatory-and-organizational)
    - [Assumptions](#assumptions)
  - [Open Questions](#open-questions)
  - [Glossary](#glossary)

## Overview

`stoopid-commons` is a collection of language-spanning utilities and patterns developed at Stoopid Company for building polyglot microservice systems. It produces published packages — for Python, TypeScript, and Rust — and a parameterized Helm chart base that any project can consume to get consistent structured logging, observability conventions, k8s deployment patterns, and common utilities. The repo is open source under MIT license. Its primary internal user is the TrAICE (True AI Computing Environment) platform and the products built on it, but it is intentionally generic and contains nothing specific to TrAICE or any other Stoopid product.

The problem it solves is the inconsistency that emerges when polyglot microservices are built without shared infrastructure: logs in different formats per language, k8s manifests re-authored from scratch per service, ad-hoc retry and correlation patterns, and divergent OpenTelemetry conventions. These differences are individually small but in aggregate make cross-service debugging painful and onboarding slow. Solving it now matters because Stoopid is about to start building TrAICE, a multi-cell architecture where this divergence would compound quickly; establishing the shared foundation before the cells are written is significantly cheaper than retrofitting consistency later.

## Goals and Non-goals

### Goals

- Publish per-language structured logging packages (Python via structlog, TypeScript via pino, Rust via tracing) that produce JSON output conforming to a single shared schema, so logs from any service in any language can be aggregated and queried uniformly.
- Publish a parameterized Helm chart base (`stoopid-service-base`) that services extend via `values.yaml`, eliminating the need for services to author k8s manifests from scratch for common deployment patterns.
- Publish per-language OpenTelemetry helper packages with shared span naming and attribute conventions, so traces are consistent across services.
- Publish per-language utility packages for correlation ID generation and propagation, retry with exponential backoff, and circuit breaker patterns.
- Provide reusable build tooling (Makefile patterns, pre-commit configurations, GitHub Actions workflow templates) that any cell or service can extend.
- All packages versioned via semver, with versions enforced by conventional-commit-driven CI automation. Breaking changes require an explicit `BREAKING CHANGE:` marker.
- Maintain the repo as a real public artifact: working examples, accept external issues, respond to security disclosures.

### Non-goals

- Anything specific to the TrAICE platform or products built on it (e.g., VICI). Platform-specific contracts, schemas, and types live in a separate private repo.
- Application-level frameworks. This repo provides building blocks, not opinionated frameworks (no service generator, no scaffolded application templates beyond the Helm chart base).
- Service mesh, ingress controller, or other cluster-level infrastructure. The Helm base assumes a working cluster with standard primitives.
- Language support beyond Python, TypeScript, and Rust (deferred). Adding Go, Java, or other languages is out of scope for v1 but may be revisited if needed.
- A central logging or tracing backend. This repo defines conventions for emitting logs and traces; backends (Loki, Tempo, Grafana Cloud, etc.) are the consumer's choice.
- Cross-language RPC or schema definitions. Those concerns belong in project-specific repos (e.g., the TrAICE platform's contracts repo).
- A static documentation site (deferred). Markdown READMEs and per-package docs are sufficient until external usage warrants a site.
- Backward compatibility with non-current versions of language toolchains (Python <3.12, Node <20, Rust <1.75).

## Current State

Nothing is built yet. This document is being written as the foundation for the initial implementation.

- **Capability:** none yet
- **Known gaps:** all packages and chart base need to be authored from scratch
- **Known issues:** n/a

The repo template has been created. Next steps are filling out the cell-level documentation (`README.md`, `CONTRIBUTING.md`, this spec, ADRs), establishing the build and CI tooling, then implementing the first package (logging) as the proof-of-concept for the multi-language publishing pipeline.

## Solution

`stoopid-commons` is a polyglot monorepo. Each language has its own subdirectory under `packages/` (e.g., `packages/python/`, `packages/ts/`, `packages/rust/`) containing the language-specific package implementations. A shared `charts/` directory holds the Helm chart base. Build tooling (`scripts/`, `Makefile`, `.github/`) is shared at the repo root.

CI is responsible for: validating each package on every PR (lint, type check, unit tests, integration tests where applicable), generating any cross-language artifacts that need to stay synchronized (such as JSON schema definitions for the logging format), and publishing packages to public registries on merge to `main`. Conventional commits drive semver bumps; CI tags releases and updates per-package CHANGELOGs automatically.

Consumers do not clone this repo. They install published packages from PyPI, npm, crates.io, and the chart repository at pinned versions. Local development against in-flight changes uses a local registry (Verdaccio for npm, devpi for Python) per the workflow in `CONTRIBUTING.md`.

A diagram of the repo structure and publishing flow belongs in `diagrams/`. To be added in the next pass.

### Key components

- **logging packages** — Per-language structured logging wrappers producing a common JSON schema. `packages/python/stoopid-logging/`, `packages/ts/stoopid-logging/`, `packages/rust/stoopid-logging/`.
- **otel packages** — Per-language OpenTelemetry helpers with shared conventions. `packages/python/stoopid-otel/`, `packages/ts/stoopid-otel/`, `packages/rust/stoopid-otel/`.
- **utils packages** — Per-language utilities (correlation IDs, retry, backoff, circuit breaker). `packages/python/stoopid-utils/`, `packages/ts/stoopid-utils/`, `packages/rust/stoopid-utils/`.
- **service-base Helm chart** — Parameterized chart that services extend via `values.yaml`. `charts/stoopid-service-base/`.
- **build tooling** — Shared Makefile, pre-commit config, GitHub Actions workflow templates. `Makefile`, `.pre-commit-config.yaml`, `.github/workflows/`.
- **examples** — End-to-end usage examples per package. `examples/`.

### Significant design decisions

Significant decisions live in [`adrs/`](./adrs/) as ADRs. Reference them inline where relevant rather than restating them here. ADRs to be authored as decisions are made; initial set will include:

- ADR-0001: Polyglot monorepo with per-language `packages/` subdirectory structure
- ADR-0002: Conventional commits + semver enforced via CI automation
- ADR-0003: `main`-only branching with tags as deploy targets
- ADR-0004: Common JSON schema for structured logging across all language packages
- ADR-0005: Parameterized Helm chart base over per-service manifest authoring
- ADR-0006: Local package registry workflow (Verdaccio + devpi) for in-flight development

## Constraints and Assumptions

### Technical

- Must support consumption from Python ≥ 3.12, Node.js ≥ 20, Rust ≥ 1.75.
- Helm chart base targets Kubernetes ≥ 1.28 with Helm ≥ 3.12.
- Logging packages must produce output compatible with standard log aggregators (Loki, Elasticsearch, CloudWatch Logs) without custom parsers.
- OpenTelemetry conventions must conform to the OTel semantic conventions specification — divergence breaks interop with any standard OTel backend.
- Build and test must complete in CI in under 10 minutes for the full matrix to keep PR feedback fast.
- All published packages must be reproducibly buildable from a tagged commit.

### Regulatory and organizational

- MIT license — no copyleft obligations on consumers.
- Publishing to public registries means once published, a version cannot be unpublished or modified. Versioning discipline is mandatory.
- Security disclosures handled per `SECURITY.md`. Response SLA: acknowledge within 5 business days, fix or workaround within 30 days for high-severity issues.
- No PII or proprietary information may be committed. The repo is public from day one.

### Assumptions

- Python's structlog, Node's pino, and Rust's tracing libraries will continue to be the canonical structured logging libraries for their respective ecosystems for the foreseeable future. If any of them is deprecated or supplanted, the corresponding `stoopid-logging` package will need to be re-implemented.
- The OpenTelemetry semantic conventions will continue to evolve in a backward-compatible manner. Breaking changes in OTel would require a major version bump of the otel packages.
- GitHub will continue to host this repo and its CI. Migration to another platform would require rewriting `.github/workflows/`.
- ghcr.io and the public language registries (PyPI, npm, crates.io) will remain available for publishing.
- Internal Stoopid projects (notably TrAICE cells and the products built on them) will be the primary consumers initially. External adoption is welcome but not assumed.

## Open Questions

- Should we publish the Helm chart base via a GitHub Pages-hosted chart repository, or use the OCI artifact pattern via ghcr.io? — Jason, 2026-05-02
- Do we want a CLI tool for scaffolding new services that consume `stoopid-commons` packages, or is a copy-paste-able example sufficient? — Jason, 2026-05-02
- Should the logging schema include OTel trace and span IDs as first-class fields, or only as a generic `context` object? — Jason, 2026-05-02
- What's the minimum viable test matrix for CI — do we test against multiple Python/Node/Rust versions, or only the minimum supported? — Jason, 2026-05-02
- Do we want a `stoopid-config` package for shared configuration loading patterns (env vars, config files, secrets), or is that scope creep? — Jason, 2026-05-02

## Glossary

- **Cell** — A top-level repo organizing related services and packages by function. Concept comes from the TrAICE platform; `stoopid-commons` is generic infrastructure that cells (TrAICE or otherwise) can consume.
- **Conventional commits** — A commit message format (`feat:`, `fix:`, `BREAKING CHANGE:`, etc.) that enables automated semver bumping and CHANGELOG generation.
- **Helm chart base** — A parameterized Helm chart that other charts extend via `dependencies` and override via `values.yaml`. Single source of truth for common k8s deployment patterns.
- **Polyglot monorepo** — A single repo containing code in multiple programming languages, with shared tooling and coordinated releases.
- **Service base** — Specifically `stoopid-service-base`, the Helm chart that services in any cell extend for their k8s deployment.
- **Stoopid Company** — The organization owning this repo and the broader TrAICE platform.
- **TrAICE** — True AI Computing Environment. The cognitive architecture platform Stoopid is building. `stoopid-commons` provides generic infrastructure used by TrAICE cells, but TrAICE itself is not part of this repo.
- **VICI** — Virtually Intelligent Chat Interface. A chat product built on top of TrAICE. Mentioned here only for context; not relevant to `stoopid-commons` consumers.
