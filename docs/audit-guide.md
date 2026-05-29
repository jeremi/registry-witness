# Audit Guide

Registry Notary emits redacted, tamper-evident audit envelopes for
security-relevant allow and deny decisions.

## Status

Audit sinks are implemented for stdout, file or JSONL, and syslog. The service
chains records with hashes and HMAC-hashes sensitive identifiers before they
enter audit.

## Configure Audit

```yaml
audit:
  sink: file
  path: /var/log/registry-notary/audit.jsonl
  hash_secret_env: REGISTRY_NOTARY_AUDIT_HASH_SECRET
  max_size_bytes: 10485760
  max_files: 5
```

Supported sinks:

- `stdout`;
- `file` or `jsonl`;
- `syslog`.

## Hash Secret

The hash secret must be high entropy and deployment-specific. Keep it stable for
the period where auditors need to correlate records. Rotate it only with a
retention and continuity plan.

Never put the raw hash secret in YAML or logs. Reference it with
`hash_secret_env`.

## Chain Continuity

Each audit envelope contains:

- `prev_hash`;
- `record_hash`.

File and JSONL sinks resume from the retained tail hash on startup. Stdout and
syslog sinks need external anchoring if auditors must prove continuity across
process restarts.

For rotated files, retain trusted head and tail hashes outside the audit writer
so a reviewer can verify a retained window.

## Redaction Rules

Audit records must not contain:

- raw subject ids;
- raw principal ids;
- raw bearer tokens or API keys;
- source tokens;
- holder private keys or proofs;
- source rows;
- SD-JWT disclosures;
- unbounded raw error details.

Specific denial codes can be recorded when they are bounded and safe, such as
self-attestation denial categories.

## Review Procedure

For a review window:

1. Obtain the trusted starting hash for the retained window.
2. Verify each record's `prev_hash` matches the previous record's
   `record_hash`.
3. Verify the final tail hash against independently retained audit metadata.
4. Correlate records with HMAC-hashed identifiers only when the hash secret is
   available to authorized reviewers.

## Done Check

Audit is ready when the sink is configured, the hash secret is stable and
secret-backed, records are emitted for allow and deny paths, retained chain
verification is documented, and sample records contain no raw identifiers,
tokens, source rows, holder material, or disclosures.
