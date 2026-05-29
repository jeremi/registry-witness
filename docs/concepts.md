# Concepts

This document explains the core ideas used across Registry Notary. Read it
before changing config, writing claims, or integrating an API client.

## Status

Implemented concepts in this document reflect the current runtime. Planned
concepts are named directly when they are not complete runtime features.

## Service Boundary

Registry Notary is an evidence service. It reads configured registry facts,
evaluates configured claims, and returns minimized evidence. It does not become
the system of record, identity provider, wallet, trust registry, workflow
engine, exchange layer, or source registry.

The service should be deployed where it can enforce:

- caller authentication;
- purpose and disclosure policy;
- bounded source reads;
- redacted audit;
- replay protection where credentials, wallet nonces, or federation are used.

## Subject

A subject is the person, household, organization, or other entity that evidence
is about. Requests carry a subject id and subject id type. Source bindings map
that request subject to a source lookup field.

Self-attestation is stricter than machine-client evaluation. For
self-attestation, the subject must be derived from the verified citizen token
and must exactly match the requested subject before any source read happens.

## Claim

A claim is a configured question the Notary can answer. A claim defines:

- an id, title, version, and subject type;
- optional inputs and dependencies;
- one or more source bindings;
- a rule;
- allowed operations, disclosures, formats, and credential profiles;
- optional CCCEV or OOTS metadata.

Claims are the product contract. The runtime should not expose source facts that
are not reachable through a configured claim and allowed disclosure.

## Purpose

Purpose describes why evidence is requested. It can be supplied in the request
body or through the `Data-Purpose` header where a route supports it. Clients
should avoid sending conflicting purpose values.

Purpose is part of policy and source-read context. Treat it as a required
business input, not as logging metadata.

## Disclosure

Disclosure controls how much information leaves the service. The default
posture is redacted. Claims can allow other disclosure profiles such as
predicate output, but callers cannot receive wider evidence unless the claim
allows it.

Use `downgrade: deny` when callers should fail fast rather than silently receive
a different evidence shape.

## Source Connection

A source connection is a configured outbound registry integration. Current
connector kinds are:

- `registry_data_api`;
- `dci`.

Connections authenticate with either a static bearer token env var or OAuth2
client credentials. Each source connection also has a process-global
`max_in_flight` cap so the Notary cannot overwhelm an upstream source.

## Rule

A rule turns source facts and dependent claim values into a claim result.
Supported runtime rule types are:

- `exists`, to answer whether a source record exists;
- `extract`, to return a configured source field;
- `cel`, when the CEL feature is enabled.

`plugin` exists in the config model, but runtime execution is not implemented.

## Evaluation

Evaluation is the process of authenticating the caller, checking policy, reading
sources, running rules, storing the evaluation, returning evidence, and writing
audit.

Single-subject evaluation uses `POST /claims/evaluate`. Batch evaluation uses
`POST /claims/batch-evaluate` and is allowed only for claims that explicitly
enable `operations.batch_evaluate.enabled`.

## Credential Profile

A credential profile defines how an evaluated claim can become an issued
credential. Current issuance supports short-lived SD-JWT VC credentials using:

- `application/dc+sd-jwt`;
- EdDSA over Ed25519 issuer keys;
- optional `did:jwk` holder binding.

Credential issuance does not grant raw registry access. It signs a configured
credential from a stored evaluation.

## Replay Store

Replay protection prevents reuse of one-time identifiers. It is used for
federation request JWTs, OID4VCI nonces, and holder proof JWTs.

In-memory replay is valid only for a single running process. Use Redis whenever
more than one Notary process can accept the same class of request.

## Audit

Audit records are redacted tamper-evident envelopes. Sensitive identifiers are
HMAC-hashed before they enter audit. Audit should describe decisions without
including raw tokens, source rows, subject ids, holder private material, or
SD-JWT disclosures.

## Federation

The implemented federation slice is inbound static-peer delegated evaluation. A
trusted peer sends a signed request JWT, the serving Notary evaluates one local
profile, and the serving Notary returns a signed response JWT.

Outbound Notary-to-Notary connectors, runtime composition across peers, dynamic
trust chains, federated credential issuance, and audit checkpoint exchange are
planned areas, not current runtime behavior.
