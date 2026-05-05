# Contributing

Thanks for contributing to this project. This document covers how to get a
local environment running, the conventions we follow, and the checks your
changes will be measured against.

For security-related concerns and how to report vulnerabilities, see
[`SECURITY.md`](./SECURITY.md). For expected behavior in project spaces, see
[`CODE_OF_CONDUCT.md`](./CODE_OF_CONDUCT.md).

## Table of Contents

- [Template Setup](#template-setup) _(delete after setup)_
- [Getting Started](#getting-started)
- [Repository Structure](#repository-structure)
- [Branching and Workflow](#branching-and-workflow)
- [Commit Conventions](#commit-conventions)
- [Pull Requests](#pull-requests)
- [Code Quality](#code-quality)
- [Per-Language Conventions](#per-language-conventions)
- [Documentation](#documentation)
- [Releasing](#releasing)
- [Troubleshooting](#troubleshooting)
- [Getting Help](#getting-help)

## Getting Started

### Prerequisites

Install the toolchain for whichever languages this project uses:

- **Git** — current version
- **Make** — for the standard task runner
- **Docker** — for containerized builds and local services
- **mise** _(recommended)_ — manages language runtime versions per project via
  [`.tool-versions`](https://mise.jdx.dev/configuration.html#tool-versions).
  Install with `brew install mise` or `curl https://mise.run | sh`. After
  installation, run `mise install` from the repo root to install the exact
  versions this project requires. [`asdf`](https://asdf-vm.com) is a
  supported alternative; both read the same file.
- **pre-commit** — for git hooks (`pip install pre-commit` or `brew install pre-commit`)
- **Node.js** + **pnpm** — if the project includes TypeScript/JavaScript _(installed automatically by mise)_
- **Python** + **uv** — if the project includes Python _(Python installed automatically by mise; install uv via `pip install uv` or `brew install uv`)_
- **Rust** + **cargo** — if the project includes Rust _(installed automatically by mise)_
- **Terraform** — if the project includes infrastructure code _(installed automatically by mise)_

The source of truth for tool versions is **`.tool-versions`** at the repo
root. Run `mise install` to align your local environment with the versions
this project requires. Language-specific manifests
(`package.json` `engines`, `pyproject.toml` `requires-python`,
`rust-toolchain.toml`) should align with `.tool-versions` and are kept in
sync as part of any version bump.

### Bootstrap

```sh
git clone <repo-url>
cd <repo>
make bootstrap
```

`make bootstrap` installs dependencies, sets up pre-commit hooks, and prepares
any local state required to run the project.

### Local Environment

Copy the example environment file:

```sh
cp .env.example .env
```

Edit `.env` with values appropriate for local development. Real secrets must
come from an external secret store — never commit them. See
[`SECURITY.md`](./SECURITY.md#secrets-management).

### Running Locally

```sh
make dev      # start the project for local development
make test     # run all tests
make lint     # run all linters
make format   # apply formatters
make check    # run full CI-equivalent checks locally
```

For projects with multiple services, the `make dev` target may use
[Tilt](https://tilt.dev) or `docker compose` under the hood. Either is a
reasonable choice; consult the project's `README.md` for specifics.

## Repository Structure

The exact layout varies by project, but common top-level directories include:

```sh
<repo>/
├── .github/             # CI workflows, issue/PR templates, Dependabot config
├── .vscode/             # Shared editor settings and extension recommendations
├── docs/                # ADRs, runbooks, and design documentation
│   ├── adrs/            # Architectural Decision Records
│   └── runbooks/        # Operational procedures
├── packages/            # Reusable libraries, grouped by language
├── services/            # Deployable services, grouped by language
├── scripts/             # Supporting scripts and tooling
├── data/                # Database migrations and schema files
├── .env.example         # Template environment variables
├── .gitignore
├── .editorconfig
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── LICENSE
├── Makefile
├── README.md
└── SECURITY.md
```

Not every project uses every directory. Each `packages/` and `services/`
subtree is grouped by language (e.g. `packages/ts/`, `services/python/`).

## Branching and Workflow

- `main` is the only long-lived branch and is always deployable.
- All work happens on short-lived feature branches off `main`.
- Branches are merged to `main` via pull request after review and CI pass.
- Releases are cut from `main` via tags. Tags are the deploy target — not branches.
- Force-pushes to `main` are prohibited. Force-pushes to feature branches are allowed before review.

### Branch naming

Use a short, descriptive prefix:

- `feat/<short-description>` — new functionality
- `fix/<short-description>` — bug fixes
- `chore/<short-description>` — tooling, dependencies, refactoring with no behavior change
- `docs/<short-description>` — documentation only

Example: `feat/add-rate-limit-middleware`

## Commit Conventions

This project requires [Conventional Commits](https://www.conventionalcommits.org/).
A CI check (commitlint) verifies every commit on every pull request.

### Format

```html
<type
  >(<scope
    >)<!>:
    <subject>
      <body>
        <footer></footer></body></subject></scope
></type>
```

- `<type>` — one of: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`
- `<scope>` — optional. Typically a package or service name.
- `<!>` — optional. Marks a breaking change.
- `<subject>` — imperative mood, lowercase, no trailing period, ≤72 chars
- `<body>` — optional. Wrap at 72 chars. Explain _why_, not _what_.
- `<footer>` — optional. Used for `BREAKING CHANGE:` notes and issue refs.

### Examples

```sh
feat(auth): add refresh token rotation

fix: handle null user agent in request logger

refactor(api)!: rename `getUser` to `findUser`

BREAKING CHANGE: callers of `getUser` must update to `findUser`.
The old name is removed; there is no deprecation period.
```

### Type guidance

- `feat` and `fix` produce CHANGELOG entries. Use them for user-visible changes.
- `chore`, `ci`, `build`, `style`, `test`, `refactor`, `docs` do not appear in the CHANGELOG.
- A breaking change in any type bumps the major version on release.

## Pull Requests

### Before opening

- Rebase on the latest `main`.
- Run `make check` locally and confirm it passes.
- Update or add tests for any behavior change.
- Update documentation if the change affects public interfaces, configuration, or operational concerns.

### PR title and description

- The PR title MUST follow Conventional Commits format. It becomes the squash-merge commit message.
- The description should answer: _what_ changed, _why_, and _how to verify_. Link any relevant issues.
- Use Draft status for work-in-progress.

### Review requirements

- One approval from a code owner is required to merge.
- Code owners are defined in `.github/CODEOWNERS`.
- All required CI checks must pass.
- Conversations must be resolved before merge.

### Merge strategy

Squash and merge is the default. The squash commit message uses the PR title,
which preserves Conventional Commits for the release tooling.

## Code Quality

### Pre-commit hooks

`make bootstrap` installs pre-commit hooks that run formatters and fast
linters on staged files. Do not bypass with `--no-verify` for routine changes.

### CI checks

Every pull request runs:

- Linting and formatting checks for all relevant languages
- Type checking where applicable
- Unit and integration tests
- Test coverage measurement (threshold: **80% on changed lines**, enforced)
- Conventional Commits validation
- Secret scanning and CodeQL (see [`SECURITY.md`](./SECURITY.md))
- Dependency vulnerability checks

A pull request that lowers test coverage below the threshold will be blocked
until coverage is restored. Disabling or skipping tests to meet the threshold
is not acceptable.

### Definition of done

A change is ready to merge when:

- All CI checks pass
- New code has corresponding tests
- Coverage threshold is met on changed lines
- Documentation is updated where affected
- A code owner has approved
- Conversations are resolved

## Per-Language Conventions

### TypeScript / JavaScript

- **Package manager:** pnpm
- **Lockfile:** `pnpm-lock.yaml` is committed
- **Linter:** ESLint
- **Formatter:** Prettier (settings in `.prettierrc` or `package.json`)
- **Testing:** Jest
- **Type checking:** `tsc --noEmit` runs in CI

```sh
pnpm install
pnpm test
pnpm lint
```

### Python

- **Package manager:** uv
- **Lockfile:** `uv.lock` is committed
- **Linter / formatter:** Ruff (replaces black, isort, flake8, pylint)
- **Type checker:** Pyright (via Pylance) or mypy, project-dependent
- **Testing:** pytest
- **Line length:** 88 (Ruff default)

```sh
uv sync
uv run pytest
uv run ruff check .
uv run ruff format .
```

### Rust

- **Build tool:** cargo
- **Lockfile:** `Cargo.lock` is committed for binaries; library `Cargo.lock` policy is per-crate
- **Linter:** clippy with `-D warnings`
- **Formatter:** rustfmt (settings in `rustfmt.toml`)
- **Testing:** `cargo test`
- **Line length:** 100

```sh
cargo build
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

### Terraform

- **Version:** pinned in `.terraform-version` or `versions.tf`
- **Formatter:** `terraform fmt`
- **Validation:** `terraform validate`
- **Linter:** `tflint` recommended

```sh
terraform fmt -check -recursive
terraform validate
```

### Other

For SQL, shell scripts, Dockerfiles, YAML, and Markdown, formatting and
linting tools are configured in the repository as needed. Run `make format`
and `make lint` to apply them across the whole tree.

## Documentation

### What to document

- **Public interfaces** — function signatures, API endpoints, CLI flags, configuration. Document at the source.
- **Architectural decisions** — write an ADR in `docs/adrs/` for any decision that is non-obvious, hard to reverse, or affects multiple components.
- **Operational procedures** — write a runbook in `docs/runbooks/` for any procedure that on-call engineers need to follow.
- **Local dev quirks** — capture in `README.md` or this file.

### What not to document

- Implementation details that are clear from the code itself
- Things that will go stale faster than they get read (specific commit SHAs, ephemeral state, exact log lines)
- Tutorials for things the language ecosystem already documents

### ADR format

Use [Michael Nygard's template](https://github.com/joelparkerhenderson/architecture-decision-record/blob/main/locales/en/templates/decision-record-template-by-michael-nygard/index.md):

- Title
- Status (proposed / accepted / deprecated / superseded)
- Context
- Decision
- Consequences

Number ADRs sequentially: `0001-use-pnpm.md`, `0002-adopt-conventional-commits.md`, etc.

## Releasing

> **Setup:** Releases are managed by `release-please`. Confirm
> `release-please-config.json` has the correct `release-type` (`simple`,
> `node`, `python`, or `rust`) and `package-name` for this project. The
> first release will move the version from `0.0.0` to `0.1.0`.

Releases are triggered by tags on `main`. The release workflow:

1. A release PR is opened (or updated) automatically based on Conventional Commits since the last release.
2. The PR contains the version bump and a generated CHANGELOG entry.
3. Merging the release PR creates a tag.
4. The tag triggers the build and publish workflow.

CHANGELOG entries are generated automatically from `feat:`, `fix:`, and
breaking-change commits. Other commit types are excluded.

### Versioning

This project uses [Semantic Versioning](https://semver.org):

- `MAJOR` — breaking changes
- `MINOR` — new features, backward-compatible
- `PATCH` — bug fixes, backward-compatible

A breaking change in any commit (`feat!:`, `fix!:`, or a `BREAKING CHANGE:`
footer) bumps the major version.

### Manual releases

Manual releases are not standard. If a hotfix path is required, it will be
documented in the project `README.md`.

## Troubleshooting

### `make bootstrap` fails

- Check that all prerequisite tools are installed and on `PATH`.
- Check that you've copied `.env.example` to `.env`.
- Run with `make -d bootstrap` for verbose output.

### Pre-commit hooks are slow

- Hooks run only on staged files. If they're slow, the toolchain is doing too much. Open an issue.
- Do not bypass with `--no-verify` as routine practice.

### CI passes locally but fails in PR

- Confirm you ran `make check`, not just `make test`.
- Verify your toolchain versions match what CI uses (see workflow files in `.github/workflows/`).
- Look for environment differences: case-sensitive filesystems, line endings, locale.

### Local services won't start

- Confirm Docker is running.
- Check for port conflicts with `lsof -i :<port>`.
- Inspect service logs via the orchestrator (Tilt UI, `docker compose logs`, etc.).

## Getting Help

- Check existing issues and discussions in this repository first.
- For project-specific questions, ask in the project's designated chat channel (see `README.md`).
- For security concerns, see [`SECURITY.md`](./SECURITY.md).
- For org-wide engineering questions, use the standard internal channels.
