# Security Model

Registry Notary is designed to fail closed and return minimized evidence from
configured registry sources. Operators and integrators must preserve the trust
boundaries in this document.

## Status

This guide summarizes implemented security controls and explicit assumptions.
It is not a deployment-specific threat model.

## Trust Boundaries

| Boundary | Control |
| --- | --- |
| Caller to Notary | API key, bearer token, or OIDC auth |
| Citizen to Notary | OIDC plus subject binding and self-attestation allow-lists |
| Notary to source | Source token or OAuth2 client credentials |
| Notary to wallet | Holder proof, nonce, short-lived SD-JWT VC |
| Peer Notary to Notary | Signed static-peer federation JWTs |
| Notary to auditor | Redacted tamper-evident audit envelopes |

## Fail-Closed Behavior

The server refuses unsafe startup or request paths when:

- auth credentials are missing;
- audit hash secret is required but absent;
- source connection auth is invalid;
- replay or credential status Redis config is required but unavailable;
- credential profile format or holder-binding method is unsupported;
- claim dependencies are invalid;
- self-attestation, OID4VCI, or federation cross-block config is inconsistent.

## Auth Modes

`api_key` mode supports API key and bearer token fingerprints. `oidc` mode
verifies bearer JWTs against configured issuer, audience, token type, JWKS, and
scope mapping.

Citizen self-attestation requires OIDC.

## Source Read Boundary

Source reads happen only after caller auth and policy checks. Citizen
self-attestation also requires exact subject binding before source reads.

Raw source rows are not returned by default. Claims and disclosure profiles
control which derived facts can leave the service.

## Replay Protection

Replay storage protects:

- federation request JWTs;
- OID4VCI nonces;
- holder proof JWTs.

In-memory replay is single-process only. Use Redis for active-active
deployments.

## Holder Proof

Holder-bound SD-JWT VC profiles currently support `did:jwk`. The holder proof
binds the credential to the holder key. It does not prove the holder DID is the
same identifier as the registry subject.

## Audit Privacy

Audit records use HMAC hashing for sensitive identifiers and include chain
hashes. They must not include raw tokens, private keys, subject ids, holder
private material, source rows, or SD-JWT disclosures.

## Metrics Privacy

Metrics must use low-cardinality labels only. Do not add labels containing
subject ids, principal ids, request ids, correlation ids, holder material,
tokens, source rows, raw errors, or disclosure values.

## Operational Controls

Deployments should also provide:

- TLS at the deployment edge or service mesh;
- network policy from Notary to only approved source systems;
- restricted access to `/metrics`;
- secret injection that avoids raw secrets in config files;
- Redis for shared replay in active-active deployments;
- external audit anchoring when stdout, syslog, or rotated files are used.

## Unsupported Assumptions

Do not assume Registry Notary provides:

- identity proofing;
- deduplication;
- representation authority;
- consent management;
- open federation trust;
- wallet UX;
- full OpenID4VCI feature coverage.

## Done Check

A deployment preserves the security model when every trust boundary has an
explicit control, source reads are allowed only through configured claims,
secret-backed values are not printed or stored in config, replay storage matches
topology, audit and metrics remain redacted, and unsupported assumptions are
handled outside Notary.
