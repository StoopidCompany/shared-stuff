# ADR-0001: Polyglot monorepo with per-language `packages/` subdirectories

**Status:** accepted
**Date:** 2026-05-02
**Deciders:** Jason Anton

## Context

Stoopid is about to start building TrAICE, a multi-cell architecture whose
services span Python, TypeScript, and Rust. Without shared infrastructure, each
language ecosystem will diverge in logging format, OpenTelemetry conventions,
retry semantics, correlation propagation, and Kubernetes deployment patterns.
Those differences are individually small but in aggregate make cross-service
debugging painful and onboarding slow.

The shared infrastructure is naturally cross-cutting: a logging schema bump
must land in Python, TypeScript, and Rust simultaneously, or the cross-language
log aggregation it exists to enable breaks. The same applies to OTel attribute
conventions, correlation ID propagation rules, and the JSON schema definitions
for the logging packages.

Three structural options exist:

1. One repo per language (three or more repos).
2. One repo per package (nine-plus repos at v1).
3. One monorepo with per-language subtrees.

Option 1 makes cross-language schema changes a multi-PR coordination problem.
Option 2 multiplies that further and adds release-coordination tax. Option 3
keeps cross-cutting changes atomic at the cost of CI matrix breadth.

## Decision

`stoopid-commons` is a single repository with the following structure:

```
packages/
  python/<package-name>/
  ts/<package-name>/
  rust/<package-name>/
charts/
  stoopid-service-base/
scripts/
.github/
Makefile
```

Each language sub-tree owns its own lockfile and build configuration. The
`Makefile` detects the presence of language manifests (`pyproject.toml`,
`package.json`, `Cargo.toml`) and dispatches to language-native tooling. The
`charts/` directory holds the parameterized Helm chart base. Cross-language
artifacts (e.g. JSON schema definitions referenced by all three logging
packages) live at the root or under `schemas/`.

## Consequences

### Positive

- Cross-language changes (logging schema, OTel attribute conventions, common
  utility semantics) are atomic in a single PR.
- Single CI configuration, single CONTRIBUTING/SECURITY/CODEOWNERS, single
  pre-commit configuration, single dev-tooling baseline.
- The `Makefile` becomes the one entry point for `bootstrap`, `build`, `test`,
  `lint`, `format`, and `check` regardless of language.
- New language packages do not require new repos, branch protection rules, or
  release pipelines.

### Negative

- The CI matrix grows multiplicatively as packages multiply, which presses on
  the under-10-minute CI budget.
- Contributors need at least cursory familiarity with all three toolchains to
  navigate the repo even if they only modify one language.
- `release-please` configuration must be aware of multiple per-package
  versioned roots within the same repo.

### Neutral

- No attempt is made at a unified cross-language build (no Bazel, no Nx).
  Each language stays on its native tooling.
- The repo can later add or remove language sub-trees without affecting the
  others, subject to ADR-0002's versioning discipline.

## Alternatives Considered

- **One repo per language** — rejected. Cross-language schema sync becomes a
  manual coordination problem, and each repo carries duplicate scaffolding
  (CI, pre-commit, contributor docs).
- **One repo per package** — rejected. Combinatorial explosion of repos for a
  small team; coordinated releases across packages become impractical.
- **Bazel/Nx-style unified build** — rejected. The tooling tax (build
  configuration in a non-native language, contributor onboarding cost) exceeds
  the benefit at this scale; the per-language tooling is mature enough that
  unification offers little.

## References

- [docs/README.md](../README.md) — project specification, "Solution" section.
- `Makefile` — language detection logic.
