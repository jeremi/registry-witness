# Registry Notary Feature Inventory

This document is the implementation-reviewed feature inventory for Registry
Notary. It records what the project currently contains, where each feature is
documented, and what "done" means for completing the documentation set.

## Status

The inventory was reviewed against the route table, config model, CLI commands,
client method surface, OpenFn sidecar routes, tests, performance harness, and
existing specs. It should be updated whenever runtime behavior, public routes,
config fields, or generated clients change.

## Status Legend

| Status | Meaning |
| --- | --- |
| Supported | Implemented in runtime code and covered by tests or existing product docs |
| Lab-supported | Works for demos or local scenarios, but is not yet a complete production feature |
| Partial | Important pieces exist, but named product gaps remain |
| Planned | Captured in specs or scenario docs, but not implemented as a runtime feature |

## Feature Matrix

| Area | Feature | Status | Primary docs |
| --- | --- | --- | --- |
| Product model | Overview and boundaries | Supported | [Overview](overview.md), [Concepts](concepts.md) |
| Product model | Scenario catalog and support status | Lab-supported | [Scenario catalog](notary-scenario-catalog.md) |
| Evidence | Configured claim evaluation | Supported | [Claim authoring guide](claim-authoring-guide.md), [API guide](api-guide.md) |
| Evidence | Single-subject evaluate | Supported | [API guide](api-guide.md) |
| Evidence | Batch evaluate | Supported | [API guide](api-guide.md), [Claim authoring guide](claim-authoring-guide.md) |
| Evidence | Claim dependency DAG | Supported | [Claim authoring guide](claim-authoring-guide.md) |
| Evidence | Batch memoization and single-flight source reads | Supported | [Claim authoring guide](claim-authoring-guide.md), [Deployment guide](deployment-guide.md) |
| Evidence | Bounded subject and binding concurrency | Supported | [Deployment guide](deployment-guide.md), [Observability guide](observability-guide.md) |
| Evidence | CEL rules | Supported when `registry-notary-cel` is enabled | [Claim authoring guide](claim-authoring-guide.md) |
| Evidence | Extract and exists rules | Supported | [Claim authoring guide](claim-authoring-guide.md) |
| Evidence | Plugin rule shape | Partial | [Claim authoring guide](claim-authoring-guide.md) |
| Evidence | CCCEV and OOTS metadata fields | Partial | [Claim authoring guide](claim-authoring-guide.md) |
| Evidence | Evidence discovery | Supported | [API guide](api-guide.md) |
| Evidence | Evidence rendering | Supported | [API guide](api-guide.md) |
| Evidence | Disclosure policy | Supported | [Claim authoring guide](claim-authoring-guide.md), [Security model](security-model.md) |
| Credentials | Direct SD-JWT VC issuance | Supported | [Credential issuance guide](credential-issuance-guide.md), [SD-JWT VC conformance profile](sd-jwt-vc-conformance-profile.md) |
| Credentials | Holder proof validation and replay protection | Supported | [Credential issuance guide](credential-issuance-guide.md), [Security model](security-model.md) |
| Credentials | Credential status endpoint | Supported | [Credential issuance guide](credential-issuance-guide.md), [Deployment guide](deployment-guide.md) |
| Credentials | OpenID4VCI wallet facade | Supported, narrow scope | [Credential issuance guide](credential-issuance-guide.md), [OpenID4VCI wallet facade spec](openid4vci-wallet-facade-spec.md) |
| Citizen access | OIDC citizen self-attestation | Supported | [Self-attestation guide](self-attestation-guide.md), [Citizen self-attestation spec](citizen-self-attestation-spec.md) |
| Citizen access | Wallet-origin CORS | Supported | [Self-attestation guide](self-attestation-guide.md), [Deployment guide](deployment-guide.md) |
| Federation | Inbound static-peer delegated evaluation | Partial | [Federation guide](federation-guide.md), [Federated evaluation operator guide](federated-evaluation-operator-guide.md) |
| Federation | Outbound Notary connector | Planned | [Federation guide](federation-guide.md) |
| Federation | Runtime composition across peer Notaries | Planned | [Federation guide](federation-guide.md) |
| Federation | Federated credential issuance | Planned | [Federation guide](federation-guide.md) |
| Sources | Registry Data API connector | Supported | [Source connectors guide](source-connectors-guide.md) |
| Sources | DCI connector | Supported | [Source connectors guide](source-connectors-guide.md) |
| Sources | OAuth2 client-credentials source auth | Supported | [Source connectors guide](source-connectors-guide.md), [Configuration guide](configuration-guide.md) |
| Sources | Bulk source reads | Supported | [Source connectors guide](source-connectors-guide.md) |
| Sources | OpenFn sidecar source | Supported | [OpenFn sidecar guide](openfn-sidecar-guide.md) |
| Security | API key auth | Supported | [Security model](security-model.md), [Configuration guide](configuration-guide.md) |
| Security | Static bearer token auth | Supported | [Security model](security-model.md), [Configuration guide](configuration-guide.md) |
| Security | OIDC bearer JWT auth | Supported | [Security model](security-model.md), [Configuration guide](configuration-guide.md) |
| Security | Fail-closed startup and policy checks | Supported | [Security model](security-model.md), [Deployment guide](deployment-guide.md) |
| Security | Replay store, in-memory or Redis | Supported | [Security model](security-model.md), [Deployment guide](deployment-guide.md) |
| Security | Redacted tamper-evident audit | Supported | [Security model](security-model.md), [Audit guide](audit-guide.md) |
| Operations | Health and readiness endpoints | Supported | [Observability guide](observability-guide.md) |
| Operations | Prometheus metrics | Supported | [Observability guide](observability-guide.md) |
| Operations | HTTP security headers and request body limits | Supported | [Security model](security-model.md), [Deployment guide](deployment-guide.md) |
| Operations | Admin credential status updates | Supported | [Credential issuance guide](credential-issuance-guide.md), [API guide](api-guide.md) |
| Operations | Admin reload placeholder | Supported as no-op | [API guide](api-guide.md) |
| Tooling | Standalone binary startup | Supported | [Getting started](getting-started.md), [CLI reference](cli-reference.md) |
| Tooling | `openapi` command | Supported | [CLI reference](cli-reference.md) |
| Tooling | `doctor` command | Supported | [CLI reference](cli-reference.md), [Deployment guide](deployment-guide.md) |
| Tooling | `explain-config` command | Supported | [CLI reference](cli-reference.md), [Configuration guide](configuration-guide.md) |
| Tooling | `init dci` command | Supported | [CLI reference](cli-reference.md), [Source connectors guide](source-connectors-guide.md) |
| Tooling | `hash-api-key` command | Supported | [CLI reference](cli-reference.md), [Configuration guide](configuration-guide.md) |
| Tooling | `demo-issuer-key` command | Supported | [CLI reference](cli-reference.md), [Credential issuance guide](credential-issuance-guide.md) |
| Tooling | `schema` command | Supported | [CLI reference](cli-reference.md), [Configuration guide](configuration-guide.md) |
| Clients | Rust client | Supported | [Client SDK guide](client-sdk-guide.md), [Client API spec](client-api-spec.md) |
| Clients | Python binding | Supported | [Client SDK guide](client-sdk-guide.md) |
| Clients | Node.js binding | Supported | [Client SDK guide](client-sdk-guide.md) |
| Performance | k6 performance harness | Lab-supported | [Observability guide](observability-guide.md), [Performance README](../perf/README.md) |

