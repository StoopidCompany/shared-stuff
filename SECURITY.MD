# Security Policy

This document describes how security is handled in this project, how to report
vulnerabilities, and what conventions contributors must follow.

> **Template note:** Replace all `Stoopid Company`, `stoopid-commons`, and
> `<security@stoopid.email>` placeholders before publishing. Remove this note once
> instantiated.

## Reporting a Vulnerability

**Do not open a public issue for security vulnerabilities.**

Report vulnerabilities by emailing **<security@stoopid.email>**.

Include the following when possible:

- A description of the issue and its potential impact
- Steps to reproduce, or proof-of-concept code
- The affected version, commit SHA, or deployment environment
- Your suggested remediation, if any

### Response expectations

| Stage                  | Target                                 |
| ---------------------- | -------------------------------------- |
| Initial acknowledgment | Within 2 business days                 |
| Triage and severity    | Within 5 business days                 |
| Status updates         | At least every 7 days until resolution |
| Fix or mitigation      | Severity-dependent (see below)         |

### Severity and remediation targets

| Severity | Examples                                      | Target fix  |
| -------- | --------------------------------------------- | ----------- |
| Critical | RCE, auth bypass, data exfiltration           | 7 days      |
| High     | Privilege escalation, sensitive data exposure | 30 days     |
| Medium   | DoS, info disclosure with limited impact      | 90 days     |
| Low      | Hardening issues, defense-in-depth gaps       | Best effort |

### External researchers

We do not run a formal bug bounty program. We will work with external
researchers in good faith on a case-by-case basis. Coordinated disclosure is
expected; please give us reasonable time to remediate before publishing.

## Supported Versions

This project follows semantic versioning. Security fixes are applied to:

- The current major version (latest)
- The previous major version, for **6 months** after a new major release

Older versions are not supported. Operators are expected to upgrade.

## Scope

In-scope concerns for this repository:

- Application-layer vulnerabilities in the code maintained here
- Secret handling in code, CI, and deployment artifacts
- Dependency vulnerabilities introduced by this project
- Misuse or misconfiguration of public APIs and interfaces this project exposes

Out of scope (handled elsewhere):

- Infrastructure hardening (host OS, container runtime, cluster control plane)
- Identity provider configuration
- Physical and corporate security
- Vulnerabilities in unmodified upstream dependencies (report those upstream;
  we will track and update)

## Secrets Management

**No secrets are ever stored in this repository.** This includes but is not
limited to: API keys, tokens, passwords, private keys, certificates, database
credentials, and signing keys.

### What goes where

| Type                         | Location                                                                         |
| ---------------------------- | -------------------------------------------------------------------------------- |
| Production / staging secrets | External secret store (Vault, AWS Secrets Manager, 1Password, etc.)              |
| Local development defaults   | `.env.example` (committed, non-sensitive placeholders only)                      |
| Developer-specific overrides | `.env` (gitignored, never committed)                                             |
| CI / CD secrets              | GitHub Actions secrets, scoped to the minimum required workflows                 |
| Runtime secrets              | Mounted from external secret stores at runtime; never baked into build artifacts |

### Mandatory rules

1. `.env.example` MUST contain only placeholder values (e.g. `DATABASE_URL=postgres://localhost/dev`). It MUST NOT contain real credentials, even for shared dev environments.
2. Real secrets MUST be retrieved from the external secret store at runtime — never read from committed files.
3. Secret values MUST NOT appear in logs, traces, error messages, or telemetry. Sensitive fields should be redacted at the logging layer.
4. CI workflows MUST use scoped GitHub Actions secrets and MUST NOT echo secret values to logs.
5. Any commit containing a real secret requires immediate rotation of the secret and a follow-up to purge it from history.

### If a secret is committed

1. **Rotate the secret immediately.** Do not wait for cleanup. Treat any committed secret as compromised.
2. Notify <security@stoopid.email>.
3. Open an incident (see Incident Response below).
4. Coordinate with security to scrub history if required. Note that scrubbing alone is insufficient — rotation is the primary control.

## Authentication and Authorization

Code in this project is expected to follow the principle of least privilege:

- Services and processes run with the minimum permissions required
- Service accounts, IAM roles, and database users are scoped per-component, not shared
- Tokens, credentials, and session data are short-lived where possible
- Authentication of incoming and outgoing calls is the responsibility of the project; specific mechanisms (mTLS, JWT, OAuth, API keys, etc.) are documented in the project's own design documentation

## Dependency and Supply Chain Security

This repository enforces the following via CI:

- **Dependabot** — automated dependency update PRs for all supported ecosystems (npm, pip, cargo, terraform, github-actions, docker)
- **Secret scanning** — GitHub native secret scanning with push protection enabled
- **CodeQL** — static analysis for supported languages, run on every PR and on a weekly schedule

### Contributor obligations

- Do not disable, bypass, or weaken any of the above checks without security review
- Address Dependabot PRs promptly. Critical and high-severity advisories should be merged or have a documented mitigation within the timelines above
- Do not introduce dependencies from untrusted sources. Prefer well-maintained, audited packages with active security postures
- Pin dependencies to specific versions; lockfiles MUST be committed
- New container base images MUST be from approved sources and SHOULD be digest-pinned where practical

## Secure Development Practices

Contributors are expected to:

- Follow the conventions in `CONTRIBUTING.md` for code review, branching, and CI checks
- Treat all input as untrusted and validate at trust boundaries
- Use parameterized queries; never construct SQL by string concatenation
- Avoid logging raw request bodies, headers, tokens, or user-supplied data without sanitization
- Avoid introducing new cryptographic primitives. Use vetted libraries from language standard libraries or well-known, maintained packages
- Run pre-commit hooks locally; do not push code that fails them

## Incident Response

This section provides the high-level flow. Detailed procedures, when they exist, are documented separately per project.

### When to escalate

Escalate immediately if any of the following are observed or suspected:

- Confirmed or suspected unauthorized access to production systems or data
- Exposure of secrets, credentials, or PII
- Active exploitation of a vulnerability
- Ransomware, data destruction, or cryptojacking activity
- Compromise of CI/CD pipelines or signing infrastructure

### Initial response steps

1. **Notify** <security@stoopid.email> with subject prefix `[INCIDENT]`. Page on-call if the issue is active or affects production.
2. **Preserve evidence.** Do not delete logs, terminate suspicious workloads, or rotate credentials before security has captured forensic data — unless continued operation poses ongoing risk.
3. **Contain.** Isolate affected components (revoke tokens, scale to zero, network-isolate, etc.) per project conventions.
4. **Communicate.** Use the designated incident channel. Avoid speculation in public or shared channels.
5. **Document.** Maintain a timeline of actions taken, decisions made, and their rationale. This becomes the basis for the post-incident review.

### Post-incident

Every incident requires a blameless post-mortem documenting: timeline, root cause, contributing factors, remediation, and follow-up action items with owners.

## Compliance and Audit

Logs, audit trails, and evidence relevant to compliance obligations are
retained per `Stoopid Company` policy. Contributors should not alter or delete audit
data without explicit security approval.

## Questions

Security questions that are not vulnerabilities can be directed to
<security@stoopid.email>.
