# Registry Notary User Stories

## Purpose

These user stories define the product journeys Registry Notary should support.
Use them to plan tests, review feature completeness, and check whether new work
stays aligned with the product boundary.

The stories intentionally include security, privacy, audit, and operational
criteria. Those concerns are part of the user value, not separate afterthoughts.

## Status

This document is a maintained product validation artifact. Keep it current when
features move from planned to implemented, when scenarios are dropped, or when
new acceptance criteria are discovered through implementation.

Status labels:

- Supported: implemented runtime path exists.
- Lab-supported: useful for demos or pilots but not a complete production path.
- Partial: important pieces exist, but named gaps remain.
- Planned: product story is important, but runtime support is not complete.

## Story 1: Authorized Service Evaluates A Claim

Status: Supported.

As an authorized service, I want to ask Registry Notary whether a subject
satisfies a configured claim, so that I can make an eligibility or workflow
decision without directly reading registry source data.

Acceptance criteria:

- The request is authenticated before any source lookup occurs.
- The caller can request only configured claims, purposes, formats, and
  disclosures.
- Registry Notary returns only the configured claim result or disclosure.
- Raw source rows are not returned by default.
- Ambiguous source matches fail closed.
- Allow and deny decisions emit redacted audit events.

Test hooks:

- `POST /claims/evaluate`
- source connector exact, not-found, and ambiguous cases
- audit sink output

## Story 2: Authorized Service Evaluates Claims In Batch

Status: Supported.

As an authorized service, I want to evaluate the same configured claim for
multiple subjects in one request, so that operational workflows can reduce
request overhead while preserving source-read policy.

Acceptance criteria:

- Batch evaluation is allowed only for claims that enable
  `operations.batch_evaluate.enabled`.
- The batch subject count cannot exceed the configured limit.
- Source reads use bounded subject and binding concurrency.
- Repeated source reads inside the batch can be memoized without memoizing
  errors.
- Partial item failures are represented without exposing sensitive source
  details.
- Citizen self-attestation cannot use batch evaluation.

Test hooks:

- `POST /claims/batch-evaluate`
- `evidence.inline_batch_limit`
- claim `operations.batch_evaluate`
- source read counters or source stub assertions

## Story 3: Operator Authors A Claim

Status: Supported.

As an operator or implementer, I want to define claims in config, so that the
Notary can answer policy questions through a controlled source-read and
disclosure contract.

Acceptance criteria:

- Config validation rejects unknown fields, missing source connections, missing
  dependencies, dependency cycles, and invalid concurrency settings.
- Each claim names its subject type, source binding, rule, allowed formats, and
  disclosure policy.
- `exists`, `extract`, and `cel` rules work according to configuration.
- `plugin` rules are not treated as implemented behavior.
- CCCEV and OOTS fields remain metadata unless a runtime feature implements
  exchange behavior.

Test hooks:

- `registry-notary doctor`
- `registry-notary explain-config`
- positive, negative, not-found, and ambiguous source cases

## Story 4: Notary Reads A Registry Data API Source

Status: Supported.

As an integrator, I want Registry Notary to read a Registry Data API-shaped
source, so that source registries can provide evidence facts through a narrow
HTTP contract.

Acceptance criteria:

- Source auth is configured through a static token env var or OAuth2 client
  credentials, not both.
- Source URLs follow the deployment URL policy.
- The source receives the intended purpose context.
- The connector distinguishes exact match, not found, ambiguous match, source
  denial, malformed response, oversized response, and timeout.
- `max_in_flight` protects the upstream source from unbounded fan-out.

Test hooks:

- `registry_data_api` source binding
- `doctor --live`
- source stub assertions for headers and query parameters

## Story 5: Notary Reads A DCI Source

Status: Supported.

As an integrator, I want Registry Notary to read a DCI source, so that existing
DCI registry APIs can participate in evidence workflows.

Acceptance criteria:

- DCI source config defines search path, sender id, query type, records path,
  and max result handling.
- DCI responses map into exact, not-found, and ambiguous source behavior.
- OAuth2 client-credentials source auth can fetch and refresh tokens.
- DCI bulk search is used only when the configured source supports it.
- Request signing gaps are documented for demo endpoints that currently accept
  unsigned requests.

Test hooks:

- DCI source config under `evidence.source_connections`
- `doctor --live`
- OpenCRVS and OpenSPP demo configs

## Story 6: Citizen Requests Self-Attestation

Status: Supported.

As a citizen, I want to request an attestation about myself after authenticating
through a trusted identity provider, so that I can receive verified evidence
without exposing raw registry records.

Acceptance criteria:

- Self-attestation is disabled by default and requires OIDC authentication when
  enabled.
- Registry Notary verifies token, client or audience policy, scope policy,
  token freshness, assurance policy, and exact subject binding before source
  reads.
- The citizen can request only one subject at a time, and that subject must be
  bound to the authenticated token.
- Batch evaluation, arbitrary subject lookup, raw registry access, and delegated
  access are denied.
- Denied subject-binding attempts are generic to the caller, rate limited, and
  auditable without recording raw citizen identifiers.

Test hooks:

- `POST /claims/evaluate` with OIDC auth
- subject match and mismatch cases
- self-attestation rate limiter
- denial audit categories

## Story 7: Wallet Holder Receives A Holder-Bound Credential

Status: Supported.

