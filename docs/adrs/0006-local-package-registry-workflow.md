# ADR-0006: Local package registry workflow for in-flight development

**Status:** accepted
**Date:** 2026-05-02
**Deciders:** Jason Anton

## Context

TrAICE cells and other consumers pull `stoopid-commons` packages from
public registries (PyPI, npm, crates.io) by version pin. During in-flight
development of a `stoopid-commons` change — say a new field in the
logging schema — a consumer needs to validate against the unreleased
package _before_ it ships. Two naïve approaches fail:

1. **Publish a real version to public registries to test it.** Public
   registries are append-only and global. Pre-release tags pollute
   version history with dev noise that cannot be removed. Mistakes are
   permanent (see ADR-0002).
2. **Use path-based local installs (e.g. `pip install -e ../local-path`,
   npm `file:` deps).** Works on one developer machine, but does not
   reproduce in containers or CI. It also does not validate the
   _published_ package layout — the `pyproject.toml` packaging metadata,
   the npm `files` whitelist, the included assets — and so it routinely
   passes locally and breaks at consumer install time.

A useful in-flight workflow needs to (a) avoid touching public
registries, (b) reproduce the exact install path that production will
use, and (c) be cheap to set up and tear down.

The Python and TypeScript ecosystems both have mature local-registry
options:

- **devpi** for Python — a local PyPI mirror and dev-publishing index.
- **Verdaccio** for npm — a local npm proxy/registry.

Both run as a local Docker container, expose a registry URL, and accept
real `twine`/`uv publish` and `npm publish`/`pnpm publish` against them.

Rust is asymmetric: there is no widely-deployed open-source equivalent
to devpi or Verdaccio that is as turnkey. The pragmatic Rust idiom is
Cargo path dependencies for same-machine work and Cargo git
dependencies for cross-repo work, both of which are first-class in
Cargo and reproduce the published package layout (`Cargo.toml` /
`[lib]` / `[bin]` configuration) more faithfully than the Python or
TypeScript path-install equivalents.

## Decision

The canonical in-flight-dev workflow is:

- **Python:** local **devpi** instance. Developers run a docker-compose
  snippet documented in [CONTRIBUTING.md](../../CONTRIBUTING.md),
  publish dev-tagged versions (e.g. `0.5.0-dev.<short-sha>`) to it via
  `uv publish` against the devpi index URL, and have the consumer set
  the appropriate index URL to pull from devpi.
- **TypeScript:** local **Verdaccio** instance. Same pattern: publish
  via `pnpm publish --registry http://localhost:4873`, consume by
  setting `npmrc` registry to the local instance.
- **Rust:** Cargo path dependencies for same-machine dev (most common),
  Cargo git dependencies (against a feature branch) for cross-repo dev.
  No local Cargo registry is required.

Promotion to the public registries follows the normal release-please
flow (see ADR-0002): the dev-tagged versions stay local, and a real
SemVer release is cut by merging to `main` with a Conventional Commit
that triggers the appropriate bump. The dev artifacts are discarded
when the local registry container is torn down.

## Consequences

### Positive

- Zero risk of accidental public publishes during dev. Pre-release
  noise stays off PyPI/npm/crates.io.
- Consumers exercise the exact install path that production will use,
  so packaging-metadata mistakes (missing files, wrong entry points)
  are caught before publication.
- Local registries are trivial to tear down (`docker compose down`).
- Cargo path/git deps are first-class and well-understood by Rust
  developers.

### Negative

- Python and TypeScript developers run an additional local container.
  Mitigated by a docker-compose snippet documented in
  [CONTRIBUTING.md](../../CONTRIBUTING.md).
- Asymmetry between Rust (path/git deps) and Python/TypeScript (local
  registry). The asymmetry is documented but real, and contributors
  switching languages must remember it.

### Neutral

- This decision does not preclude adopting a self-hosted Cargo
  registry (kellnr, shipyard) later if Rust path/git deps prove
  insufficient — for example, if cross-org Rust consumers proliferate
  and want a Verdaccio-equivalent dev path.
- It also does not preclude using GitHub Packages or another private
  registry for Python or TypeScript if needs change.

## Alternatives Considered

- **Path/file installs only (e.g. `pip install -e`, npm `file:`)** —
  rejected for Python and TypeScript. Does not validate published
  packaging metadata; ships hidden bugs to first real consumer at
  install time.
- **Pre-release tags published to public registries during dev** —
  rejected. Public registries are append-only; dev-tagged versions
  pollute version history permanently and cannot be un-published.
- **Tarball drops in PR comments / shared-drive artifacts** —
  rejected. Not reproducible; not scriptable in CI; high friction.
- **Self-hosted Cargo registry (kellnr, shipyard) for Rust at v1** —
  considered, deferred. Cargo path/git deps cover the v1 use cases
  without operational overhead. Revisit if usage demands it.

## References

- [docs/README.md](../README.md) — "Solution" section, local-registry
  workflow.
- [CONTRIBUTING.md](../../CONTRIBUTING.md) — local-development
  workflow (to be filled in with docker-compose snippet).
- ADR-0002 — public-registry publication discipline.
