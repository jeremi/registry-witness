# Diagnostics Guide

Registry Notary exposes diagnostics through stable machine-readable codes while
keeping sensitive values out of public responses, audit records, logs, and
metrics.

## Status

This guide describes the implemented public and audit-visible diagnostic
contract for evaluation, issuance, credential status, self-attestation,
OID4VCI, and federation failures.

## Public Diagnostics

Evidence, discovery, admin, credential status, and federation routes return
Registry Notary Problem Details as `application/problem+json`. Public Problem
Details include:

- `type`: a stable problem URI derived from the code;
- `title`: short human-readable category;
- `status`: HTTP status code;
- `detail`: bounded human-readable detail;
- `code`: stable machine-readable code.

Clients should branch on `code`, not on `title` or `detail`.

Examples:

| Code | Typical status | Meaning |
| --- | --- | --- |
| `auth.missing_credential` | 401 | Caller did not provide an accepted credential |
| `auth.purpose_required` | 400 | Request needs a `Data-Purpose` header |
| `auth.scope_denied` | 403 | Caller lacks a required scope |
| `source.not_found` | 404 | Configured source lookup found no matching record |
| `source.ambiguous` | 409 | Source lookup returned too many matching records |
| `source.unavailable` | 503 | Upstream source failed or timed out |
| `claim.not_found` | 404 | Requested claim id is not configured |
| `claim.disclosure_not_allowed` | 403 | Requested disclosure is not allowed for the claim |
| `claim.format_not_supported` | 406 | Requested output format is not supported for the claim |
| `evaluation.not_found` | 404 | Stored evaluation is not available for render or issuance |
| `credential.holder_proof_required` | 400 | Credential issuance needs a holder proof |
| `credential.holder_proof_replay` | 409 | Holder proof JWT has already been consumed |
| `idempotency.conflict` | 409 | Idempotency key was reused with a different request |
| `credential_status.disabled` | 404 | Credential status endpoint is disabled |
| `credential_status.not_found` | 404 | Credential status record is not available |
| `self_attestation.denied` | 403 | Self-attestation policy denied the request |

OID4VCI wallet facade routes use OpenID4VCI wire errors instead of Problem
Details. Their public body contains `error` and optional `error_description`.
Internally, audit records use `oid4vci.*` error codes for these failures.

## Audit-Only Diagnostics

Audit records may include `error_code` and bounded denial context that is safer
for authorized operators than for public callers. Examples:

- self-attestation may return public `self_attestation.denied` while audit
  records include a more specific `denial_code`, such as
  `self_attestation.subject_mismatch`;
- OID4VCI public errors use OAuth-compatible wire names, while audit records
  include internal `oid4vci.*` error codes;
- source and policy errors keep the same stable code in public responses and
  audit `error_code`.

Audit records still must not contain raw subject ids, principal ids, bearer
tokens, API keys, source tokens, holder material, source rows, SD-JWT
disclosures, or unbounded raw error details.

## Redaction Boundary

Public response details are intentionally generic where a more specific message
could reveal source existence, citizen identifiers, holder material, or policy
internals. More specific operational context belongs in redacted audit, not in
public HTTP responses.

Logs and metrics should use low-cardinality route, status, outcome, profile, or
source labels. They must not include request bodies, subject ids, tokens,
correlation ids, holder proofs, raw source responses, or disclosure values.

## Client Guidance

Clients should:

- treat `code` as the stable machine-readable diagnostic;
- present generic user-facing messages for auth, source, and policy failures;
- avoid logging request bodies, credentials, holder proofs, nonces, compact
  SD-JWTs, source rows, or Problem Details `detail` by default;
- use audit access, not public responses, when an authorized operator needs
  deeper denial context.

## Done Check

A diagnostic change is ready when public responses have stable codes, audit
records contain only bounded safe context, focused tests prove sensitive values
do not leak, and docs identify which details are public versus audit-only.