## Main Runtime API Surface

| Area | Endpoint |
| --- | --- |
| Health | `GET /healthz` |
| Readiness | `GET /ready` |
| Metrics | `GET /metrics` |
| OpenAPI | `GET /openapi.json` |
| Evidence service metadata | `GET /.well-known/evidence-service` |
| Evidence JWKS | `GET /.well-known/evidence/jwks.json` |
| Claims list | `GET /claims` |
| Claim detail | `GET /claims/{claim_id}` |
| Formats | `GET /formats` |
| Evaluate claims | `POST /claims/evaluate` |
| Batch evaluate claims | `POST /claims/batch-evaluate` |
| Render evidence | `POST /evidence/render` |
| Issue credential | `POST /credentials/issue` |
| Credential status | `GET /credentials/status/{credential_id}` |
| Admin credential status update | `POST /admin/credentials/status/{credential_id}` |
| Admin reload placeholder | `POST /admin/reload` |
| OID4VCI metadata | `GET /.well-known/openid-credential-issuer` |
| OID4VCI credential offer | `GET /oid4vci/credential-offer` |
| OID4VCI nonce | `POST /oid4vci/nonce` |
| OID4VCI credential | `POST /oid4vci/credential` |
| Federation evaluation | `POST /federation/v1/evaluations` |

