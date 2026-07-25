# nap — Narrative Addressing Protocol

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

## Installation

### Installation Script

```bash
curl -fsSL https://github.com/portalshq/narrativeengine/releases/latest/download/install.sh | bash
```

The installation script installs both native binaries: `nap` and `nap-mcp-server`. The MCP server is dormant by default; agent clients start it on demand over stdio so sandboxed agents can use NAP through host-side CLI proxy calls.

### Skills Install

Install these skills to use NAP with agent workflows, including entity-aware prompts, generation templates, and the resolve/update steps that keep character and scene output consistent.

```bash
npx skills add portalshq/narrativeengine
```

---

## Quick Start

```bash
# Initialize a repository (prompts for provider on first run)
nap init starwars

# Initialize with local provider
nap init starwars --provider local

# Configure provider only (no repository)
nap init --provider local

# Initialize with remote provider
nap init --provider remote --remote-url lore://localhost:41337 --workspace-id my-workspace

# Initialize with Portals Cloud
nap init --provider portals-cloud

# Check system status
nap status

# Run diagnostics
nap doctor

# Run diagnostics with auto-repair
nap doctor --repair
```

### Create a Repository

```bash
# Initialize a new repository
nap init starwars

# See what you created
ls starwars/
# → .nap/  repository.yaml  characters/  locations/  scenes/  props/
```

### Create & Inspect Entities

```bash
# Create a character
nap create character lukeskywalker -u starwars -n "Luke Skywalker"

# Create a location
nap create location tatooine -u starwars -n "Tatooine"

# Set properties
nap set nap://starwars/character/lukeskywalker species human
nap set nap://starwars/character/lukeskywalker homeworld "nap://starwars/location/tatooine"

# Resolve a manifest
nap resolve nap://starwars/character/lukeskywalker

# Query a specific field
nap resolve nap://starwars/character/lukeskywalker#properties.species
# → human

# Query a subtree
nap query nap://starwars/character/lukeskywalker properties
```

### Version Control

```bash
# View commit history
nap history nap://starwars/character/lukeskywalker

# Create branches
nap branch starwars canon

# Sync with remote
nap sync starwars

# Publish to remote
nap publish starwars
```

### Output Formats

```bash
nap resolve nap://starwars/character/lukeskywalker -f json
nap resolve nap://starwars/character/lukeskywalker -f yaml
```


---

## Core Primitives

NAP is built on four primitives:

### 1. URI — Identity

A `nap://` URI identifies any narrative resource. Version and branch are **orthogonal selectors** passed alongside the URI — never encoded in the path (mirrors Git, OCI, and package managers).

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
    hash: "blake3:e3b0c44..."
    format: png
provenance:
  model: "midjourney-v6"
  prompt_hash: "blake3:abc123..."
```

### 3. Commit — History

Commits are content-addressed (BLAKE3) snapshots with patch metadata. Full history and revision identity live in the VCS, keeping manifests bounded and avoiding self-referential revision pointers.

### 4. Resolver — URI → Manifest

The resolver turns a `nap://` URI into a manifest (or a subtree of one). With optional selectors for branch or commit hash, it supports versioned resolution and fragment-based queries for efficient data access.

### Scene Clips as Representations

Scenes can own generated video clips the same way characters own reference images. A generated clip is not usually a representation of one character; it is a representation of a scene, with references back to the characters, locations, props, and style guides that shaped it.

```bash
nap create scene cantina -u starwars -n "Cantina"
nap add nap://starwars/scene/cantina clip-01 ./cantina-clip-01.mp4 --format mp4 -m "Add cantina scene clip"
```

The scene manifest remains simple and durable:

```yaml
id: "nap://starwars/scene/cantina"
name: "Cantina"
entity_type: scene
version: 3
properties:
  summary: "Luke and Obi-Wan enter a crowded cantina while searching for passage off Tatooine."
  time_of_day: night
  mood: tense
references:
  characters:
    - "nap://starwars/character/lukeskywalker"
    - "nap://starwars/character/obiwankenobi"
  location: "nap://starwars/location/mos-eisley-cantina"
representations:
  clip-01:
    hash: "blake3:af1349b9..."
    format: mp4
    uri: "clip-01.mp4"
```

When resolved with provenance, NAP returns versioned per-file provenance for the manifest and each direct representation. This keeps generation metadata attached to the committed files without requiring users to manage the underlying VCS directly.

