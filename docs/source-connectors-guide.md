# Source Connectors Guide

Registry Notary reads registry facts through configured source connections. The
source connector is a trust boundary: callers should receive only evidence that
a configured claim and disclosure allow.

## Status

Runtime source connectors support Registry Data API and DCI. Source auth
supports static bearer tokens and OAuth2 client credentials. Bulk source reads
are implemented for eligible Registry Data API and DCI configurations.

## Registry Data API Connector

Use `connector: registry_data_api` for HTTP sources that expose Registry Data
API-shaped lookup routes.

```yaml
evidence:
  source_connections:
    civil:
      base_url: https://civil.example.gov
      token_env: CIVIL_SOURCE_TOKEN
      max_in_flight: 8
      retry_on_5xx: true
```

Notary sends a purpose header to the source and reads bounded JSON responses.
Missing, ambiguous, oversized, malformed, or denied source responses become
Notary source errors.

## DCI Connector

Use `connector: dci` for DCI-style search APIs:

```yaml
evidence:
  source_connections:
    dci_civil:
      base_url: https://dci.example.gov
      token_env: DCI_SOURCE_TOKEN
      dci:
        search_path: /registry/sync/search
        sender_id: registry-notary
        query_type: idtype-value
        records_path: /message/search_response/0/data/reg_records
        max_results: 2
```

The connector maps DCI search responses into the same exact, not-found, and
ambiguous-source behavior used by local claim evaluation.

## Source Auth

Every source connection must configure exactly one auth mechanism:

- `token_env`, for a static bearer token;
- `source_auth.type: oauth2_client_credentials`, for OAuth2 client credentials.

OAuth2 source auth supports `form` and `json` token request formats, a scope
string, and token refresh skew.

## Local Insecure Source Escapes

Production source URLs should use HTTPS. Local and demo deployments can opt in
to:

- `allow_insecure_localhost`;
- `allow_insecure_private_network`.

These are development escape hatches. Do not use them for production source
registries.

## Concurrency And Politeness

`max_in_flight` caps process-global concurrent outbound requests per source
connection. Batch evaluation also has subject and binding concurrency caps:

- `evidence.concurrency.subjects`;
- `evidence.concurrency.bindings`.

Together these prevent one Notary process from overwhelming an upstream source.

## Bulk Modes

| Mode | Connector | Behavior |
| --- | --- | --- |
| `none` | Any | One source read per subject and binding |
| `rda_in_filter` | Registry Data API | Collapses eligible subjects into one RDA in-filter request |
| `dci_batched_search` | DCI | Collapses eligible subjects into one DCI batched search |

`rda_in_filter` requires `bulk_mode_lookup_unique: true` and
`lookup.cardinality: one`. If a collision is detected at runtime, Notary falls
back to per-subject reads.

`dci_batched_search` requires the DCI connector and uses
`dci.bulk_records_path` to parse each response entry.

## Failure Semantics

Source errors should preserve the Notary security boundary:

- callers receive bounded error responses;
- audit receives safe denial or error context;
- logs must not include raw source rows or source tokens;
- ambiguous source matches fail closed;
- retries are limited and can be disabled for non-idempotent sidecars.

Disable `retry_on_5xx` when a synchronous source execution must not be repeated.

## Diagnostics

Run config-only diagnostics:

```bash
registry-notary doctor --config notary.yaml --env-file .env.local
```

Run live source diagnostics:

```bash
registry-notary doctor --config notary.yaml --env-file .env.local --live
```

Live diagnostics validate source auth and reachability without printing raw
tokens or subject values.

## Done Check

A source connection is ready when auth is secret-backed, the URL policy matches
the environment, live diagnostics pass where appropriate, ambiguous and
not-found responses are understood, concurrency caps are set, and retry behavior
matches the upstream execution semantics.
