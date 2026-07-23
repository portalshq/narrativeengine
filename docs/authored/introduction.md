**NAP is a protocol that makes narrative resources addressable, resolvable, and interoperable across tools, storage systems, formats, and AI workflows.**

Characters, locations, scenes, props, and entire fictional repositories — NAP gives each one a stable URI, a human-and-machine-readable manifest, a content-addressed history, and a resolver that connects them all.

In the same way that IPFS content-addressed files and OCI container-addressed images, NAP is **narrative-addressed** — a universal namespace for the building blocks of stories.

---

## Why NAP?

Today, narrative assets live in silos:
- Worldbuilding docs in Notion or Google Docs
- Character sheets in spreadsheets
- Concept art in Dropbox or S3
- Scene breakdowns in Final Draft or Fade In
- AI prompts scattered across chat logs
- 3D assets on Sketchfab or Polycam

None of these tools talk to each other. NAP unifies them under a single addressing and resolution layer.

```text
nap://starwars/character/lukeskywalker
nap://starwars/location/tatooine
nap://starwars/scene/cantina
nap://toystory/prop/andy-hat
```

---

## Core Primitives

NAP is built on four primitives:

### 1. URI — Identity

A `nap://` URI identifies any narrative resource. Version, branch, and tag are **orthogonal selectors** passed alongside the URI — never encoded in the path (mirrors Git, OCI, and package managers).

```text
nap://starwars/character/lukeskywalker#references.appears_in
────┬── ───┬──── ────┬──── ──────┬────── ─────────────┬───────────
 scheme repository  entity_type entity_id          fragment (query)
```

### 2. Manifest — Current State

A YAML manifest is the durable representation of a narrative resource. It is simultaneously:

- **Human-editable** — readable by worldbuilders
- **Machine-editable** — structured, schema-validated
- **Agent-readable** — subtree-queryable for AI workflows
- **Portable** — no runtime dependency, just a file
- **Signable** — hash the content, sign the hash (Ed25519 in v0+)
- **Versionable** — the manifest *is* what gets committed

```yaml
id: "nap://starwars/character/lukeskywalker"
name: "Luke Skywalker"
entity_type: character
version: 17
properties:
  homeworld: "nap://starwars/location/tatooine"
  species: human
representations:
  reference_image:
    hash: "sha256:e3b0c44..."
    format: png
provenance:
  model: "midjourney-v6"
  prompt_hash: "sha256:abc123..."
head: "a72c9f3b..."
```

### 3. Commit — History

Commits are content-addressed (BLAKE3) snapshots with patch metadata. The manifest stores only `head` — a pointer to the latest commit. Full history lives in the VCS, keeping manifests bounded.

### 4. Resolver — URI → Manifest

The resolver turns a `nap://` URI into a manifest (or a subtree of one). With optional selectors for branch, tag, or commit hash, it supports versioned resolution and fragment-based queries for efficient data access.

---

## Entity Types

| Type | Example URI | Description |
|---|---|---|
| `character` | `nap://starwars/character/lukeskywalker` | Persistent character with identity across scenes/episodes |
| `location` | `nap://starwars/location/tatooine` | Spatial location within a fictional repository |
| `scene` | `nap://starwars/scene/cantina` | Narrative scene — participants, timeline, events |
| `prop` | `nap://toystory/prop/andy-hat` | Physical object with materials, variants, ownership |
| `group` | `nap://toystory/group/buzz-and-woody-flying` | Mixed-media groups |
| `world` | `nap://starwars/world/starwars` | The repository itself — rules, canon, top-level metadata |

---

## Repository Layout

Each repository is a Git repository on disk:

```text
starwars/                    ← repository root (Git repo)
├── .nap/
│   └── config.yaml          ← repository configuration
├── repository.yaml            ← world manifest
├── characters/
│   ├── lukeskywalker.yaml
│   └── darthvader.yaml
├── locations/
│   └── tatooine.yaml
├── scenes/
│   └── cantina.yaml
└── props/
```
