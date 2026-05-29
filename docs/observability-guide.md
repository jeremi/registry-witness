# Observability Guide

Registry Notary exposes liveness, readiness, metrics, traces, logs, and audit
signals.

## Status

Health, readiness, and Prometheus metrics are implemented. The `perf/`
directory contains a lab-supported k6 performance harness.

## Health And Readiness

`GET /healthz` returns process liveness:

```json
{
  "status": "ok",
  "checks": { "total": 1, "ok": 1, "failed": 0 }
}
```

`GET /ready` returns ready only when the enabled evidence service, replay store,
and credential status store are usable. When dependencies are unavailable, it
returns `503` with opaque counters rather than dependency names or sensitive
config details.

Use readiness, not liveness, to decide whether an instance should receive
traffic.

## Metrics

`GET /metrics` returns Prometheus text metrics.

Metrics cover HTTP requests, audit outcomes, credentials, replay outcomes, and
source behavior. Labels must remain low cardinality and safe:

- route;
- method;
- outcome;
- status class;
- profile;
- source id.

Do not add subject ids, holder ids, request ids, correlation ids, source rows,
tokens, raw errors, purposes, or claim values as labels.

## Logs And Traces

The binary initializes tracing with a default filter when `RUST_LOG` is absent.
Request spans use method and matched path. They must not include raw query
strings or secret-bearing headers.

Set `RUST_LOG` through deployment config when you need more detail. Avoid
debug-level logging in environments where source responses or token-bearing
headers could appear through dependencies.

## Performance Harness

The `perf/` directory contains:

- k6 scenarios for evaluate, CEL evaluate, batch evaluate, list claims, auth
  denial, and politeness checks;
- a deterministic DCI source stub;
- scripts to generate perf secrets and capture baselines.

Credential issuance is intentionally not covered in the current k6 suite
because each request needs fresh holder proof material.

## Operational Alerts

Useful alert surfaces:

- readiness failures;
- high 5xx rate;
- source timeout or denial rates;
- replay errors;
- audit sink failures;
- credential status store failures;
- sidecar worker saturation when using OpenFn.

## Done Check

Observability is ready when probes are wired to the deployment platform,
metrics are scraped through restricted network paths, alert labels are
low-cardinality, logs and traces avoid sensitive values, audit records are
collected, and performance baselines exist for important traffic shapes.
