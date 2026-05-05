# ADR-0002: Conventional commits + semver enforced via CI automation

**Status:** accepted
**Date:** 2026-05-02
**Deciders:** Jason Anton

## Context

`stoopid-commons` publishes packages to public registries: PyPI for Python,
npm for TypeScript, crates.io for Rust, and a Helm chart repository for
charts. Once published, a version cannot be unpublished or rewritten — the
public registry retention policies are deliberate and irreversible.
Versioning mistakes are permanent.

At v1 the repo will publish on the order of nine packages (three logging,
three OTel helper, three utility) plus the Helm chart base. Manual version
bumps across that surface area are error-prone. Manual CHANGELOG curation is
worse. Consumers — including the TrAICE platform pinned at exact versions —
need to trust that minor and patch bumps are non-breaking, or the value of
SemVer evaporates.

Two disciplines are required:

1. A machine-checkable commit format that encodes intent (feature vs fix vs
   breaking change).
2. Tooling that maps commits to per-package version bumps and CHANGELOG
   entries deterministically.

## Decision

The repo adopts Conventional Commits 1.0 as the commit-message format, and
[release-please](https://github.com/googleapis/release-please) as the release
automation:

- Every commit on `main` follows Conventional Commits format. The PR title
  also follows the format because squash-merge uses the PR title as the
  commit subject.
- `commitlint` (in CI) and `conventional-pre-commit` (locally) reject
  non-conforming messages.
- `release-please` (configured in `release-please-config.json`) computes
  per-package SemVer bumps from commit history, generates CHANGELOG entries,
  opens a release PR, and tags releases on merge.
- A `BREAKING CHANGE:` footer or a `!` after the type/scope triggers a major
  bump on the next release.
- `feat:` produces a minor bump; `fix:` and `perf:` produce patch bumps;
  `chore:`, `ci:`, `build:`, `style:`, `test:`, `refactor:`, `docs:` do not
  appear in CHANGELOGs and do not bump versions.

## Consequences

### Positive

- Version bumps are deterministic and auditable straight from `git log`.
- CHANGELOGs are free.
- The same convention applies uniformly across three languages and the Helm
  chart, which removes per-language process drift.
- Release PRs are reviewable artifacts: a reviewer sees the bump and the
  changelog before publication.

### Negative

- Contributors must learn the commit format. Mitigated by `make commit`,
  pre-commit validation, and PR-title linting that catches mistakes early.
- A bad commit message that lands on `main` cannot be retroactively fixed
  without rewriting public history (which is forbidden — see ADR-0003). The
  solution is a follow-up commit and accurate CHANGELOG editing in the
  release PR.

### Neutral

- The CI test-matrix breadth (which Python, Node, and Rust versions to run
  against) is a separate decision and is deferred to a later ADR. This ADR
  commits to the version-management mechanism only.
- `release-please` is not the only tool that can do this; the choice is
  defensible and reversible if a better fit appears.

## Alternatives Considered

- **Manual versioning per package** — rejected. Coordinating SemVer across
  nine-plus packages by hand will be skipped under deadline pressure, leading
  to version mistakes that cannot be corrected after publication.
- **`semantic-release`** — considered. Mature and capable, but less native to
  multi-language monorepos with multiple versioned roots than `release-please`.
- **Calendar versioning (CalVer)** — rejected. Library consumers expect
  SemVer; CalVer offers no signal about breaking-change risk for a downstream
  upgrade decision.

## References

- [docs/README.md](../README.md) — "Solution" and "Constraints and
  Assumptions" sections.
- `release-please-config.json` — release automation configuration.
- `.pre-commit-config.yaml` — local commit message validation.
- [Conventional Commits 1.0](https://www.conventionalcommits.org/en/v1.0.0/).
