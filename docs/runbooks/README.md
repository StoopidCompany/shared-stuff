# Runbooks

This directory contains operational runbooks — step-by-step procedures for
handling specific operational situations. Runbooks are written to be followed
under stress, often by someone who did not write them.

## When to write a runbook

Write a runbook for any procedure that:

- An on-call engineer might need to execute at 3 AM.
- Recurs predictably (weekly maintenance, quarterly key rotation, deployment cutover).
- Has a non-obvious recovery path.
- Has caused an incident before — the runbook is the first artifact of the post-mortem.

Do **not** write a runbook for:

- One-off tasks that will not recur.
- Procedures fully automated by tooling — automate further or document the tool's usage instead.
- General development workflow — that belongs in `CONTRIBUTING.md`.

## What makes a good runbook

A runbook is read by a tired, stressed person who needs to act. Optimize for
that reader, not for completeness or elegance.

- **Specific triggers.** Vague triggers ("when the system is slow") make
  runbooks get followed at the wrong time. State exact alerts, exact
  thresholds, exact symptoms.
- **One action per step.** Multi-action steps cause skipped actions under
  pressure.
- **Expected results inline.** Every action says what should happen if it
  worked. Without this, the responder cannot tell whether to continue or
  stop.
- **Mandatory rollback section.** Even if the answer is "no rollback
  possible," state that explicitly.
- **Verification before declaring done.** Resolution without verification is
  just hope.

## Format

The canonical template is in [`template.md`](./template.md). It includes:

- Status header (last reviewed date, owner, severity)
- Purpose
- When to use this runbook (the trigger)
- Prerequisites (access, tools, context)
- Diagnosis (confirm before acting)
- Resolution (numbered steps, expected results)
- Verification
- Rollback
- Escalation
- Post-incident
- References

## Naming

Use a short, action-oriented filename:

```text
restore-from-backup.md
rotate-database-credentials.md
recover-from-failed-deploy.md
handle-elevated-error-rate.md
```

Avoid alert names as filenames — alert names change. Describe what the
runbook _does_.

## Maintenance

Runbooks rot faster than any other documentation because the systems they
operate on change. To stay useful:

- **Review on use.** Anyone who runs a runbook updates the
  `Last reviewed` date and corrects anything that was wrong, missing, or
  unclear. This is non-negotiable. A runbook that worked in March may not
  work in November.
- **Quarterly audit.** Owner reviews any runbook not touched in 90 days.
  Either re-validate, update, or archive.
- **Archive, don't delete.** When a runbook is no longer needed (system
  retired, automated away), move it to an `archived/` subdirectory rather
  than deleting. The historical record is useful.

## Creating a new runbook

Use the Makefile target from the repository root:

```sh
make runbook TITLE="Restore from backup"
```

This copies [`template.md`](./template.md) and fills in the title, date, and
slug. It does not commit the file — review and edit before committing.

## Index

<!-- Update this section when adding or changing runbooks. Group by category -->
<!-- when the list grows past ~10 entries. -->

- _(no runbooks yet)_
