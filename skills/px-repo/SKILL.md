---
name: px-repo
description: Initialize PX repositories, clone/pull repositories, and create branches at the repository level via px-mcp-server (px_init, px_pull, px_branch). The px CLI is not available for agentic use — not for creating or revising individual entities; see px-resolve and px-update for those.
metadata:
  author: portals
  version: "0.8.25"
---

# PX Skill: Repository Management

A repository is the top-level container that holds entities (characters, locations, assets, etc.) and their PX/Lore version history.

## When to Apply

Reference these guidelines when:

- Initializing a new PX repository
- Cloning or pulling an existing repository
- Creating a new branch at the repository level
For creating or resolving individual entities, use `px-resolve`. For revising entity content and persisting iterations, use `px-update`. For questions about `px` CLI syntax from humans, use `px-cli-reference` (read-only; never execute CLI commands).

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

`repository.yaml` is the repository's manifest and the sole source of truth for project-wide context. It owns durable global visual or narrative style, reusable asset conventions, global exclusions, and canonical references through its `properties`, `representations`, and `references`.

Before creating an entity or generating content, resolve the repository.yaml from the same target branch as the entity work (via `px_resolve` on the repository URI). Read its `properties`, `representations`, and `references`, and resolve only the referenced resources relevant to the requested work. Treat repository context as the project-wide baseline; entity-specific identity, behavior, and representation data apply as refinements. Keep entity manifests focused on identity and entity-specific facts; do not add a repository-level summary or reference for every entity.

If project and entity instructions truly conflict, pause and ask the user for direction rather than choosing one. If the repository.yaml cannot be read, warn the user and ask how to proceed; never silently generate without project context.

Update `repository.yaml` only when the user explicitly defines or approves a project-wide property or reference. If entity work reveals a potentially reusable global fact, present it as a proposed repository update and wait for user approval before writing it. Preserve existing global context when adding an approved change. Never promote inferred project-wide facts into `repository.yaml` as part of an entity update.

Unless the user explicitly requests a different provider or storage location, call `px_init` with no provider argument and preserve the configured provider and default PX directory. Never infer a local provider from an example. Never choose an isolated storage location merely to isolate a repository. Pass a provider only when the user explicitly requests a provider change, and a storage location only when the user explicitly names one.

**No tagging.** Lore VCS has no native tag support — branches are the only mechanism for human-readable names on a revision point. Do not append tags to URIs.

Checking the current workspace is not required: PX usually stores all repositories in a centralized directory unless configured otherwise.



# px_init
Initialize a repository repository and/or configure the backend provider


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| provider | string | No |  | Provider type: local, portals-cloud, or remote |
| remote | string | No |  | Remote URL to add as origin after init |
| remote\_url | string | No |  | Remote URL (required for remote provider) |
| repository | string | No |  | Repository name. If provided, initializes a new repository repository |
| reset | boolean | No | false | Reset the provider configuration file |
| workspace\_id | string | No |  | Workspace ID (for remote provider) |




# px_branch
Create or list branches


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| name | string | No |  | Branch name to create. Omit to list all branches |
| repository | string | Yes |  | Repository name |




# px_pull
Clone or pull a repository from a remote


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| url\_or\_name | string | Yes |  | URL (clone) or repository name (pull existing) |




# px_list
List repositories or entities within a repository


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| entity\_type | string | No |  | Entity type to list (if repository is specified) |
| repository | string | No |  | Repository name. Omit to list all repositories |




# px_switch
Switch to a branch


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| name | string | Yes |  | Branch name to switch to |
| repository | string | Yes |  | Repository name |


