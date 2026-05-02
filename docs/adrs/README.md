# Architectural Decision Records

This directory contains the architectural decision records (ADRs) for this
project. ADRs document the significant choices made during the project's
lifetime — the choices that are non-obvious, hard to reverse, or that affect
multiple components.

## When to write an ADR

Write an ADR when a decision is:

- **Non-obvious** — a future engineer might reasonably wonder why this choice was made.
- **Hard to reverse** — once made, undoing it requires meaningful migration work.
- **Cross-cutting** — it affects multiple components, services, or teams.
- **Externally constrained** — driven by regulation, contract, or an organizational mandate that should be recorded.

Do **not** write an ADR for:

- Implementation details that are clear from the code itself.
- Reversible defaults (e.g. choice of CSS framework in a single-page UI).
- Decisions covered by an existing ADR — supersede or amend it instead.

If you are unsure whether something deserves an ADR, the cost of writing one
is low and the cost of not having one years later is high. Write the ADR.

## Format

Every ADR follows the [Michael Nygard
template](https://github.com/joelparkerhenderson/architecture-decision-record/blob/main/locales/en/templates/decision-record-template-by-michael-nygard/index.md)
extended with a few sections we have found useful:

- **Status** — proposed / accepted / deprecated / superseded by ADR-NNNN
- **Date** — when the decision was made (or last updated)
- **Deciders** — who agreed to it
- **Context** — what forces are at play
- **Decision** — the choice made
- **Consequences** — what becomes true, easier, and harder
- **Alternatives Considered** — what was rejected and why
- **References** — optional, links to prior art and related ADRs

The canonical template is in [`template.md`](./template.md).

## Lifecycle

ADRs are append-only as a body of work. Individual ADRs progress through
statuses; they are never deleted.

```text
proposed → accepted → (deprecated | superseded by ADR-NNNN)
```

- **proposed** — the decision is drafted but not yet ratified. Open for discussion.
- **accepted** — the decision is in effect.
- **deprecated** — the decision no longer applies, but no replacement exists. Rare.
- **superseded by ADR-NNNN** — the decision has been replaced. Link to the replacement.

When superseding an ADR, update the old one's status to
`superseded by ADR-NNNN` and add a one-line note pointing at the new ADR.
Never edit the body of a superseded ADR — its content is the historical
record.

## Numbering

ADRs are numbered sequentially with a four-digit zero-padded prefix:

```text
0001-use-postgres-for-primary-storage.md
0002-adopt-conventional-commits.md
0003-deploy-via-helm.md
```

Numbers are never reused. If an ADR is abandoned before acceptance, leave a
short stub explaining why.

## Creating a new ADR

Use the Makefile target from the repository root:

```sh
make adr TITLE="Use Postgres for primary storage"
```

This finds the next available number, copies [`template.md`](./template.md),
and fills in the title, date, and slug. It does not commit the file —
review and edit before committing.

## Index

<!-- Update this section when adding or changing ADRs. Do not auto-generate -->
<!-- — the index is part of the documentation, and a hand-curated list reads -->
<!-- better than a `ls`-style dump. -->

- _(no ADRs yet)_
