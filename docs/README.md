# <PROJECT_NAME>

> **Template note:** This is the technical specification for the project. It
> is the bridge between business need and technical reality — the document a
> new engineer reads to understand _what_ this project is and _why_ it exists,
> and the document a business stakeholder reads to understand _what_ is
> actually being built. Keep it current; a stale tech spec is worse than no
> tech spec.
>
> Replace `<PROJECT_NAME>` and all other placeholders below before publishing.
> Delete this blockquote once instantiated.

**Status:** <draft | active | deprecated>
**Last reviewed:** <YYYY-MM-DD>
**Owner:** <team or individual>

## Table of Contents

- [\<PROJECT_NAME\>](#project_name)
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

<One paragraph: what this project is, who it serves, and what it produces.
Written so a non-technical stakeholder can understand it.>

<One paragraph: the problem it solves. What was the situation before this
project existed, what was painful, and why is solving it worth doing now.>

## Goals and Non-goals

### Goals

What this project will do. Each goal should be specific enough to be testable.

- <Goal 1 — e.g. "Reduce average report generation time from 4 hours to under 5 minutes">
- <Goal 2>
- <Goal 3>

### Non-goals

What this project will explicitly **not** do. This section is as important as
the goals section — it prevents scope creep and sets expectations.

Use the `(deferred)` suffix for items that are out of scope _for now_ but may
be revisited. Items without the suffix are out of scope permanently.

- <Non-goal 1 — e.g. "Real-time report generation (deferred to v2)">
- <Non-goal 2 — e.g. "Support for legacy XYZ format">
- <Non-goal 3>

## Current State

What is actually built and working today. This section drifts faster than any
other; review it whenever a meaningful capability lands or is removed.

- <Capability 1 — what works, where it lives>
- <Capability 2>
- <Known gaps — things planned but not yet built>
- <Known issues — things built but not working as intended>

## Solution

<High-level description of the architecture. What the major components are,
how they fit together, and how data and control flow through the system.>

<Embed or link a diagram here. Diagram source files belong in `diagrams/` if
this project uses that directory.>

### Key components

- **<Component 1>** — <one-line purpose>. <Where it lives in the repo.>
- **<Component 2>** — <one-line purpose>. <Where it lives in the repo.>

### Significant design decisions

Significant decisions live in [`adrs/`](./adrs/) as ADRs. Reference them inline
where relevant rather than restating them here. Examples:

- See [ADR-0001](./adrs/0001-example.md) for <decision topic>.
- See [ADR-0002](./adrs/0002-example.md) for <decision topic>.

## Constraints and Assumptions

Things outside the team's control that shape the solution. Keep this honest —
unstated assumptions cause the worst kinds of failure.

### Technical

- <e.g. "Must run on Kubernetes 1.28+">
- <e.g. "Downstream system X has a 100 req/sec rate limit">

### Regulatory and organizational

- <e.g. "PII must remain in US-East region">
- <e.g. "Quarterly security audit required by compliance">

### Assumptions

- <e.g. "Upstream service Y will continue to provide field Z in its current format">
- <e.g. "Team has at least one engineer with Rust experience for the duration">

## Open Questions

Append-only. When a question is answered, either:

1. Move the resolution into the relevant section above and delete the question, or
2. If the resolution involves a significant design decision, capture it in an ADR and link from here.

- <Question 1 — owner, date raised>
- <Question 2 — owner, date raised>

## Glossary

Project-specific terms. Generic industry terms do not belong here. When this
list grows past ~20 entries, split it into `docs/glossary.md`.

- **<Term>** — <definition>
- **<Term>** — <definition>
