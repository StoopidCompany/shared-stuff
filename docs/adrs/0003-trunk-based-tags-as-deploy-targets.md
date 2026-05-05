# ADR-0003: `main`-only branching with tags as deploy targets

**Status:** accepted
**Date:** 2026-05-02
**Deciders:** Jason Anton

## Context

`stoopid-commons` is a library and chart repository. Its consumers — TrAICE
cells today, external adopters tomorrow — depend on it by version pin, never
by branch. The repo is small enough, and its CI gating is strong enough
(linting, tests, type checks, coverage threshold, secret scanning, CodeQL),
that a single trunk with required-PR-review and required-CI is sufficient
gating for `main` to be continuously shippable.

Long-lived release branches (`develop`, `release/*`, `next`) carry costs:

- Backports double the surface area for any fix that must reach an older
  major.
- Version skew between branches drifts and must be reconciled.
- Tag-pinned consumers care about _tags_, not branches; branch ceremony does
  not buy them anything.
- For a v0/v1 library with no production consumers yet, none of those costs
  is justified.

Tags are the natural deploy target because release-please (see ADR-0002)
already creates them, they are immutable by convention, and they uniquely
identify "what version is what code."

## Decision

The repository uses one long-lived branch — `main` — with the following
rules:

- All work happens on short-lived feature branches off `main`.
- Branches merge to `main` via pull request after code-owner review and CI
  pass.
- Squash-merge is the default; the squash commit message uses the PR title
  to preserve Conventional Commits format.
- Releases are git tags created by release-please on merges that contain
  releasable commits.
- **Tags, not branches, are the deploy target.** Consumers pin to versions
  (i.e. tags); the `main` branch HEAD is not itself a deployable artifact.
- Force-push to `main` is forbidden. The local `make push` target enforces
  no-divergence on `main`. Force-push to feature branches is allowed before
  review.
- No `develop`, `release/*`, or `next` branches exist by default.

If a future scenario demands a back-patch to an older major (e.g., a major
external consumer pinned to an unsupported major needs a security fix), the
exception is to create a long-lived `release/N.x` branch _only at that
moment_, not preemptively.

## Consequences

### Positive

- Linear, auditable history. `git log main` is the project's canonical
  timeline.
- No backport overhead by default; fixes go forward only.
- Tags are the single source of truth for "what version corresponds to what
  code."
- Lower cognitive load for contributors: there is one branch to be aware of.

### Negative

- Hotfixes for older majors require a deliberate exception (a long-lived
  `release/N.x` branch created when the need arises).
- The first time a back-patch is required, the team will incur a one-time
  cost to set up the back-patch branch and re-run release tooling against
  it.

### Neutral

- The deploy target for downstream consumers (Helm chart base versions
  consumed by TrAICE, package versions installed from PyPI/npm/crates.io) is
  always a tag.
- This decision is consistent with ADR-0002: release-please creates the tags;
  this ADR records that the tags are what consumers pin to.

## Alternatives Considered

- **GitFlow (`develop` + `release/*` + `main`)** — rejected. Adds ceremony
  per release without buying anything for a library repo with strong CI on
  trunk.
- **Trunk + per-minor `release/X.Y` branches preemptively** — rejected. Not
  yet justified by demand; the cost-of-introduction is low if a back-patch
  becomes necessary.

## References

- [docs/README.md](../README.md) — "Solution" section.
- `Makefile` — `push` target's main-divergence check.
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — "Branching and Workflow".
- ADR-0002 — release-please owns tag creation.
