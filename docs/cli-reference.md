# CLI Reference

The `registry-notary` binary starts the server and provides local operator
commands.

## Status

The commands in this file are implemented by `registry-notary-bin`.

## Global Options

```text
--config <PATH>
--env-file <PATH>
--env-file-override
```

`--config` can also be supplied through `REGISTRY_NOTARY_CONFIG`.
`--env-file` can also be supplied through `REGISTRY_NOTARY_ENV_FILE`.

Use `--env-file` for local development. Production should normally rely on the
platform secret injection mechanism.

## Start The Server

```bash
registry-notary --config notary.yaml
```

The server loads config, resolves env-backed values, validates feature blocks,
initializes runtime dependencies, and serves the standalone API.

## `openapi`

Print the OpenAPI document:

```bash
registry-notary openapi > registry-notary.openapi.json
```

Use this to refresh API references or client generation inputs.

## `doctor`

Validate config, env-backed secrets, source auth, and credential wiring:

```bash
registry-notary doctor --config notary.yaml --env-file .env.local
```

Options:

- `--live`: fetch OAuth source tokens and run live reachability checks.
- `--subject-id`: subject id for record-level live probes. Output is redacted.
- `--subject-id-type`: override the DCI lookup field used by live probes.
- `--issue-demo-vc`: validate local VC issuing setup without printing
  credentials.
- `--show-expanded-config`: print resolved config with secret values redacted.

Run `doctor` before deployment and after config changes.

## `explain-config`

Print resolved config and required environment variables:

```bash
registry-notary explain-config --config notary.yaml
```

Use this when preparing deployment secrets or reviewing the effective config
shape.

## `init dci`

Generate a generic DCI starter skeleton:

```bash
registry-notary init dci \
  --output ./notary-dci \
  --base-url https://dci.example.gov \
  --token-url https://dci.example.gov/oauth2/client/token \
  --lookup-field SUBJECT_ID \
  --claim-id dci-record-exists \
  --claim-title "DCI record exists" \
  --demo-issuer \
  --with-env-file
```

Useful options:

- `--force`: overwrite generated files.
- `--print-secrets`: print generated local secrets. Use only for local
  development.
- `--demo-issuer`: include local demo issuer wiring and a generated issuer key.
- `--with-env-file`: create `.env.local` with generated local secrets.

## `hash-api-key`

Generate or hash an API key fingerprint:

```bash
registry-notary hash-api-key
registry-notary hash-api-key --hash-only "local-api-key"
printf '%s' "local-api-key" | registry-notary hash-api-key --stdin --hash-only
```

Store `sha256:<hex>` in the env var named by config. Do not store plaintext API
keys in YAML.

## `demo-issuer-key`

Generate a local Ed25519 issuer JWK for demo credentials:

```bash
registry-notary demo-issuer-key --kid did:web:localhost#registry-notary-demo
```

Do not use demo-generated keys for production.

## `schema`

Print a lightweight JSON schema for top-level config discovery:

```bash
registry-notary schema
```

## Done Check

CLI usage is correct when config commands are run before startup, generated
secrets are kept out of committed files, live diagnostics are used only in the
right environment, and generated OpenAPI output is treated as a build artifact
unless the repository intentionally tracks it.
