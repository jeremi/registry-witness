# Self-Attestation Guide

Citizen self-attestation lets an authenticated citizen request configured
evidence about themself. It is stricter than machine-client evaluation and is
disabled by default.

## Status

Self-attestation is implemented for OIDC-authenticated citizens with exact
subject binding, allow-listed operations, and rate limits. Delegated,
representative, guardian, household, and consented access are not implemented.

## Security Model

Self-attestation requires:

- OIDC auth mode;
- a trusted subject-binding token claim;
- a citizen client or audience policy;
- optional or required citizen scopes, depending on `scope_policy`;
- exact subject match before any source read;
- explicit allow-lists for operations, claims, purposes, disclosures, formats,
  and credential profiles;
- rate limits for probing and issuance paths.

## Minimal Config Shape

```yaml
self_attestation:
  enabled: true
  requires_auth_mode: oidc
  subject_binding:
    token_claim: https://id.example.gov/claims/national_id
    request_field: subject_id
    id_type: national_id
    normalize: exact
    allow_sub_as_civil_id: false
  citizen_clients:
    allowed_client_ids:
      - citizen-portal
    allowed_audiences:
      - registry-notary-citizen
  token_policy:
    max_auth_age_seconds: 900
    max_access_token_lifetime_seconds: 900
    max_evaluation_age_seconds: 600
    max_credential_validity_seconds: 600
  allowed_operations:
    evaluate: true
    render: true
    issue_credential: true
    batch_evaluate: false
  allowed_purposes:
    - citizen_self_attestation
  allowed_claims:
    - person-is-alive
  allowed_formats:
    - application/vnd.registry-notary.claim-result+json
    - application/dc+sd-jwt
  allowed_disclosures:
    - predicate
  credential_profiles:
    - civil_status_sd_jwt
  rate_limits:
    mode: in_process
    invalid_token_per_client_address_per_minute: 20
    per_principal_per_minute: 10
    subject_mismatch_per_principal_per_hour: 5
    per_holder_per_hour: 10
    credential_issuance_per_principal_per_hour: 5
```

## Request Flow

1. Citizen obtains an OIDC access token from the configured issuer.
2. Citizen calls `POST /claims/evaluate` for exactly their bound subject.
3. Notary verifies token, client or audience, scope policy, token freshness,
   assurance, and exact subject binding.
4. Notary derives an internal source-read capability only for the allowed claim.
5. Notary evaluates and returns configured evidence.
6. Citizen can render or request a configured holder-bound credential.

## Denial Behavior

Public denial responses are generic where detail could aid probing. Audit
records contain bounded denial context without raw citizen identifiers or token
values.

Common denial causes:

- self-attestation disabled;
- invalid or missing token;
- untrusted client or audience;
- missing required scope;
- stale or over-long token lifetime;
- failed assurance policy;
- subject mismatch;
- disallowed claim, purpose, disclosure, format, or credential profile;
- batch operation attempted;
- rate limit exceeded.

## Browser And Wallet Origins

`allowed_wallet_origins` controls CORS for wallet-facing and self-attestation
paths. Wildcard origins are rejected by config validation.

Keep this list specific to trusted wallet or citizen portal origins. General
server CORS does not replace wallet-origin policy.

## Out Of Scope

- Parent, guardian, representative, delegated, or consented access.
- Raw registry row access.
- Multi-subject requests.
- Batch evaluation.
- Account recovery or identity proofing.
- Proving that holder DID and civil subject are the same person.

## Done Check

Self-attestation is ready when OIDC validation is configured, subject binding is
tested for match and mismatch cases, every allowed claim and purpose is
intentional, batch is denied, rate-limit behavior is verified, wallet origins
are explicit, and denial audit records remain redacted.
