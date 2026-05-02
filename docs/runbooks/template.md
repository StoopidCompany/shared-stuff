# Runbook: <short title — what this runbook is for>

**Last reviewed:** <YYYY-MM-DD>
**Owner:** <team or individual>
**Severity:** <SEV-1 | SEV-2 | SEV-3 | informational>

## Purpose

<One or two sentences: what condition or task this runbook addresses, and who
should follow it. If a reader is not the right audience, they should know
within five seconds.>

## When to use this runbook

<The trigger. An alert firing, a customer report, a scheduled task, a
deployment step. Be specific — a runbook with a vague trigger gets followed
when it shouldn't and ignored when it should.>

- <Specific symptom, alert name, or condition>
- <Specific symptom, alert name, or condition>

## Prerequisites

What the responder needs before starting. If any of these are not true, stop
and acquire them before continuing — half-credentialed operators cause
incidents.

- **Access:** <required permissions, VPN, bastion, kubeconfig, etc.>
- **Tools:** <CLI tools, dashboard URLs, query consoles>
- **Context:** <links to relevant dashboards, on-call channel, incident channel>

## Diagnosis

How to confirm the condition matches what this runbook is written for. Do not
skip this step — running the wrong runbook makes things worse.

1. <Check 1 — what to look at, what "good" looks like, what "bad" looks like>
2. <Check 2>
3. <Check 3>

If diagnosis does not match, **stop**. Either find the correct runbook or
escalate per the [Escalation](#escalation) section.

## Resolution

Numbered steps. One action per step. Each step states the expected result so
the responder knows whether to continue or stop.

1. <Action> — _expected result:_ <what should happen>
2. <Action> — _expected result:_ <what should happen>
3. <Action> — _expected result:_ <what should happen>

### Verification

How to confirm the issue is resolved. Resolution without verification is just
hope.

- <Verification check 1>
- <Verification check 2>

## Rollback

If resolution makes things worse, how to undo it. This section is mandatory
even when "no rollback needed" — make that explicit rather than leaving it
ambiguous.

1. <Rollback action> — _expected result:_ <what should happen>

## Escalation

When this runbook is not enough.

- **If <condition>:** <who to page, channel to post in, ticket to file>
- **If unresolved after <duration>:** <next escalation step>

## Post-incident

What to do after the immediate situation is handled.

- File an incident report if this was a SEV-1 or SEV-2.
- Update this runbook if any step was wrong, missing, or unclear.
- Open a follow-up ticket for any underlying cause that this runbook only worked around.

## References

- <Link to related ADRs>
- <Link to dashboards, alerts, or upstream docs>
- <Link to past incident reports involving this runbook>
