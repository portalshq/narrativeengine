## Representation downloads and presigned URLs

Lore can hydrate arbitrary committed bytes with `lore file write`. NAP exposes
the same binary-safe primitive through `VcsBackend::read_file_bytes_at_ref` and
can create a temporary public URL for a direct representation:

```bash
nap presign nap://toystory/character/woody reference_image \
  --branch main \
  --ttl-seconds 900
```

The representation must have a repository-relative `uri`, be committed at the
selected revision, and have been pushed to the Lore server. Branches are pinned
to a commit before NAP resolves the manifest and content address. External
URLs, linked repositories, absolute paths, path traversal, URI fragments, and
unversioned working-tree files are not supported.

The returned URL is a bearer capability: anyone who has it can download the
immutable bytes until it expires. Do not place it in logs, analytics, exception
messages, source control, or long-lived storage.

### Configuration

For NAP-managed local Lore installations created after this feature, NAP writes
a unique presign HMAC key into the owner-only Lore config. Existing configs are
not modified automatically. Add a unique 32-byte hex key and restart Lore:

```toml
[server.http]
presigned_url_hmac_key = "<64 hex characters>"
```

Generate a key with `openssl rand -hex 32`. The HTTP origin defaults to
`http://127.0.0.1:41339` for local Lore. For an explicitly configured
self-hosted deployment, pass `--http-url` or set `NAP_LORE_HTTP_URL`.

Authenticated servers also require a repository-scoped bearer token. Set
`NAP_LORE_HTTP_TOKEN` (preferred) or `NAP_LORE_GRPC_TOKEN`, or point
`--token-env` at another environment variable. Literal token command-line
arguments are intentionally unsupported because process listings can expose
them.

> **Portals Cloud status:** public presign and redemption ingress is not enabled
> yet. NAP deliberately does not derive an HTTP endpoint from
> `grpcs://lore.portals.works`. Production enablement remains a separately
> reviewed deployment task.

Rust callers use `Resolver::presign_representation`. Python exposes synchronous
`presign_representation`, and TypeScript exposes asynchronous
`presignRepresentation`. All return the URL, expiry, pinned revision,
repository ID, Lore address, representation name, and format.
