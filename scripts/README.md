# Scripts

Supporting scripts that aren't part of any package or service. This directory
holds repository-level tooling — linters, validators, generators, one-off
utilities — that operate on the project as a whole.

## What belongs here

- Validators that check repository-wide invariants (e.g. `adr-lint.py`)
- Generators that produce documentation, reports, or scaffolding
- Migration helpers that don't fit inside any one service
- Utility scripts called by `Makefile` targets, CI workflows, or pre-commit hooks

## What does **not** belong here

- Code that is part of a package or service — that lives in `packages/` or `services/`
- Generic developer tooling already provided by language ecosystems (use the tool directly)
- One-off scripts that won't be re-run — keep those in a personal scratch directory, not in version control

## Conventions

### Language

Default to Python for any script over ~30 lines. Bash is acceptable for
short, obvious scripts (a few `find` and `sed` invocations). Past 30 lines,
bash becomes harder to maintain than Python and the case for switching is
strong.

### Self-contained Python

Python scripts use [`uv` inline script
metadata](https://docs.astral.sh/uv/guides/scripts/) so they have no
project-level dependencies. The shebang and metadata block at the top of
the file declare what the script needs:

```python
#!/usr/bin/env -S uv run --quiet --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
```

This means scripts can be invoked directly (`./scripts/foo.py`) without
activating a virtualenv or running `uv sync`. If `uv` is not installed,
they can still be run as `python3 scripts/foo.py` provided the dependencies
are available.

Prefer stdlib over external dependencies. If a script needs an external
package, weigh whether the script earns its dependency cost.

### Naming

Hyphenated, lowercase, descriptive of what the script does:

```text
adr-lint.py
docs-staleness-check.py
generate-codeowners.sh
```

Avoid underscored names. Scripts are CLI tools, not Python modules — they
are never imported.

### Executable bit

All scripts are committed with the executable bit set:

```sh
chmod +x scripts/<name>
```

Verify after creating a new script.

### Exit codes

Scripts use conventional exit codes:

- `0` — success
- `1` — checks failed (the script ran correctly but found problems)
- `2` — invocation error (bad arguments, missing files, environment issue)

This separation matters in CI — a `1` is a "fix the code" signal, a `2` is
"fix the workflow" signal.

### Arguments

Use `argparse` (Python) or document `getopts` flags (bash). Every script
supports `--help` and prints something useful.

## Invocation

Scripts are typically invoked via:

- A `Makefile` target (preferred for anything a developer runs locally)
- A CI workflow step
- A pre-commit hook
- Direct invocation when debugging

Avoid invoking scripts from inside other scripts unless there is a clear
reason. Composition belongs in the `Makefile` or workflow YAML, not buried
in script chains.

## Index

<!-- Update this section when adding or removing scripts. -->

| Script                         | Purpose                                                  |
| ------------------------------ | -------------------------------------------------------- |
| [`adr-lint.py`](./adr-lint.py) | Validate the integrity of architectural decision records |
