## Core Primitives

PX is built on four primitives:

### 1. URI — Identity

A `px://` URI identifies any narrative resource. Version and branch are **orthogonal selectors** passed alongside the URI — never encoded in the path (mirrors Git, OCI, and package managers).

```text
px://toystory/character/woody#references.appears_in
────┬── ───┬──── ────┬──── ──────┬────── ─────────────┬───────────
 scheme repository  entity_type entity_id          fragment (query)
```

### 2. Manifest — Current State

A YAML manifest is the durable representation of a narrative resource. It is simultaneously:

- **Human-editable** — readable by toybox-builders
- **Machine-editable** — structured, schema-validated
- **Agent-readable** — subtree-queryable for AI workflows
- **Portable** — no runtime dependency, just a file
- **Signable** — hash the content, sign the hash (Ed25519 in v0+)
- **Versionable** — the manifest *is* what gets committed

```yaml
id: "px://toystory/character/woody"
name: "Woody"
entity_type: character
version: 17
properties:
  homeworld: "px://toystory/location/andys-room"
  toy_type: human
representations:
  reference_image:
    hash: "blake3:e3b0c44..."
    format: png
provenance:
  model: "midjourney-v6"
  prompt_hash: "blake3:abc123..."
```

### 3. Commit — History

Commits are content-addressed (BLAKE3) snapshots with patch metadata. Full history and revision identity live in the VCS, keeping manifests bounded and avoiding self-referential revision pointers.

### 4. Resolver — URI → Manifest

The resolver turns a `px://` URI into a manifest (or a subtree of one). With optional selectors for branch or commit hash, it supports versioned resolution and fragment-based queries for efficient data access.

### Scene Clips as Representations

Scenes can own generated video clips the same way characters own reference images. A generated clip is not usually a representation of one character; it is a representation of a scene, with references back to the characters, locations, props, and style guides that shaped it.

```bash
px create scene pizza-planet -u toystory -n "Pizza Planet"
px add px://toystory/scene/pizza-planet clip-01 ./pizza-planet-clip-01.mp4 --format mp4 -m "Add pizza-planet scene clip"
```

The scene manifest remains simple and durable:

```yaml
id: "px://toystory/scene/pizza-planet"
name: "Pizza Planet"
entity_type: scene
version: 3
properties:
  summary: "Woody and Buzz enter a crowded pizza-planet while searching for passage off Andy's Room."
  time_of_day: night
  mood: tense
references:
  characters:
    - "px://toystory/character/woody"
    - "px://toystory/character/buzzlightyear"
  location: "px://toystory/location/pizza-planet"
representations:
  clip-01:
    hash: "blake3:af1349b9..."
    format: mp4
    uri: "clip-01.mp4"
```

When resolved with provenance, PX returns versioned per-file provenance for the manifest and each direct representation. This keeps generation metadata attached to the committed files without requiring users to manage the underlying VCS directly.

```bash
px resolve px://toystory/scene/pizza-planet --provenance
```

```yaml
manifest:
  id: "px://toystory/scene/pizza-planet"
  name: "Pizza Planet"
  entity_type: scene
  version: 3
  representations:
    clip-01:
      hash: "blake3:af1349b9..."
      format: mp4
      uri: "clip-01.mp4"
provenance:
  revision: "a72c9f3b..."
  files:
    - role: manifest
      path: "scene/pizza-planet.yaml"
      provenance:
        px.provenance.kind: edit
        px.provenance.author: toybox-builder
    - role: representation
      name: clip-01
      path: "scene/clip-01.mp4"
      uri: "clip-01.mp4"
      hash: "blake3:af1349b9..."
      format: mp4
      provenance:
        px.provenance.kind: generation
        px.provenance.model: video-generator
        px.provenance.prompt.address: "blake3:b4d2..."
```

---

## Entity Types

| Type | Example URI | Description |
|---|---|---|
| `character` | `px://toystory/character/woody` | Persistent character with identity across scenes/episodes |
| `location` | `px://toystory/location/andys-room` | Spatial location within a fictional repository |
| `scene` | `px://toystory/scene/pizza-planet` | Narrative scene — participants, timeline, events |
| `prop` | `px://toystory/prop/andy-hat` | Physical object with materials, variants, ownership |
| `group` | `px://toystory/group/buzz-and-woody-flying` | Mixed-media groups |
| `world` | `px://toystory/world/toystory` | The repository itself — rules, canon, top-level metadata |

---

## Repository Layout

Each repository is a Git repository on disk:

```text
toystory/                    ← repository root (Git repo)
├── .px/
│   └── config.yaml          ← repository configuration
├── repository.yaml            ← world manifest
├── characters/
│   ├── woody.yaml
│   └── slinky.yaml
├── locations/
│   └── andys-room.yaml
├── scenes/
│   └── pizza-planet.yaml
└── props/
```