As a wallet holder, I want Registry Notary to issue a short-lived SD-JWT VC
bound to my holder key, so that I can present verified evidence to a relying
party.

Acceptance criteria:

- Credential issuance is based on a valid, recent stored evaluation.
- The requested credential profile, claim, disclosure, and format are allowed by
  configuration.
- Holder-bound profiles validate proof of possession for the holder key before
  issuance.
- Holder proof replay is denied.
- Issued credentials follow the SD-JWT VC conformance profile.
- Invalid holder proofs, stale evaluations, and disallowed profiles are denied
  and audited without exposing holder private material.

Test hooks:

- `POST /credentials/issue`
- holder proof validation
- replay store
- [SD-JWT VC conformance profile](sd-jwt-vc-conformance-profile.md)

## Story 8: Wallet Uses The OID4VCI Facade

Status: Supported, narrow scope.

As a wallet, I want to request a configured self-attestation credential through
OID4VCI-shaped endpoints, so that wallet integration does not need to call the
direct issuance route.

Acceptance criteria:

- Issuer metadata advertises only configured credential capabilities.
- Credential offers map to configured self-attestation credential profiles.
- Nonces are one-time use.
- Credential requests validate proof JWTs before issuance.
- The subject comes from the verified citizen token, not from arbitrary wallet
  request input.
- Unsupported OpenID4VCI features are denied or omitted rather than implied.

Test hooks:

- `/.well-known/openid-credential-issuer`
- `/oid4vci/credential-offer`
- `/oid4vci/nonce`
- `/oid4vci/credential`

## Story 9: Peer Notary Requests Delegated Evaluation

Status: Partial.

As a peer Notary, I want to send a signed delegated evaluation request to a
trusted peer, so that a local workflow can receive evidence from another
registry authority without direct source access.

Acceptance criteria:

- The serving Notary mounts federation routes only when federation is enabled.
- The request is a signed compact JWS with the expected type, algorithm,
  issuer, audience, time window, profile, purpose, and `jti`.
- Peer policy is local and explicit.
- Replay of the same request `jti` is denied.
- The response is signed by the serving Notary and binds to the request `jti`.
- Subject references are pairwise hashed.

Current gaps:

- No outbound Notary connector.
- No runtime composition across multiple peer responses.
- No federated credential issuance.
- No dynamic trust chains or audit checkpoint exchange.

Test hooks:

- `POST /federation/v1/evaluations`
- federation replay store
- signed request and response fixtures

## Story 10: Operator Runs The Service Safely

Status: Supported.

As an operator, I want predictable startup, readiness, audit, metrics, and
diagnostics, so that Registry Notary can be deployed without hidden manual
steps.

Acceptance criteria:

- Missing required secrets or invalid config fail startup.
- `doctor` catches config and secret wiring errors before deployment.
- `/ready` fails when enabled dependencies are unavailable.
- `/metrics` exposes low-cardinality labels only.
- Audit records are redacted and chained.
- Active-active deployments use Redis for replay and credential status where
  required.

Test hooks:

- `registry-notary doctor`
- `/healthz`
- `/ready`
- `/metrics`
- audit chain verification

## Story 11: Developer Uses A Client SDK

Status: Supported.

As an application developer, I want a Registry Notary client SDK, so that my
application does not reimplement auth headers, purpose handling, bounded
responses, Problem Details parsing, or redaction.

Acceptance criteria:

- The client supports exactly one configured auth mode.
- Purpose handling avoids conflicting header and body values.
- Responses are bounded by route-specific limits.
- Debug output and errors redact secrets, holder material, source rows, and
  sensitive Problem Details internals.
- Feature-gated methods exist for OID4VCI and federation.
- Python and Node.js bindings use the same underlying contract rather than
  inventing separate semantics.

Test hooks:

- Rust client unit and integration tests
- Python binding tests
- Node binding tests
- redaction assertions

## Story 12: Implementer Uses An OpenFn Sidecar Source

Status: Supported.

As an implementer, I want Registry Notary to evaluate claims using a Registry
Data API-shaped sidecar backed by pinned OpenFn adaptor jobs, so that existing
registry integrations can provide evidence without becoming part of Registry
Notary itself.

Acceptance criteria:

- Registry Notary calls the sidecar through the existing Registry Data API
  connector contract.
- The sidecar maps target-service outcomes to the expected `{ "data": [...] }`
  response shape for exact match, not found, and ambiguous match.
- Registry Notary remains the attestation authority for caller auth, claim
  rules, disclosure, provenance, audit, and credential issuance.
- Sidecar readiness fails when pinned jobs, adaptors, credentials, workers, or
  smoke lookups are missing or mismatched.
- Sidecar timeouts, worker saturation, invalid output, target failures, and
  credential non-disclosure are handled explicitly.

Test hooks:

- OpenFn sidecar `/ready`
- sidecar `/datasets/{dataset}/{entity}`
- Notary evaluate through `registry_data_api`
- worker timeout and invalid-output cases

## How To Use These Stories

For each release or major feature branch:

1. Map changed code to the stories above.
2. Confirm each affected acceptance criterion has a test, documented manual
   verification, or explicit product decision.
3. Update story status when runtime support changes.
4. Add a new story when a workflow cannot be expressed by the current set.
5. Remove or archive stories only when the product workflow is intentionally
   dropped.
