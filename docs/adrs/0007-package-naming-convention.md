# ADR-0007: Package naming convention across registries

**Status:** accepted
**Date:** 2026-05-02
**Deciders:** Jason Anton

## Context

`stoopid-commons` publishes packages to four public, append-only namespaces:
PyPI (Python), npm (TypeScript), crates.io (Rust), and a Helm chart
repository (charts). Package names on those registries are effectively
permanent: a name reserved at first publish cannot be cleanly renamed
without abandoning the original and re-onboarding every consumer. Naming
must therefore be decided once, deliberately, and applied consistently
across all four registries.

Each ecosystem has its own naming conventions:

- **PyPI** — bare names with hyphens (`stoopid-logging`). Distribution
  name vs. import name can differ; the import name uses underscores
  (`stoopid_logging`).
- **npm** — bare names (`stoopid-logging`) or scoped names
  (`@stoopid/logging`). Scopes provide namespace protection but introduce
  asymmetry with the other registries.
- **crates.io** — bare names with hyphens (`stoopid-logging`). No
  namespace concept.
- **Helm** — chart names with hyphens (`stoopid-service-base`). No
  namespace concept beyond the chart repository itself.

Three plausible conventions exist:

1. **Bare `stoopid-<thing>` everywhere.** Simple, consistent, easy to
   discover by search.
2. **Scoped npm + bare elsewhere.** Idiomatic on npm, asymmetric across
   languages.
3. **Always-prefixed `stoopid-commons-<thing>`.** Self-documenting that
   the artifact ships from this repo, but verbose and treats the *repo
   name* as part of every *artifact name*.

Three of the planned first-target packages share a common name across
languages: a Python `stoopid-logging`, a TypeScript `stoopid-logging`,
and a Rust `stoopid-logging`. Each lives on a different registry, so the
bare-name collision only matters for git-tag disambiguation, which is
handled separately (see Decision).

## Decision

Use bare `stoopid-<thing>` package names across PyPI, npm, and crates.io.
Helm charts use `stoopid-<thing>` (e.g. `stoopid-service-base`).

- The published distribution name is `stoopid-<thing>` on every registry.
- For Python, the corresponding import name is `stoopid_<thing>` per PEP 8
  module conventions.
- npm packages are **not** scoped (`@stoopid/<thing>` is rejected).
- The repo-name prefix `stoopid-commons-` is **not** used as a package
  prefix; "commons" describes the *repo*, not the *artifacts*.
- Identical published names across languages (e.g. three different
  `stoopid-logging` packages on three different registries) are
  disambiguated in git tags by a language suffix on the release-please
  `component`: `stoopid-logging-py`, `stoopid-logging-ts`,
  `stoopid-logging-rs`. The suffix appears in the tag, not in the
  published package name.

Before first publish to a registry, package authors verify that the
chosen name is available; if not, the fallback is to publish as
`@stoopid/<thing>` on npm (acceptable degraded path) or to negotiate a
different name and update this ADR. Do not silently rename without an
ADR change.

## Consequences

### Positive

- Consistent and discoverable: searching "stoopid-" on any of the three
  language registries surfaces the full set.
- Easy to document and remember; no per-registry rule for contributors
  to learn.
- Trivial mapping from package directory (`packages/python/stoopid-logging/`)
  to published name (`stoopid-logging`).

### Negative

- Bare names assume availability on each registry. Each first-publish
  must verify availability; collisions force a per-package fallback.
- npm without a scope provides no namespace squat-protection. Mitigated
  by publishing the names early (before any external attention) and by
  the documented fallback to `@stoopid/<thing>` if a collision actually
  occurs.
- Identical published names across languages require explicit git-tag
  disambiguation via the `component` field in
  `release-please-config.json` — without it, three packages would all
  produce tags like `stoopid-logging-v0.1.0` and collide.

### Neutral

- Helm chart naming follows the same convention; no extra rule needed.
- This decision is independent of the chart distribution mechanism (the
  GitHub Pages vs OCI question tracked in `docs/README.md` open
  questions).

## Alternatives Considered

- **Scoped npm (`@stoopid/<thing>`) + bare elsewhere** — rejected.
  Adds asymmetry between languages for marginal namespace-protection
  benefit on a single registry. The fallback path remains available if
  a real collision forces it.
- **Always-prefixed `stoopid-commons-<thing>`** — rejected. Verbose,
  duplicates information already implied by the repo, and conflates the
  repository name with the artifact name. Consumers should not need to
  know which repo a package came from to use it.
- **Per-language idiomatic naming (e.g. `stoopid_logging` Python
  distribution name)** — rejected. PEP 8 governs *import* names, not
  *distribution* names; the distribution name is a marketing and
  discovery surface, and consistency across languages outweighs
  per-ecosystem idiom.

## References

- ADR-0002 — publication discipline; once published, names are
  permanent.
- `release-please-config.json` — `package-name` and language-suffixed
  `component` per package.
- [docs/README.md](../README.md) — Solution section and goals.
