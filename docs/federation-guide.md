# Federation Guide

Registry Notary supports a first static-peer delegated evaluation slice. One
trusted peer sends a signed evaluation request to another trusted peer, which
evaluates a configured local claim and returns a signed response.

## Status

Inbound delegated evaluation is implemented. Outbound Notary connectors,
runtime composition, federated credential issuance, dynamic trust chains, and
audit checkpoint exchange are planned areas, not runtime features.

## Endpoint

```text
POST /federation/v1/evaluations
```

The route is mounted only when `federation.enabled = true`.

## Request And Response Contract

Requests are compact JWS JWTs:

- `typ = registry-notary-request+jwt`;
- `alg = EdDSA`;
- caller issuer and node id must match configured peer policy;
- purpose and profile must be allowed;
- `jti`, `iat`, `nbf`, and `exp` are required;
- request lifetime must be within configured limits.

Responses are compact JWS JWTs:

- `typ = registry-notary-response+jwt`;
- `alg = EdDSA`;
- response binds back to the request `jti`;
- response is signed by the serving Notary.

## Config Areas

Federation config includes:

- local `node_id`, `issuer`, `jwks_uri`, and `federation_api`;
- supported protocol versions;
- response signing key;
- pairwise subject hash secret;
- replay settings;
- response-shaping settings;
- emergency denylist;
- peer allow-list;
- evaluation profiles.

## Operational Requirements

- Use Redis replay storage for active-active federation deployments.
- Keep the pairwise subject hash secret dedicated to federation.
- Do not use private-network insecure peer URLs in production.
- Keep peer policies local and explicit.
- Treat manifest metadata as discovery help, not access control.

## Security Invariants

- A peer request must be signed and allowed before source reads.
- Federation must not expose raw subject ids to peer-specific logs, audit, or
  backend replay keys.
- Response shaping must be configured per profile.
- Emergency denylist changes should take effect before peer requests are served.

## Current Gaps

- No outbound connector for one Notary to call another Notary.
- No runtime composition of multiple peer responses.
- No federated credential issuance.
- No open federation or dynamic trust chains.
- No checkpoint publisher, Merkle builder, or peer audit monitor.

## Done Check

Inbound federation is ready when peer keys and policies are pinned, replay is
shared across all serving instances, the profile allows only intended claims and
purposes, signed request and replay-denial tests pass, and audit records contain
only redacted peer and subject context.
