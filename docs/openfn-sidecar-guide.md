# OpenFn Sidecar Guide

The OpenFn sidecar exposes a synchronous Registry Data API-shaped source
endpoint backed by a bounded pool of long-lived worker processes. Registry
Notary connects to it with the normal Registry Data API connector.

## Status

The sidecar is implemented as a source integration boundary. Notary still owns
caller auth, claim rules, disclosure, audit, credential issuance, and source
read policy.

## When To Use It

Use the sidecar when a registry integration is naturally expressed as pinned
OpenFn adaptor jobs, but the Notary should keep the evidence contract and
security model.

Do not use the sidecar to bypass source connector policy or to expose arbitrary
OpenFn job output to callers.

## Routes

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/datasets/{dataset}/{entity}` | Registry Data API-shaped lookup |
| `GET` | `/ready` | Manifest, credentials, worker, version, and smoke readiness |
| `GET` | `/healthz` | Process liveness |
| `GET` | `/metrics` | Prometheus metrics |

## Notary-To-Sidecar Contract

Registry Notary calls:

```text
GET /datasets/{dataset}/{entity}?{lookup_field}={lookup_value}&fields=a,b&limit=2
Authorization: Bearer <notary-to-sidecar-token>
Data-Purpose: <purpose>
```

The sidecar returns:

```json
{ "data": [] }
```

or:

```json
{ "data": [{ "field": "value" }] }
```

At most two records are returned so Notary can preserve exact, not-found, and
ambiguous-source behavior.

## Manifest Responsibilities

The sidecar manifest defines:

- bind address;
- bearer token fingerprint env vars;
- worker pool limits;
- OpenFn runtime and adaptor pins;
- worker command and version checks;
- source workflows;
- credential env vars;
- allowed base URLs;
- smoke lookups.

Raw bearer tokens are rejected in manifest config. Use hash env vars instead.

## Worker Boundary

The Rust sidecar owns HTTP, auth, validation, limits, readiness, and
non-disclosure. Workers receive JSON over private stdin and return JSON over
stdout. Worker failures, invalid output, oversized output, and timeouts are not
retried for the same request.

## Security Notes

- Do not expose the sidecar publicly.
- Constrain outbound network access at deployment level.
- `allowed_base_urls` validates configured credential targets, but it is not a
  JavaScript egress sandbox.
- Error formatting reports byte counts and truncation state, not captured
  credential-bearing content.
- Configure Notary source `retry_on_5xx: false` when worker execution must not
  repeat.

## Local Demo

Use the scripts in `crates/registry-notary-openfn-sidecar/scripts/` for local
smoke tests and the HTTP adaptor demo.

## Done Check

An OpenFn sidecar integration is ready when the manifest references only
secret-backed credentials, worker versions are pinned, readiness smoke checks
pass, Notary can evaluate a claim through the sidecar, ambiguous output fails
closed, metrics show bounded worker use, and logs do not include credential
payloads.