## Known Gaps And Planned Areas

These areas are explicitly not complete runtime features today:

- Outbound Notary-to-Notary connector.
- Runtime composition across multiple peer Notaries.
- Federated credential issuance.
- Open federation and dynamic trust chains.
- Audit checkpoint exchange, Merkle checkpoint publishing, and peer monitoring.
- User-presented proof verifier runtime.
- Parent, guardian, household, or representative authority policy.
- Full general-purpose OpenID4VCI issuer feature coverage.
- JSON-LD VC, Data Integrity proof, mDoc/mDL, CWT proof, `did:key`, and
  `did:web` holder-binding support.
- Product UI or wallet UX.

## Documentation Definition Of Done

The documentation set is done when all of the following are true:

- Every supported, lab-supported, partial, and planned feature in the matrix has
  a linked Markdown document in `docs/` or a named external repo path.
- Practical guides describe tasks readers need to perform, not only code
  structure.
- Each feature guide states status, scope, prerequisites, workflow, invariants,
  operational notes, failure modes, known limitations, and a done check.
- Concepts are explained once in [Concepts](concepts.md), and guides link back
  to that vocabulary instead of redefining terms differently.
- Specs remain as design records. Implemented behavior is documented in
  practical guides.
- Public routes are documented with method, path, auth expectations, purpose
  behavior, response role, and important failure modes.
- Config docs cover all top-level config blocks and the validation invariants
  that can fail startup.
- Security docs cover fail-closed behavior, auth modes, source-read boundaries,
  redaction, replay, holder proof, subject binding, audit, metrics privacy, and
  unsupported trust assumptions.
- Operator docs cover local startup, production deployment, secrets, Redis,
  TLS, CORS, readiness, metrics, audit sinks, and active-active constraints.
- Client docs cover Rust, Python, and Node usage, auth setup, purpose handling,
  bounded responses, retries, redacted errors, and feature-gated APIs.
- Known partial and planned items are named directly so readers do not mistake
  specs or modeled config fields for completed runtime behavior.
- Examples are runnable or clearly marked as shape examples.
- Markdown files have no trailing whitespace and use relative links that resolve
  inside the repository.

## Primary Repository Areas

| Path | Responsibility |
| --- | --- |
| `crates/registry-notary-core` | Shared config, domain model, DTOs, validation, and SD-JWT helpers |
| `crates/registry-notary-server` | Axum routes, auth, runtime evaluation, rendering, issuance, federation, audit, metrics, and source connectors |
| `crates/registry-notary-bin` | CLI, config loading, process startup, tracing, shutdown, and OpenAPI generation |
| `crates/registry-notary-client` | Typed Rust client and facade boundary |
| `crates/registry-notary-openfn-sidecar` | OpenFn-backed Registry Data API-shaped source sidecar |
| `bindings/python` | Python client binding |
| `bindings/node` | Node.js client binding |
| `demo/config` | Demo Registry Notary configurations |
| `perf` | k6 scenarios, Python harnesses, and performance baselines |
