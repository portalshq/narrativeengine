## Design Principles

1. **Content-addressed** — Every piece of content is identified by its cryptographic hash. Manifests are immutable once committed.

2. **URI-addressed** — Every entity has a stable, portable URI. URIs are never invalidated by renames or moves.

3. **Human-readable** — YAML manifests are readable by worldbuilders and AI agents alike.

4. **Portable** — No runtime dependencies. A manifest is just a YAML file. A repository is just a Git repo.

5. **AI-native** — Subtree queries let AI agents fetch exactly the data they need. Provenance tracking records generation metadata.

6. **Schema-validated** — All manifests conform to a JSON Schema. Invalid manifests are rejected at commit time.

7. **Decentralized** — Repositories are Git repositories. They can be cloned, forked, merged, and published independently.

8. **Extensible** — New entity types, representation formats, and merge strategies can be added without breaking existing data.

---

## Status

This is a v0 prototype. APIs and formats may change.

## License

MIT