```bash
nap resolve nap://starwars/scene/cantina --provenance
```

```yaml
manifest:
  id: "nap://starwars/scene/cantina"
  name: "Cantina"
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
      path: "scene/cantina.yaml"
      provenance:
        nap.provenance.kind: edit
        nap.provenance.author: worldbuilder
    - role: representation
      name: clip-01
      path: "scene/clip-01.mp4"
      uri: "clip-01.mp4"
      hash: "blake3:af1349b9..."
      format: mp4
      provenance:
        nap.provenance.kind: generation
        nap.provenance.model: video-generator
        nap.provenance.prompt.address: "blake3:b4d2..."
```

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


---

## MCP Server

The NAP installer bundles the native `nap-mcp-server` binary alongside `nap`. The MCP server is not a daemon; agent clients start it on demand over stdio, and it proxies tool calls to the host `nap` CLI.

## Connect with Codex

Codex stores MCP configuration in `~/.codex/config.toml` alongside the rest of its config. The Codex CLI, the ChatGPT desktop app, and the IDE extension all share that MCP configuration, so you only need to register `nap` once.

Add the server with the CLI:

```bash
codex mcp add nap --env NAP_DIR="$HOME/.nap" -- /bin/sh -lc 'exec nap-mcp-server'
```

If `nap-mcp-server` is not on `PATH`, use the full installed path instead, usually `~/.local/bin/nap-mcp-server` or `/usr/local/bin/nap-mcp-server`.

You can also configure it manually in `~/.codex/config.toml`:

```toml
[mcp_servers.nap]
command = "/bin/sh"
args = ["-lc", "NAP_DIR=\"$HOME/.nap\" exec nap-mcp-server"]
enabled = true
```

Project-scoped config works too for trusted projects:

```toml
[mcp_servers.nap]
command = "/bin/sh"
args = ["-lc", "NAP_DIR=\"$HOME/.nap\" exec nap-mcp-server"]
enabled = true
```

Use the same block in `.codex/config.toml` inside a trusted project if you want the server scoped to that repository.

## Other MCP Clients

Claude Desktop and other MCP clients use the same stdio pattern. Add a server entry that runs the bundled `nap-mcp-server` command on demand, and keep `NAP_DIR` pointed at your NAP workspace if you need a non-default data directory.

Example host-side launch command:

```bash
/bin/sh -lc 'NAP_DIR="$HOME/.nap" exec nap-mcp-server'
```

Use the same command/args form in any client that supports stdio MCP servers.

Inside sandboxes, use the MCP tools instead of shelling out to `nap` directly for network-backed operations. Direct `nap` CLI commands remain the right choice for humans and host-local shells.


---


# NAP CLI Reference
The `nap` command-line interface (v0.5.5) provides tools for creating, resolving, and managing narrative resources using the Narrative Addressing Protocol.


## Command Overview

