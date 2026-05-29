# Registry Notary Overview

Registry Notary is a standalone evidence service for registry interoperability.
It answers configured questions about registry subjects without exposing raw
source rows by default.

## Status

This overview describes implemented runtime behavior. Known partial and planned
areas are tracked in [Feature inventory](feature-inventory.md).

## What It Does

Registry Notary:

- authenticates callers;
- checks purpose, claim, disclosure, format, and source-read policy;
- reads configured source registry facts through source connectors;
- evaluates configured claim rules;
- returns minimized evidence;
- stores evaluations for rendering or credential issuance;
- issues configured SD-JWT VC credentials;
- emits redacted audit records;
- exposes discovery, health, readiness, metrics, and OpenAPI endpoints;
- accepts inbound static-peer delegated evaluation when federation is enabled.

## What It Does Not Do

Registry Notary does not replace:

- the source registry;
- the identity provider;
- a deduplication or identity matching system;
- a consent or representation-authority system;
- a workflow engine;
- a wallet;
- a trust registry;
- a general data exchange platform.

Do not assume it proves parent, guardian, household, delegated, or
representative authority. Those policies need their own product and legal
model.

## Main Roles

| Role | Responsibility |
| --- | --- |
| Source registry | Operational system of record |
| Registry Data API, DCI, or OpenFn sidecar source | Read-only source API used by Notary |
| Registry Notary | Evaluates claims, returns evidence, and issues configured credentials |
| Service portal or case system | Requests evidence for a workflow |
| Citizen portal or wallet | Requests self-attestation evidence or credentials |
| Peer Notary | Sends signed delegated evaluation requests when federation is enabled |
| Auditor | Reviews minimized decisions and audit chains |

## Runtime Flow

1. A caller requests one or more configured claims for a subject and purpose.
2. Notary authenticates the caller and checks operation policy.
3. Notary verifies the requested disclosure and output format.
4. Notary reads only the configured source fields.
5. Notary evaluates the configured rule and dependencies.
6. Notary returns the allowed evidence shape.
7. Notary stores the evaluation when later rendering or issuance needs it.
8. Notary writes a redacted audit event.

## Product Invariants

- No route should perform a source read before authentication and policy checks.
- Raw source rows should not leave the service unless a configured claim and
  disclosure explicitly allow a derived value.
- Self-attestation must bind the requested subject to the verified citizen
  token before source reads.
- Credentials must be issued from stored evaluations and configured profiles.
- Audit and metrics must avoid secrets, subject ids, holder material, and raw
  source rows.

## Where To Go Next

Read [Concepts](concepts.md) for the mental model, then
[Getting started](getting-started.md) for a local run. Operators should continue
with [Deployment guide](deployment-guide.md) and
[Security model](security-model.md). Integrators should use
[API guide](api-guide.md), [Claim authoring guide](claim-authoring-guide.md),
and [Source connectors guide](source-connectors-guide.md).
