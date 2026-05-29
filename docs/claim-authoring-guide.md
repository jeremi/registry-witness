# Claim Authoring Guide

Claims define what evidence Registry Notary can evaluate. A claim should be
written as a narrow, auditable product contract, not as a generic data access
query.

## Status

Runtime claim evaluation supports `exists`, `extract`, and `cel` rules when CEL
is enabled. `plugin` is modeled in config but returns unsupported at runtime.
CCCEV and OOTS fields are metadata only.

## Claim Checklist

Before adding a claim, define:

- the policy question it answers;
- the subject type and subject id type;
- the allowed purpose values;
- the exact source fields needed;
- the minimum disclosure profile;
- whether batch evaluation is allowed;
- whether the claim can be used for credential issuance;
- the expected not-found and ambiguous-source behavior.

## Basic Claim Shape

```yaml
evidence:
  claims:
    - id: person-is-alive
      title: Person is alive
      version: "1"
      subject_type: person
      purpose: benefits_eligibility
      inputs:
        - name: subject_id
          type: string
      source_bindings:
        civil:
          connector: registry_data_api
          connection: civil
          required_scope: civil:evidence
          dataset: civil_registry
          entity: civil_person
          lookup:
            input: subject.id
            field: national_id
            cardinality: one
          fields:
            deceased:
              field: deceased
              type: boolean
              required: true
      rule:
        type: cel
        expression: "civil.deceased == false"
      disclosure:
        default: predicate
        allowed: [predicate]
        downgrade: deny
      formats:
        - application/vnd.registry-notary.claim-result+json
```

## Source Bindings

Each source binding names:

- connector kind, currently `registry_data_api` or `dci`;
- source connection id;
- dataset and entity;
- lookup input and source lookup field;
- expected cardinality;
- fields to read from the source row.

Registry Notary fails closed when source results are missing required fields,
malformed, oversized, denied, or ambiguous.

## Rule Types

| Rule | Runtime support | Use |
| --- | --- | --- |
| `exists` | Supported | Return whether a configured source record exists |
| `extract` | Supported | Return one configured source field |
| `cel` | Supported when CEL feature is enabled | Derive a value or predicate from fields and prior claims |
| `plugin` | Not implemented | Config shape exists, runtime returns unsupported |

Prefer `exists` or `extract` when the rule is simple. Use CEL when the decision
needs explicit logic across fields or dependent claims.

## Claim Dependencies

Use `depends_on` when one claim needs another claim result:

```yaml
depends_on:
  - farmed-land-size
```

Config validation rejects unknown dependency ids and dependency cycles. Runtime
evaluation groups independent claims by dependency level and evaluates siblings
using the configured binding concurrency cap.

## Disclosure

Disclosure is part of the claim contract:

```yaml
disclosure:
  default: redacted
  allowed:
    - redacted
    - predicate
  downgrade: deny
```

Use the narrowest disclosure that satisfies the relying workflow. `downgrade:
deny` is the safest behavior because callers cannot silently receive a
different evidence shape than requested.

## Formats And Credential Profiles

On a claim, `formats` controls evidence response formats.
`credential_profiles` controls which credential profiles can use the claim:

```yaml
formats:
  - application/vnd.registry-notary.claim-result+json
credential_profiles:
  - civil_status_sd_jwt
```

The credential profile must also allow the claim. Both sides of the relationship
are validated.

## Batch Evaluation

Batch evaluation is disabled per claim unless enabled:

```yaml
operations:
  batch_evaluate:
    enabled: true
    max_subjects: 100
```

The top-level `evidence.inline_batch_limit` is the default batch limit. During
batch evaluation, Notary uses a per-batch memo so repeated source reads for the
same binding, lookup value, and purpose can be shared. Errors are not memoized.

Self-attestation callers cannot use batch evaluation.

## Bulk Reads

Source connections can use bulk modes:

| Mode | Connector | Behavior |
| --- | --- | --- |
| `none` | Any | One source read per subject and binding |
| `rda_in_filter` | Registry Data API | Collapses eligible subjects into one RDA in-filter request |
| `dci_batched_search` | DCI | Collapses eligible subjects into one DCI batched search |

Bulk modes are configured on source connections, not on claims. See
[Source connectors guide](source-connectors-guide.md).

## Metadata Fields

Claim definitions include optional `cccev` and `oots` metadata. Use them to
describe evidence semantics for cataloging and future interoperability work.
They do not implement a full CCCEV or OOTS exchange flow today.

## Authoring Procedure

1. Add or update the source connection.
2. Add the claim with the narrowest source fields possible.
3. Choose the rule type and disclosure profile.
4. Enable batch only when the upstream source and policy allow it.
5. Add the claim to credential profiles only when issuance is intended.
6. Run `registry-notary doctor`.
7. Evaluate a known positive case, known negative case, not-found case, and
   ambiguous-source case where the source can produce one.

## Done Check

A claim is ready when config validation passes, expected source-read behavior is
tested, disclosure is no wider than the relying workflow needs, audit records
are redacted, and credential issuance is tested when the claim is linked to a
credential profile.
