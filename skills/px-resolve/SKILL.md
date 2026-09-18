---
name: px-resolve
description: Create PX entities, resolve PX URIs, and query entity context via px-mcp-server (px_create, px_resolve, px_query), establishing active entity continuity so later refinements automatically persist through px-update. The px CLI is not available for agentic use.
---

# PX Resolve

Use this skill to create entities, resolve PX URIs, and gather entity context for creative workflows.

## When to Apply

Reference these guidelines when:

- Creating new entities (e.g., characters, locations, items, events)
- Resolving PX URIs into manifests
- Querying subtree data for creative workflows
For revising entity content and persisting iterations, use `px-update`. For repository-level init/pull/branch, use `px-repo`. For questions about `px` CLI syntax from humans, use `px-cli-reference` (read-only; never execute CLI commands).

## MCP Server (mandatory for agents)

Agents MUST issue all PX operations exclusively through the `px-mcp-server` MCP tools (`px_<command>`). The `px` CLI is NOT available for agentic use — never shell out, never follow `px ...` shell examples.

The standard PX installer bundles the native `px-mcp-server` binary with `px`. If the MCP server is missing or broken, rerun the standard PX installer from a host shell.

The MCP server is not a daemon; agent clients start it on demand over stdio (see `docs/authored/mcp/install.md` for client configuration), and it proxies tool calls to the host PX installation.

## Available MCP Tools

Every PX command is available as an MCP tool with a `px_` prefix and dashes/spaces converted to underscores. For example:

- `px_resolve` — resolve a PX URI to its manifest or a subtree
- `px_create` — create a new entity manifest
- `px_query` — query a subtree from a manifest
- `px_set` — set a property on an entity manifest
- `px_add` — add a file representation to an entity manifest
- `px_commit` — commit changes to a repository

Per-tool parameters are documented under `docs/generated/mcp/<tool>.md` (generated from the same command definitions as the MCP server itself). Tool arguments use MCP field names (for example `branch`, `commit`, `format`, `include_blobs`); pass the branch explicitly on every call that accepts one — do not rely on whichever branch happens to be checked out. The CLI reference (`docs/generated/cli.md`, `docs/generated/commands/`) and shell examples elsewhere in these docs are for humans in host shells only and MUST NOT be used as agent instructions.


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

Scenes can own generated video clips the same way characters own reference images. A generated clip is not usually a representation of one character; it is a representation of a scene, with references back to the characters, locations, props, and style guides that shaped it. A scene clip is stored as a content-addressed representation (for example `clip-01`, identified by its BLAKE3 hash), not as a field on any single character.

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


## Repository Context (`repository.yaml`)

`repository.yaml` is the repository's world manifest and the sole source of truth for project-wide context. It owns durable global canon, visual or narrative style, reusable asset conventions, global exclusions, and canonical references through its `properties`, `representations`, and `references`.

Before creating an entity or generating content, resolve the repository's world manifest from the same target branch as the entity work (via `px_resolve` on the repository URI). Read its `properties`, `representations`, and `references`, and resolve only the referenced resources relevant to the requested work. Treat repository context as the project-wide baseline; entity-specific identity, behavior, and representation data apply as refinements. Keep entity manifests focused on identity and entity-specific facts; do not add a repository-level summary or reference for every entity.

If project and entity instructions truly conflict, pause and ask the user for direction rather than choosing one. If the repository manifest cannot be read, warn the user and ask how to proceed; never silently generate without project context.

Update `repository.yaml` only when the user explicitly defines or approves a project-wide property or reference. If entity work reveals a potentially reusable global fact, present it as a proposed repository update and wait for user approval before writing it. Preserve existing global context when adding an approved change. Never promote inferred project-wide facts into `repository.yaml` as part of an entity update.

Unless the user explicitly requests a different provider or storage location, call `px_init` with no provider argument and preserve the configured provider and default PX directory. Never infer a local provider from an example. Never choose an isolated storage location merely to isolate a repository. Pass a provider only when the user explicitly requests a provider change, and a storage location only when the user explicitly names one.

**No tagging.** Lore VCS has no native tag support — branches are the only mechanism for human-readable names on a revision point. Do not append tags to URIs.

Checking the current workspace is not required: PX usually stores all repositories in a centralized directory unless configured otherwise.


