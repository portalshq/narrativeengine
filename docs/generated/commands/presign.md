---
generated: "true"
generator: nap-docgen
version: 0.8.13
source: clap
---


# nap presign
Create a time-limited public URL for a committed representation


## Synopsis
```bash
nap presign [OPTIONS] <ENTITY_ID> <REPRESENTATION>
```


## Description
Create a time-limited public URL for a committed representation.

Pass the entity ID first and the representation name second:

```bash
nap presign 25th-chapter/character/nathan-gunn item
```

- `25th-chapter/character/nathan-gunn` identifies the repository, entity type,
  and entity ID. The `nap://` prefix is optional.
- `item` is the exact key under the entity manifest's `representations` map.
  It is not a file path or the entity's display name.

The equivalent fully qualified command is:

```bash
nap presign nap://25th-chapter/character/nathan-gunn item
```

### How the representation is located

NAP reads the entity manifest at the selected revision and looks up
`representations.item`. For example:

```yaml
representations:
  item:
    hash: blake3:<content hash>
    format: jpg
    uri: item.jpg
```

Representation URIs are relative to the entity's asset directory, matching
`nap add`. For this entity, `uri: item.jpg` resolves to
`character/nathan-gunn/item.jpg` within the repository. Keep `uri: item.jpg`;
there is no need to put the entity ID into the representation URI.

### Revision and lifetime

```bash
nap presign 25th-chapter/character/nathan-gunn item \
  --branch main \
  --ttl-seconds 900
```

Use either `--branch` or `--commit`, never both. When neither is supplied, NAP
uses the repository's configured default branch, falling back to the global
default branch. Branches are pinned to a commit before NAP reads the manifest
and content address. Lore applies its configured lifetime bounds and defaults
when `--ttl-seconds` is omitted.

The manifest and representation file must be committed at the selected
revision, and the content must have been pushed to the Lore server.
External URLs, linked repositories, absolute paths, path traversal, URI
fragments, and unversioned working-tree files are not supported.

### Output

In a terminal, the command prints the URL, expiration, and pinned revision.
When piped or redirected, it emits JSON with `url`, `expires_at`, `revision`,
`repository_id`, `address`, `representation`, and `format`.

The returned URL is a bearer capability: anyone who has it can download the
immutable bytes until it expires. Do not place it in logs, analytics, exception
messages, source control, or long-lived storage.

### Automatic configuration

NAP records the Lore HTTP origin in `provider.toml` during backend setup and
backfills older provider configurations automatically. Local Lore uses
`http://127.0.0.1:41339`; standard remote Lore uses the same host on port 41339;
TLS deployments behind port 443 use the same HTTPS origin. Portals Cloud uses
`https://lore.portals.works`. The normal command needs no additional flags:

```bash
nap presign 25th-chapter/character/nathan-gunn item
```

Authenticated requests reuse the active `nap auth login` / Lore identity.
Only unexpired repository-scoped tokens authorized for the HTTP recipient are
used. Automatic credential reuse requires HTTPS for remote servers; loopback
HTTP is supported for development. No separate HTTP token setup is needed.

Operators with custom proxy layouts can set `http_url` in `provider.toml`.
Explicit `--http-url` or `NAP_LORE_HTTP_URL` overrides take precedence.
Bearer-token environment overrides remain available for automation.

### Server setup and signing-key security

New NAP-managed local installations create a unique 32-byte signing key in
owner-only server configuration and bind to loopback. Existing managed configs
receive a missing key without replacing existing keys or other settings.
Restart an already running server after its configuration changes.

Standalone development Lore provisions a persistent owner-only `presign.key`
in its configuration directory when no signing key is supplied. Persist this
directory across restarts. Never copy that key into client configuration.

Only Lore uses the key, to sign and validate download capabilities. It is
independent of login tokens, JWT signing keys, and API-key peppers; NAP clients
never need it. Keep the key stable across restarts and private to the server.
Server logs omit signing keys and signed query tokens. Signed responses prevent
caching and referrer leakage. Development URLs require network access to the host.

Production / Portals Cloud presign is WIP. NAP derives the Cloud HTTP origin,
but deployment still needs a dedicated shared signing key, scoped HTTPS routes,
and query-token-safe logging. Without a supplied production key, presign stays
disabled. These are operator concerns, not end-user flags or secrets.

### SDK methods

All three methods take the entity ID and representation name as their first
two arguments, using the same lookup as the CLI:

- Rust: `Resolver::presign_representation(entity_id, representation, &options)`
  is asynchronous.
- Python: `presign_representation(entity_id, representation, **options)` is
  synchronous.
- TypeScript: `presignRepresentation(entityId, representation, options)` is
  asynchronous.

Python:

```python
from nap_sdk import presign_representation

result = presign_representation(
    "25th-chapter/character/nathan-gunn", "item", branch="main", ttl_seconds=900
)
```

TypeScript:

```typescript
import { presignRepresentation } from "@portalshq/nap-sdk";

const result = await presignRepresentation(
  "25th-chapter/character/nathan-gunn",
  "item",
  { branch: "main", ttlSeconds: 900 },
);
```

The SDKs return the same fields as the CLI JSON output.


## Arguments

| Name | Description | Required |
|---|---|---|
| representation | Representation name (manifest key), e.g. item. Its URI is relative to the entity's asset directory | Yes |
| uri | Entity ID, e.g. 25th-chapter/character/nathan-gunn. The nap:// prefix is optional; fragments are not supported | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --branch | Resolve at a specific branch |  |
|     --commit | Resolve at a specific commit hash |  |
|     --http-url | Explicit Lore HTTP origin, such as http://127.0.0.1:41339 |  |
|     --token-env | Environment variable containing a repository-scoped bearer token |  |
|     --ttl-seconds | Requested lifetime in seconds; Lore enforces its configured bounds |  |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help (see more with '--help') |


## Source
`crates/nap-cli/src/main.rs` — `presign` command

