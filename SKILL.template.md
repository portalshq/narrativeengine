---
name: nap
description: NAP (Narrative Addressing Protocol) — identity, addressing, resolution, and attribution for narrative resources.
metadata:
  author: portals
  version: "{{version}}"
---

# NAP Coding Assistant

{{include docs/generated/cli.md}}

{{include docs/generated/options.md}}

{{include docs/generated/environment.md}}

## Repository Guidance

A NAP universe is a Git repository containing narrative entities (characters, locations, scenes, props, groups, worlds). Each entity is a YAML manifest file identified by a `nap://` URI.

### Directory Structure

```text
.universe/
    config.yaml
universe.yaml
characters/
    lukeskywalker.yaml
    leiaorgana.yaml
locations/
    tatooine.yaml
scenes/
    cantina.yaml
```

### Key Conventions

- Every piece of content must be indexed by its BLAKE3 hash.
- Use `nap set` to modify entity properties. This creates a commit automatically.
- Use `nap resolve` to read manifests. Append `#path.to.field` for subtree queries.
- The underlying Lore VCS does not support tags. Use branches for named checkpoints.
- When piped, all output switches to JSON automatically. Use `-f yaml` to override.
- Author identifiers default to `nap-cli`. Set `-a` to record provenance.

## Entity Types

| Type | Description |
|---|---|
| `character` | Persistent character with identity across scenes |
| `location` | Spatial location within a fictional universe |
| `scene` | Narrative scene — participants, timeline, events |
| `prop` | Physical object with materials, variants, ownership |
| `group` | Mixed-media groups combining entities and assets |
| `world` | The universe itself — rules, canon, top-level metadata |
