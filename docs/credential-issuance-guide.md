# Credential Issuance Guide

Registry Notary issues SD-JWT VC credentials from configured credential
profiles. Issuance is based on a stored evaluation and does not grant callers
raw source registry access.

## Status

Current runtime support is intentionally narrow:

| Capability | Current support |
| --- | --- |
| Media type | `application/dc+sd-jwt` |
| JWT `typ` | `dc+sd-jwt` |
| Signing algorithm | `EdDSA` |
| Issuer key type | `OKP/Ed25519` |
| Holder binding DID method | `did:jwk` |
| Default validity | 600 seconds |

See [SD-JWT VC conformance profile](sd-jwt-vc-conformance-profile.md) for the
wire profile and non-support list.

## Configure A Credential Profile

```yaml
evidence:
  credential_profiles:
    civil_status_sd_jwt:
      format: application/dc+sd-jwt
      issuer: https://notary.example.gov
      issuer_key_env: REGISTRY_NOTARY_ISSUER_JWK
      issuer_kid: did:web:notary.example.gov#issuer
      vct: https://registry.example.gov/credentials/civil-status
      validity_seconds: 600
      allowed_claims:
        - person-is-alive
      holder_binding:
        mode: did
        proof_of_possession: required
        allowed_did_methods:
          - did:jwk
      disclosure:
        allowed:
          - predicate
```

Use `registry-notary demo-issuer-key` for a local demo key. Production keys
should be generated and stored by the deployment key-management process.

## Direct Issuance Flow

1. Evaluate a configured claim.
2. Submit `POST /credentials/issue` with the evaluation id and credential
   profile id.
3. Include holder DID and holder proof when the profile requires holder binding.
4. Notary validates profile, holder proof, replay, and evaluation freshness.
5. Notary signs and returns the SD-JWT VC.

## Holder Proof

Holder-bound profiles currently support `did:jwk`. The holder proof binds the
credential to the holder key. It does not prove the holder DID belongs to the
same person as the registry subject.

Replay protection is applied to holder proof JWTs. Use Redis replay storage
when more than one Notary process can issue credentials.

## Credential Status

Credential status is disabled by default. Enable it when relying parties need a
status URL in issued credentials:

```yaml
credential_status:
  enabled: true
  base_url: https://notary.example.gov
  storage: redis
  retention_seconds: 86400
  redis:
    url_env: REGISTRY_NOTARY_STATUS_REDIS_URL
    key_prefix: registry-notary
```

Supported states are `valid`, `suspended`, `revoked`, and derived `expired`.
Admin updates use:

```text
POST /admin/credentials/status/{credential_id}
```

The caller must have `registry_notary:admin`.

## OpenID4VCI Facade

The OpenID4VCI facade lets wallets request configured self-attestation
credentials:

- issuer metadata;
- credential offer;
- nonce;
- credential request.

The facade still runs Notary's self-attestation guard. It does not accept a
subject id from the wallet request in V1; the subject comes from the verified
citizen token.

## Unsupported Today

- `application/vc+sd-jwt` aliases.
- JSON-LD VC issuance.
- Data Integrity proofs.
- mDoc or mDL.
- CWT proof binding.
- `did:key` or `did:web` holder binding.
- Full general-purpose OpenID4VCI issuer behavior.

## Done Check

Credential issuance is ready when the profile allows only intended claims,
issuer keys are secret-backed, holder proof behavior is tested, replay storage
matches the deployment topology, credential status is tested when enabled, and
issued credentials are checked against the conformance profile.
