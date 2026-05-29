# API Guide

This guide describes the human-facing HTTP API. The generated OpenAPI document
is available from `GET /openapi.json` and `registry-notary openapi`.

## Status

The routes listed here are implemented in the standalone server. Some routes
are mounted or useful only when their feature blocks are enabled.

## Common Headers

| Header | Use |
| --- | --- |
| `Authorization` | `Bearer <token>` for bearer or OIDC flows |
| `X-Api-Key` | API key auth when configured |
| `Data-Purpose` | Purpose for evidence and source-read workflows |
| `Idempotency-Key` | Optional for evaluation workflows |
| `Accept` | Requested response media type |
| `traceparent` | Optional distributed tracing context |

Purpose can be supplied by request body or `Data-Purpose` where the route
supports it. Do not send conflicting purpose values.

## Auth Expectations

Most API routes require the configured caller auth mode. Public operational
routes such as health and readiness are intended for deployment probes. Admin
routes require `registry_notary:admin`.

Self-attestation requests require OIDC and must pass the self-attestation guard.
Federation requests use signed request JWTs and peer policy instead of normal
machine-client JSON bodies.

## Discovery Routes

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/claims` | Lists configured claims |
| `GET` | `/claims/{claim_id}` | Returns one claim definition view |
| `GET` | `/formats` | Lists supported evidence formats |
| `GET` | `/.well-known/evidence-service` | Service and credential capability metadata |
| `GET` | `/.well-known/evidence/jwks.json` | Public issuer keys |
| `GET` | `/openapi.json` | OpenAPI document |

## Evaluate Claims

`POST /claims/evaluate`

```json
{
  "subject": { "id": "person-1", "id_type": "national_id" },
  "claims": ["person-is-alive"],
  "purpose": "benefits_eligibility",
  "disclosure": "predicate",
  "format": "application/vnd.registry-notary.claim-result+json"
}
```

The route authenticates the caller, validates purpose and disclosure, reads
configured sources, evaluates claim rules, stores the evaluation for later
rendering or credential issuance, and emits audit.

## Batch Evaluate Claims

`POST /claims/batch-evaluate`

Batch evaluation accepts multiple subjects for configured claims whose
`operations.batch_evaluate.enabled` is true. It uses bounded subject
concurrency and a per-batch source-read memo.

Self-attestation callers cannot use batch evaluation.

## Render Evidence

`POST /evidence/render`

Rendering converts a stored evaluation into a requested evidence format. The
caller must still be authorized for the operation, purpose, disclosure, and
format.

## Issue Credential

`POST /credentials/issue`

Credential issuance uses a stored evaluation and a configured credential
profile. Holder-bound profiles require holder proof material. Replay protection
is applied to holder proof JWTs.

## Credential Status

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/credentials/status/{credential_id}` | Public lifecycle status when enabled |
| `POST` | `/admin/credentials/status/{credential_id}` | Admin-only mutable status update |

Supported states are `valid`, `suspended`, `revoked`, and derived `expired`.
Admin updates require `registry_notary:admin`.

## OID4VCI Facade

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/.well-known/openid-credential-issuer` | Issuer metadata |
| `GET` | `/oid4vci/credential-offer` | Credential offer object or URL |
| `POST` | `/oid4vci/nonce` | Wallet proof nonce |
| `POST` | `/oid4vci/credential` | Credential request and response |

This is a narrow wallet facade for configured self-attestation credentials. It
is not a full general-purpose OpenID4VCI issuer.

## Federation

`POST /federation/v1/evaluations`

The request body is a compact JWS request JWT. The response is a compact JWS
response JWT. The route is mounted only when federation is enabled.

## Health, Readiness, Metrics, Admin

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/healthz` | Process liveness |
| `GET` | `/ready` | Fails closed when configured dependencies are unavailable |
| `GET` | `/metrics` | Prometheus text metrics |
| `POST` | `/admin/reload` | Authenticated admin no-op in standalone mode |

## Error Shape

Errors use Problem Details-style responses. Public errors are intentionally
generic where details could reveal source existence, citizen identifiers, holder
material, or policy internals. More specific denial context belongs in redacted
audit.

## Integration Procedure

1. Read `/.well-known/evidence-service` and `/claims` for the deployed
   contract.
2. Choose the least-privileged auth mode and purpose allowed for the workflow.
3. Send one evaluation request for a known test subject.
4. Add rendering or credential issuance only when the workflow needs it.
5. Handle Problem Details responses without logging secrets, subject ids, holder
   proof material, or raw source errors.

## Done Check

An API integration is ready when it uses the right auth mode, sends exactly one
purpose value, handles policy denials and source failures distinctly at the
workflow level, avoids logging sensitive request or response fields, and has a
smoke test for each route it depends on.
