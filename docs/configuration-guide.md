# Configuration Guide

Registry Notary loads one YAML config into
`StandaloneRegistryNotaryConfig`. The config model rejects unknown fields, so a
typo should fail startup rather than silently changing behavior.

## Status

This guide covers the implemented top-level config blocks and the validation
rules that most often affect startup. Use `registry-notary schema` for a
machine-readable discovery aid, and treat the Rust config model as the source of
truth for field-level details.

## Top-Level Blocks

| Block | Purpose |
| --- | --- |
| `server` | Bind address and CORS settings |
| `evidence` | Claims, credential profiles, source connections, API metadata, and concurrency |
| `auth` | API key, bearer token, or OIDC caller auth |
| `audit` | Redacted audit sink and HMAC hash secret |
| `replay` | In-memory or Redis replay store |
| `credential_status` | Optional credential lifecycle status store |
| `self_attestation` | Optional citizen self-attestation policy |
| `oid4vci` | Optional wallet-facing credential facade |
| `federation` | Optional static-peer delegated evaluation |

## Minimal Working Shape

```yaml
server:
  bind: 127.0.0.1:4325

auth:
  mode: api_key
  bearer_tokens:
    - id: local
      hash_env: REGISTRY_NOTARY_BEARER_TOKEN_HASH

audit:
  sink: stdout
  hash_secret_env: REGISTRY_NOTARY_AUDIT_HASH_SECRET

replay:
  storage: in_memory

evidence:
  enabled: true
  service_id: registry-notary
  api_version: "2026-05"
  claims: []
  credential_profiles: {}
  source_connections: {}
```

Production configs normally add source connections, claim definitions,
credential profiles, Redis-backed replay, and explicit network controls.

## Evidence Contract

The `evidence` block defines the Notary contract:

```yaml
evidence:
  enabled: true
  service_id: registry-notary
  api_version: "2026-05"
  api_base_url: /
  claims_url: /claims
  formats_url: /formats
  inline_batch_limit: 100
  concurrency:
    subjects: 16
    bindings: 8
  claims: []
  credential_profiles: {}
  source_connections: {}
```

`concurrency.subjects` caps concurrent subjects inside batch evaluation.
`concurrency.bindings` caps concurrent source bindings. Set both to `1` when
you need a strictly sequential fallback during investigation.

## Caller Auth

Static API key and bearer-token auth use stored fingerprints:

```yaml
auth:
  mode: api_key
  api_keys:
    - id: service-a
      hash_env: REGISTRY_NOTARY_API_KEY_HASH
  bearer_tokens:
    - id: service-b
      hash_env: REGISTRY_NOTARY_BEARER_TOKEN_HASH
```

OIDC mode verifies bearer JWTs against the configured issuer and JWKS:

```yaml
auth:
  mode: oidc
  oidc:
    issuer: https://id.example.gov
    jwks_uri: https://id.example.gov/oauth2/jwks
    audiences:
      - registry-notary
```

Citizen self-attestation requires OIDC mode.

## Source Connections

Each source connection under `evidence.source_connections` must use exactly one
auth mechanism.

Static source token:

```yaml
evidence:
  source_connections:
    civil:
      base_url: https://civil.example.gov
      token_env: CIVIL_SOURCE_TOKEN
      max_in_flight: 8
```

OAuth2 client credentials:

```yaml
evidence:
  source_connections:
    civil:
      base_url: https://civil.example.gov
      source_auth:
        type: oauth2_client_credentials
        token_url: https://civil.example.gov/oauth/token
        client_id_env: CIVIL_CLIENT_ID
        client_secret_env: CIVIL_CLIENT_SECRET
        request_format: form
        scope: registry.read
        refresh_skew_seconds: 60
```

Keep raw tokens, client secrets, private keys, and audit hash secrets outside
YAML. Reference them with env var names and inject values through the deployment
secret store.

## Validation Rules That Commonly Matter

- `evidence.enabled` must be true.
- `auth.mode` must be `api_key` or `oidc`.
- API key mode requires at least one API key or bearer token.
- OIDC mode requires a valid OIDC config.
- Every claim source binding must reference an existing source connection.
- Every source connection must use either `token_env` or `source_auth`, not
  both.
- Source connection `max_in_flight` must be at least 1.
- Credential profiles must use `application/dc+sd-jwt`.
- Holder-bound credential profiles currently support only `did:jwk`.
- Credential profiles must enumerate `allowed_claims`.
- Claim dependency ids must exist and must not form a cycle.
- Batch max subjects must be within the configured limit.
- Self-attestation, OID4VCI, federation, replay, and credential status blocks
  have cross-block validation.

## Config Discovery Commands

Print a lightweight JSON schema:

```bash
registry-notary schema
```

Print resolved config and required environment variables:

```bash
registry-notary explain-config --config notary.yaml --env-file .env.local
```

Validate config, env-backed secrets, source auth, and credential wiring:

```bash
registry-notary doctor --config notary.yaml --env-file .env.local
```

Run live source reachability checks:

```bash
registry-notary doctor --config notary.yaml --env-file .env.local --live
```

Output redacts secrets and subject values.

## Change Procedure

1. Change YAML in a branch or environment-specific config file.
2. Run `explain-config` to confirm the resolved shape and env var list.
3. Run `doctor` without `--live`.
4. Run `doctor --live` only against an environment where source calls are
   expected.
5. Start one Notary instance and check `/ready`.
6. Evaluate one known non-sensitive test subject for each changed claim.

## Done Check

A config change is ready when startup validation passes, required env vars are
documented in the deployment secret store, live checks match the rollout
environment, and every changed claim or credential profile has a targeted smoke
test.
