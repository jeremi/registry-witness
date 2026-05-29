# Getting Started

This guide runs Registry Notary locally with the demo config and evaluates a
claim. It is a local smoke test, not a production deployment guide.

## Prerequisites

- Rust toolchain compatible with the workspace.
- The sibling `../registry-platform` path crates available beside this repo.
- A source service reachable at the `base_url` configured in
  `demo/config/registry-notary.yaml`.
- Env vars for API auth, audit hashing, source auth, and issuer signing.

## 1. Prepare Local Secrets

The server fails closed when required credentials are absent. For the demo
config, set the variables it references:

```bash
export REGISTRY_NOTARY_API_KEY_HASH=sha256:...
export REGISTRY_NOTARY_BEARER_TOKEN_HASH=sha256:...
export REGISTRY_NOTARY_AUDIT_HASH_SECRET=dev-registry-notary-audit-hash-secret
export EVIDENCE_SOURCE_REGISTRY_RELAY_TOKEN=dev-source-token
export REGISTRY_NOTARY_ISSUER_JWK='{"kty":"OKP","crv":"Ed25519","d":"...","x":"...","alg":"EdDSA"}'
```

Use the CLI helpers instead of hand-building values:

```bash
cargo run -p registry-notary-bin -- hash-api-key
cargo run -p registry-notary-bin -- demo-issuer-key
```

For local development, you can also put env values in `.env.local` and pass
`--env-file .env.local`.

## 2. Validate The Config

```bash
cargo run -p registry-notary-bin -- \
  --config demo/config/registry-notary.yaml \
  doctor
```

With a dotenv file:

```bash
cargo run -p registry-notary-bin -- \
  --config demo/config/registry-notary.yaml \
  --env-file .env.local \
  doctor
```

Use `doctor --live` only when the configured source service is running and
reachable.

## 3. Start The Server

```bash
cargo run -p registry-notary-bin -- \
  --config demo/config/registry-notary.yaml
```

The demo config binds to `http://127.0.0.1:4255`.

## 4. Check Liveness And Readiness

```bash
curl -s http://127.0.0.1:4255/healthz
curl -s http://127.0.0.1:4255/ready
```

`/healthz` reports process liveness. `/ready` reports whether enabled runtime
dependencies, such as Redis-backed replay or credential status stores, are
usable.

## 5. Discover Configured Claims

```bash
curl -s \
  -H "Authorization: Bearer $REGISTRY_NOTARY_BEARER_TOKEN" \
  http://127.0.0.1:4255/claims
```

If the config uses API key auth instead of bearer auth, send:

```bash
curl -s \
  -H "X-Api-Key: $REGISTRY_NOTARY_API_KEY" \
  http://127.0.0.1:4255/claims
```

## 6. Evaluate A Claim

```bash
curl -s \
  -H "Authorization: Bearer $REGISTRY_NOTARY_BEARER_TOKEN" \
  -H "Content-Type: application/json" \
  -H "Accept: application/vnd.registry-notary.claim-result+json" \
  -H "Data-Purpose: demo" \
  http://127.0.0.1:4255/claims/evaluate \
  -d '{
    "subject": { "id": "person-1", "id_type": "national_id" },
    "claims": ["person-is-alive"],
    "purpose": "demo",
    "disclosure": "predicate",
    "format": "application/vnd.registry-notary.claim-result+json"
  }'
```

The exact claim ids, purposes, subject id types, and source expectations depend
on the config you run.

## Troubleshooting

| Symptom | Likely cause | Check |
| --- | --- | --- |
| Server refuses to start | Missing env-backed secret or invalid config | `doctor --show-expanded-config` |
| `/ready` returns 503 | Enabled dependency unavailable | Redis URL, credential status store, replay store |
| Evaluate returns auth error | Wrong auth header for configured mode | `auth.mode`, `api_keys`, `bearer_tokens`, OIDC config |
| Evaluate returns source error | Source service unavailable or ambiguous | `doctor --live`, source logs, claim source binding |
| Evaluate denies disclosure | Claim does not allow requested disclosure | claim `disclosure.allowed` |

## Done Check

The local smoke test is complete when:

- `doctor` passes;
- the server starts without missing-secret warnings;
- `/healthz` and `/ready` return success;
- `/claims` returns configured claims;
- `POST /claims/evaluate` returns a claim result or a policy/source error that
  matches the demo environment.