## Target Branch

The **target branch** is whichever branch the entity's accepted work is meant to land on. It is **not always `main`** — resolve it per task, in this order:

1. If the user named a branch for this work (e.g., "we're doing this on the `classic` branch"), that branch is the target.
2. Otherwise, the branch the entity was created on or first resolved from is the target.
3. Otherwise, default to `main`.

Establish the target branch at creation/first-resolve time and carry it forward for the rest of the task. Every promotion promotes to the **resolved target branch**, never a hardcoded `main`. When reporting or asking about promotion, name the target branch explicitly (e.g., "promote to `classic`") rather than saying "main" generically.


## Active Entity Continuity

Once a PX entity URI is established in a task, carry forward:

- the active URI, repository, entity type, and entity ID
- the active revision branch and the **target branch** (see `target-branch.md`)
- stable representation keys such as `character_sheet`, `face_sheet`, `portrait`, `model_sheet`, or `reference_image`
- user-approved identity constraints and negative constraints

Later turns that keep refining the same entity are continuity work. They must trigger `px-update` even when the user does not mention PX again and does not say "save" or "commit" or repeat the URI. Carry forward stable representation keys and identity constraints so attributes do not get dropped between turns.


## Entity Creation

When creating a new entity:

1. Establish the target branch (see `target-branch.md`), then resolve and apply repository context from that branch (see `repository-stewardship.md`).
2. Create the entity on the target branch via `px_create` (`entity_type`, `entity_id`, `repository`, `name`; default to `main` if no branch was specified).
3. Report the exact URI.
4. Establish active task context: URI, repository, entity type, entity ID, target branch, default revision branch, and repository context (see `continuity.md`).
5. Create or switch to the revision branch via `px_branch` / `px_switch`:
   ```text
   revision-<entity-type>-<entity-id>
   ```
6. If the creation turn also generates a visual, text, audio, or other representation, immediately use `px-update` to commit that first accepted revision on the revision branch.

## Generation Context

Before generating from an entity:

1. Resolve the repository world manifest from the target branch and gather relevant global properties, representations, and references.
2. Resolve the entity explicitly from the relevant branch via `px_resolve` (`uri`, plus `branch`): the target branch for canonical state, the revision branch for iterative work.
3. Gather properties that affect identity, narrative role, style, behavior, continuity, and exclusions.
4. Gather relevant entity `representations` and `references` (use `px_query` with `uri` and `path` for subtrees).
5. Treat project and entity image/video/audio representations as source-of-truth for observable appearance or sound. Text properties support and constrain them.
6. Inspect flexible negative-constraint keys such as `negative_constraints`, `exclusions`, `avoid`, `forbidden`, or project-specific equivalents at both scopes.
7. Keep multi-entity context separated so attributes do not bleed between entities.

## Branch Semantics

Resolve from the target branch for canonical state. Resolve from `revision-<entity-type>-<entity-id>` for iterative work. Pass the `branch` argument explicitly on every `px_resolve` / `px_query` call. Do not rely on whichever branch happens to be checked out. Do not store VCS branch-head data in manifests. Branch heads and commit history belong to PX/Lore version control.



# px_create
Create a new entity manifest


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| author | string | No | px | Author identifier |
| entity\_id | string | Yes |  | Entity ID (slug). e.g., "woody" |
| entity\_type | string | Yes |  | Entity type (any non-empty string, e.g. character, location, custom-type) |
| name | string | Yes |  | Human-readable name |
| repository | string | Yes |  | Repository name |




# px_resolve
Resolve a PX URI to its manifest or a subtree


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| branch | string | No |  | Resolve at a specific branch |
| commit | string | No |  | Resolve at a specific commit hash |
| format | string | No | yaml | Output format: yaml, json |
| include\_blobs | boolean | No | false | Hydrate known readable provenance artifacts such as prompts and run records |
| provenance | boolean | No | false | Include condensed per-file provenance for the manifest and direct representations |
| uri | string | Yes |  | PX URI. e.g., "px://toystory/character/woody" |




# px_query
Query a subtree from a manifest


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| format | string | No | json | Output format: yaml, json |
| path | string | Yes |  | Dot-notation path. e.g., "appearances.audienceVotes" |
| uri | string | Yes |  | PX URI |