| Command | Description |
|---|---|
| [\`nap add\`](docs/generated/commands/add.md) | Add a file representation to an entity manifest |
| [\`nap branch\`](docs/generated/commands/branch.md) | Create or list branches |
| [\`nap choose\`](docs/generated/commands/choose.md) | Choose backend provider |
| [\`nap commit\`](docs/generated/commands/commit.md) | Commit changes to a repository repository |
| [\`nap content-hash\`](docs/generated/commands/content-hash.md) | Compute the BLAKE3 content hash of a file |
| [\`nap create\`](docs/generated/commands/create.md) | Create a new entity manifest |
| [\`nap diff\`](docs/generated/commands/diff.md) | Show diff between two manifest files or versions |
| [\`nap doctor\`](docs/generated/commands/doctor.md) | Run diagnostics and repair |
| [\`nap head-hash\`](docs/generated/commands/head-hash.md) | Show the current HEAD commit hash |
| [\`nap history\`](docs/generated/commands/history.md) | View commit history for an entity |
| [\`nap init\`](docs/generated/commands/init.md) | Initialize a repository repository and/or configure the backend provider |
| [\`nap install\`](docs/generated/commands/install.md) | Install required dependencies |
| [\`nap list\`](docs/generated/commands/list.md) | List repositories or entities within a repository |
| [\`nap merge\`](docs/generated/commands/merge.md) | Three-way merge of JSON/YAML values |
| [\`nap publish\`](docs/generated/commands/publish.md) | Publish changes to remote |
| [\`nap pull\`](docs/generated/commands/pull.md) | Clone or pull a repository from a remote |
| [\`nap push\`](docs/generated/commands/push.md) | Push the current branch to its configured upstream remote |
| [\`nap query\`](docs/generated/commands/query.md) | Query a subtree from a manifest |
| [\`nap remote\`](docs/generated/commands/remote.md) | Manage remotes on a repository |
| [\`nap resolve\`](docs/generated/commands/resolve.md) | Resolve a NAP URI to its manifest or a subtree |
| [\`nap revert\`](docs/generated/commands/revert.md) | Revert a commit by hash (undoes all changes in that commit) |
| [\`nap schema\`](docs/generated/commands/schema.md) | Print a JSON Schema for manifest or commit types |
| [\`nap set\`](docs/generated/commands/set.md) | Set a property on an entity manifest |
| [\`nap sign\`](docs/generated/commands/sign.md) | Sign a manifest (stub for v0) |
| [\`nap status\`](docs/generated/commands/status.md) | Show system status |
| [\`nap switch\`](docs/generated/commands/switch.md) | Switch to a branch |
| [\`nap sync\`](docs/generated/commands/sync.md) | Sync with remote |
| [\`nap validate\`](docs/generated/commands/validate.md) | Validate a manifest against the NAP schema |
| [\`nap verify\`](docs/generated/commands/verify.md) | Verify a manifest signature (stub for v0) |


## Global Options

| Flag | Description | Default |
|---|---|---|
| -d, --base-dir <BASE\_DIR> | Base directory for repository repositories. Defaults to $NAP\_DIR, or ~/.nap if unset |  |
| -v, --verbose <VERBOSE> | Enable verbose debug logging |  |


## Output Formats
Most commands support `--format` (`-f`) with values `yaml` (default) or `json`.

When stdout is not a terminal, JSON is used automatically. Override with `$NAP_OUTPUT`.


## Common Examples
```bash
# Initialize a repository
nap init starwars

# Create an entity
nap create character lukeskywalker -u starwars -n "Luke Skywalker"

# Resolve a manifest
nap resolve nap://starwars/character/lukeskywalker

# Query a subtree
nap query nap://starwars/character/lukeskywalker properties

# View commit history
nap history nap://starwars/character/lukeskywalker
```



---

## HTTP Server

The NAP resolver server provides a REST API for resolution and commits.

```bash
# Start the server (defaults to port 3100, base path = current directory)
nap-server

# Custom port and base path
NAP_PORT=8080 NAP_BASE_PATH=/path/to/repositories nap-server
```

---

## Configuration

NAP core uses environment variables for configuration. All variables serve specific purposes with minimal overlap.

### Storage Configuration

| Variable | Purpose | Default | Required |
|----------|---------|---------|----------|
| `NAP_STORAGE_BACKEND` | Storage backend selection (`local` or `s3`) | `local` | No |
| `NAP_DIR` | Base directory for local storage | `~/.nap` | No (local) |
| `NAP_S3_BUCKET` | S3 bucket name | — | Yes (s3) |
| `AWS_ACCESS_KEY_ID` | AWS/R2 access key | — | Yes (s3) |
| `AWS_SECRET_ACCESS_KEY` | AWS/R2 secret key | — | Yes (s3) |
| `AWS_REGION` | AWS region | — | Yes (s3) |
| `AWS_ENDPOINT_URL_S3` | Custom S3 endpoint (R2, MinIO) | — | No (s3) |
| `AWS_ENDPOINT_URL` | Fallback S3 endpoint if `AWS_ENDPOINT_URL_S3` unset | — | No (s3) |

### Lore VCS Configuration

| Variable | Purpose | Default | Required |
|----------|---------|---------|----------|
| `NAP_LORE_URL_BASE` | Lore server URL base | `lore://localhost:8700` | No |
| `NAP_WORKSPACE_ID` | Workspace identifier for multi-tenancy | `default` | No |
| `NAPLORE_CLI` | Path to lore CLI binary | `lore` (from PATH) | No |
| `NAP_LORE_GRPC_ENDPOINT` | gRPC endpoint for branch ref sync | — | No (optional) |
| `NAP_LORE_GRPC_TOKEN` | JWT bearer token for gRPC auth | — | No (optional) |
| `NAP_LORE_GRPC_RID` | Repository ID (hex-encoded) for gRPC | — | No (optional) |
| `NAP_LORE_GRPC_INSECURE` | Skip TLS verification (`1`/`true`/`yes`) | `0` | No (optional) |

### Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `NAP_DIR` (const) | `.nap` | Metadata directory name within repositories |

**Note:** The environment variable `NAP_DIR` (storage base directory) and the constant `NAP_DIR` (metadata directory name) serve different purposes and do not overlap.

### Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/resolve/{repository}/{entity_type}/{entity_id}` | Resolve a manifest |
| `GET` | `/resolve/{repository}/{entity_type}/{entity_id}?branch=canon` | Resolve at a branch |
| `POST` | `/commit/{repository}/{entity_type}/{entity_id}` | Commit changes |
| `GET` | `/history/{repository}/{entity_type}/{entity_id}` | Get commit history |
| `GET` | `/repositories` | List all repositories |
| `GET` | `/repositories/{repository}/entities` | List entities in a repository |
| `GET` | `/health` | Health check |

Query parameters for resolution: `branch`, `commit`, `path` (subtree query).


---


# CLI Command Reference
Complete reference for all `nap` CLI commands.


| Command | Description |
|---|---|
| [\`nap add\`](docs/generated/commands/add.md) | Add a file representation to an entity manifest |
| [\`nap branch\`](docs/generated/commands/branch.md) | Create or list branches |
| [\`nap choose\`](docs/generated/commands/choose.md) | Choose backend provider |
| [\`nap commit\`](docs/generated/commands/commit.md) | Commit changes to a repository repository |
| [\`nap content-hash\`](docs/generated/commands/content-hash.md) | Compute the BLAKE3 content hash of a file |
| [\`nap create\`](docs/generated/commands/create.md) | Create a new entity manifest |
| [\`nap diff\`](docs/generated/commands/diff.md) | Show diff between two manifest files or versions |
| [\`nap doctor\`](docs/generated/commands/doctor.md) | Run diagnostics and repair |
| [\`nap head-hash\`](docs/generated/commands/head-hash.md) | Show the current HEAD commit hash |
| [\`nap history\`](docs/generated/commands/history.md) | View commit history for an entity |
| [\`nap init\`](docs/generated/commands/init.md) | Initialize a repository repository and/or configure the backend provider |
| [\`nap install\`](docs/generated/commands/install.md) | Install required dependencies |
| [\`nap list\`](docs/generated/commands/list.md) | List repositories or entities within a repository |
| [\`nap merge\`](docs/generated/commands/merge.md) | Three-way merge of JSON/YAML values |
| [\`nap publish\`](docs/generated/commands/publish.md) | Publish changes to remote |
| [\`nap pull\`](docs/generated/commands/pull.md) | Clone or pull a repository from a remote |
| [\`nap push\`](docs/generated/commands/push.md) | Push the current branch to its configured upstream remote |
| [\`nap query\`](docs/generated/commands/query.md) | Query a subtree from a manifest |
| [\`nap remote\`](docs/generated/commands/remote.md) | Manage remotes on a repository |
| [\`nap resolve\`](docs/generated/commands/resolve.md) | Resolve a NAP URI to its manifest or a subtree |
| [\`nap revert\`](docs/generated/commands/revert.md) | Revert a commit by hash (undoes all changes in that commit) |
| [\`nap schema\`](docs/generated/commands/schema.md) | Print a JSON Schema for manifest or commit types |
| [\`nap set\`](docs/generated/commands/set.md) | Set a property on an entity manifest |
| [\`nap sign\`](docs/generated/commands/sign.md) | Sign a manifest (stub for v0) |
| [\`nap status\`](docs/generated/commands/status.md) | Show system status |
| [\`nap switch\`](docs/generated/commands/switch.md) | Switch to a branch |
| [\`nap sync\`](docs/generated/commands/sync.md) | Sync with remote |
| [\`nap validate\`](docs/generated/commands/validate.md) | Validate a manifest against the NAP schema |
| [\`nap verify\`](docs/generated/commands/verify.md) | Verify a manifest signature (stub for v0) |



---

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


---

## Status

This is a v0 prototype. APIs and formats may change.

## License

MIT
