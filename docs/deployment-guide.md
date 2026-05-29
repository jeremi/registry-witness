# Deployment Guide

This guide covers deployable Registry Notary concerns. It does not replace
environment-specific platform runbooks.

## Status

The standalone binary is deployable as a Rust service. Production readiness
depends on the surrounding platform controls for secrets, TLS, network policy,
logging, metrics, and Redis.

## Runtime Inputs

Provide:

- YAML config path through `--config` or `REGISTRY_NOTARY_CONFIG`;
- secret-bearing env vars for auth fingerprints, audit hash secret, source
  tokens, OAuth2 credentials, issuer keys, replay Redis, and status Redis;
- TLS and network controls at the deployment edge;
- source registry connectivity;
- Redis when replay or credential status must work across processes.

## Startup

```bash
registry-notary --config /etc/registry-notary/notary.yaml
```

Use `--env-file` for local development. Production should normally use the
platform's secret injection mechanism.

## Preflight

Run config validation:

```bash
registry-notary doctor --config notary.yaml
```

Run live checks when the target sources are reachable:

```bash
registry-notary doctor --config notary.yaml --live
```

`doctor` validates config, env-backed secrets, source auth, DCI wiring, and
optional demo VC setup. Live mode performs reachability checks.

## Redis

Use Redis when:

- federation runs in more than one Notary process;
- OID4VCI nonce replay must work across processes;
- holder proof replay must work across processes;
- credential status must survive process restart or scale out.

Configure Redis separately for replay and credential status when both are used.

## Health And Readiness

- `/healthz` checks process liveness.
- `/ready` checks that enabled runtime dependencies are available.

Traffic should be sent only to ready instances.

## Audit

Configure one audit sink:

- `stdout`;
- `file` or `jsonl`;
- `syslog`.

Use a high-entropy `hash_secret_env` and keep it stable for the retention period
where audit correlation is required.

## CORS

General server CORS lives under `server.cors`. Citizen wallet-facing origins
live under `self_attestation.allowed_wallet_origins`. Do not use wildcard
wallet origins.

## Concurrency

Control inbound work and upstream politeness with:

- `evidence.concurrency.subjects`;
- `evidence.concurrency.bindings`;
- `source_connections[].max_in_flight`;
- sidecar worker limits when using OpenFn.

## Network Policy

Source connectors should reach only approved source registries. OpenFn sidecars
should be reachable only from Notary, and sidecar outbound access should be
constrained by deployment networking.

Do not use `allow_insecure_localhost` or `allow_insecure_private_network` for
production source registries.

## Active-Active Constraints

Active-active deployments need shared replay storage. In-memory replay and
credential status are suitable only for single-process or lab deployments.

Credential status also needs a shared store when relying parties can query
status after a process restart or when multiple Notary instances issue
credentials.

## Rollout Procedure

1. Run `doctor` in the release environment.
2. Run `doctor --live` if source checks are acceptable before traffic.
3. Start the service with the target config.
4. Wait for `/ready` to pass.
5. Smoke test `/claims` and one configured evaluation path.
6. Smoke test credential issuance, self-attestation, OID4VCI, or federation only
   when those features are enabled.
7. Confirm audit records and metrics are emitted without sensitive values.

## Done Check

A deployment is ready when preflight passes, readiness gates traffic, Redis is
configured for every cross-process replay or status requirement, CORS and
network policy are explicit, audit is anchored according to the sink, and at
least one end-to-end evaluation smoke test passes.
