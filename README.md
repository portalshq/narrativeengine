# NAP — Narrative Addressing Protocol

**NAP is a protocol that makes narrative resources addressable, resolvable, and interoperable across tools, storage systems, formats, and AI workflows.**

Characters, locations, scenes, props, and entire fictional universes — NAP gives each one a stable URI, a human-and-machine-readable manifest, a content-addressed history, and a resolver that connects them all.

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
 scheme universe  entity_type entity_id          fragment (query)
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

Commits are content-addressed (SHA-256) snapshots with patch metadata. The manifest stores only `head` — a pointer to the latest commit. Full history lives in the VCS, keeping manifests bounded.

### 4. Resolver — URI → Manifest

The resolver turns a `nap://` URI into a manifest (or a subtree of one). With optional selectors for branch, tag, or commit hash, it supports versioned resolution and fragment-based queries for efficient data access.

---

## Entity Types

| Type | Example URI | Description |
|---|---|---|
| `character` | `nap://starwars/character/lukeskywalker` | Persistent character with identity across scenes/episodes |
| `location` | `nap://starwars/location/tatooine` | Spatial location within a fictional universe |
| `scene` | `nap://starwars/scene/cantina` | Narrative scene — participants, timeline, events |
| `prop` | `nap://toystory/prop/andy-hat` | Physical object with materials, variants, ownership |
| `world` | `nap://starwars/world/starwars` | The universe itself — rules, canon, top-level metadata |

---

## Repository Layout

Each universe is a Git repository on disk:

```text
starwars/                    ← universe root (Git repo)
├── .nap/
│   └── config.yaml          ← repository configuration
├── universe.yaml            ← world manifest
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

## Quick Start

### Install

```bash
# Build from source
git clone https://github.com/cinematiccanvas/nap.git
cd nap
cargo build --release

# Binaries land in target/release/
#   nap          — CLI tool
#   nap-server   — HTTP resolver server
```

### Create a Universe

```bash
# Initialize a new universe
nap init starwars

# See what you created
ls starwars/
# → .nap/  universe.yaml  characters/  locations/  scenes/  props/
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

# Create tags
nap tag starwars episode-4
```

### Output Formats

```bash
nap resolve nap://starwars/character/lukeskywalker -f json
nap resolve nap://starwars/character/lukeskywalker -f yaml
```

---

## HTTP Server

The NAP resolver server provides a REST API for resolution and commits.

```bash
# Start the server (defaults to port 3100, base path = current directory)
nap-server

# Custom port and base path
NAP_PORT=8080 NAP_BASE_PATH=/path/to/universes nap-server
```

### Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/resolve/{universe}/{entity_type}/{entity_id}` | Resolve a manifest |
| `GET` | `/resolve/{universe}/{entity_type}/{entity_id}?branch=canon` | Resolve at a branch |
| `POST` | `/commit/{universe}/{entity_type}/{entity_id}` | Commit changes |
| `GET` | `/history/{universe}/{entity_type}/{entity_id}` | Get commit history |
| `GET` | `/universes` | List all universes |
| `GET` | `/universes/{universe}/entities` | List entities in a universe |
| `GET` | `/health` | Health check |

Query parameters for resolution: `branch`, `commit`, `tag`, `path` (subtree query).

---

## AI Workflows

NAP is designed for AI-native workflows from day one:

**Subtree queries** let AI agents fetch exactly the data they need — 500 tokens instead of 40,000:
```bash
nap resolve nap://starwars/character/lukeskywalker#references.appears_in
nap resolve nap://starwars/scene/cantina#properties.mood
```

**Provenance tracking** records AI generation metadata in the manifest itself:
```yaml
provenance:
  model: "midjourney-v6"
  prompt_hash: "sha256:abc123..."
  seed: "42"
  derived_from: "nap://starwars/character/lukeskywalker/v1"
```

**Content-addressed representations** link manifests to assets by hash:
```yaml
representations:
  reference_image:
    hash: "sha256:e3b0c44..."
    format: png
  voice_model:
    hash: "sha256:def567..."
    format: onnx
```

---

## Project Structure

```
nap/
├── Cargo.toml              ← workspace root
├── nap-core/               ← core library
│   └── src/
│       ├── lib.rs          ← crate root, re-exports
│       ├── uri.rs          ← NapUri parser/builder
│       ├── manifest.rs     ← Manifest, Representation, Provenance
│       ├── commit.rs       ← Commit, Change, ChangeOp
│       ├── resolver.rs     ← Resolver (URI → Manifest)
│       ├── query.rs        ← Subtree query engine
│       ├── repository.rs   ← Universe repository CRUD
│       ├── types.rs        ← EntityType enum
│       ├── content.rs      ← SHA-256 content hashing
│       ├── error.rs        ← NapError types
│       ├── vcs.rs          ← VcsBackend trait
│       └── vcs_git.rs      ← Git backend implementation
├── nap-cli/                ← CLI binary (nap)
│   └── src/main.rs
└── nap-server/             ← HTTP server binary (nap-server)
    └── src/main.rs
```

---

## Build & Test

### Prerequisites

- [Rust](https://rustup.rs/) 2024 edition (1.85+)
- Git (for the VCS backend)

### Build

```bash
# Build everything (debug)
cargo build

# Build everything (release)
cargo build --release

# Build individual crates
cargo build -p nap-core
cargo build -p nap-cli
cargo build -p nap-server
```

### Test

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p nap-core

# Run a specific test
cargo test test_resolve_full_manifest

# Run tests with output
cargo test -- --nocapture

# Run doc tests
cargo test --doc
```

All 49 unit tests and 2 doc tests pass.

### Run

```bash
# CLI
cargo run -p nap-cli -- --help

# Server
cargo run -p nap-server
```

---

## Design Principles

**Manifest is current state. History is external.**
- Manifests store only `head` — a pointer to the latest commit.
- Full history lives in the VCS, preventing unbounded manifest growth.

**Version/branch/tag are NEVER in the URI.**
- They are orthogonal selectors passed alongside the URI (mirrors Git, OCI, package managers).

**Content-address everything.**
- Every representation is identified by its SHA-256 hash.
- Manifests are content-hashable for signing and verification.

**Subtree queries are first-class.**
- AI systems, CLI tools, and HTTP clients all use the same query engine.
- Fragment queries enable efficient data access without fetching entire manifests.

---

## Status

NAP is in **v0 (prototype)** — the core data model and resolution engine are functional. Signing and verification are stubbed for future iterations.

---

## License

MIT © Cinematic Canvas
