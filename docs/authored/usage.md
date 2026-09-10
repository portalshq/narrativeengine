# PX Usage Guide

> **PX protocol** — making narrative resources addressable, portable, resolvable, and versioned across tools, storage systems, formats, and AI workflows.

This guide covers practical workflows for four core capabilities of the `px` CLI:

| Domain | What it does |
|---|---|
| **Addressing** | Give every story element a stable, canonical `px://` URI |
| **Portability** | Move repositorys between tools, teams, and storage systems |
| **Resolution** | Look up manifests, query subtrees, trace history via CLI or HTTP API |
| **World-Building** | Model characters, locations, scenes, props, and their relationships |

---

## Abstract
Px makes narrative resources addressable, portable, resolvable, and versioned across tools, storage systems, formats, and AI workflows.
Px addresses are stable -- entity references are immutable.

---

## Table of Contents

- [Reference](#reference)
- [1. Narrative Resource Addressing](#1-narrative-resource-addressing)
  - [1.1 URI Anatomy](#11-uri-anatomy)
  - [1.2 Cross-Reference Grid](#12-cross-reference-grid)
  - [1.3 Fragment Queries for Precision Addressing](#13-fragment-queries-for-precision-addressing)
  - [1.4 Versioned Addressing for Canon Management](#14-versioned-addressing-for-canon-management)
- [2. Portability](#2-portability)
  - [2.1 Repository-as-Repo: Move Your World Anywhere](#21-repository-as-repo-move-your-world-anywhere)
  - [2.2 Content-Addressed Assets](#22-content-addressed-assets)
  - [2.3 AI Provenance Tracking](#23-ai-provenance-tracking)
  - [2.4 Manifest as Durable Artifact](#24-manifest-as-durable-artifact)
- [3. Resolution](#3-resolution)
  - [3.1 Subtree Query Engine](#31-subtree-query-engine)
  - [3.2 REST API for Integration](#32-rest-api-for-integration)
  - [3.3 Commit History as Audit Trail](#33-commit-history-as-audit-trail)
  - [3.4 Cross-Repository Discovery](#34-cross-repository-discovery)
- [4. World-Building](#4-world-building)
  - [4.1 Full World-Building Workflow](#41-full-world-building-workflow)
  - [4.2 Entity Types Reference](#42-entity-types-reference)
  - [4.3 Cross-Reference Graph](#43-cross-reference-graph)
  - [4.4 Multi-Repository Portfolio](#44-multi-repository-portfolio)
  - [4.5 Generative AI Provenance](#45-generative-ai-provenance)
- [Pro Tips](#pro-tips)

---

## Reference

```bash
# Initialize a repository in ~/.px/
px init toystory

# Create entities
px create character woody -u toystory -n "Woody"
px create location andys-room -u toystory -n "Andy's Room"
px create scene pizza-planet -u toystory -n "Pizza Planet Scene"

# Set properties with cross-references
px set px://toystory/character/woody toy_type human
px set px://toystory/character/woody homeworld "px://toystory/location/andys-room"

# Resolve manifests
px resolve px://toystory/character/woody

# Fragment queries
px resolve px://toystory/character/woody#properties.homeworld

# Subtree queries
px query px://toystory/character/woody properties

# Version control
px history px://toystory/character/woody
px branch toystory canon
px tag toystory episode-4

# HTTP server
px-server
curl http://localhost:3100/resolve/toystory/character/woody
```

---

## 1. Narrative Resource Addressing

### 1.1 URI Anatomy

Every narrative resource gets a stable, canonical `px://` URI:

```
px://toystory/character/woody#properties.homeworld
────┬── ───┬──── ────┬──── ──────┬────── ─────────────┬───────────
 scheme repository  entity_type entity_id          fragment (query)
```

**Key rules:**
- Version, branch, and tag are **never** in the URI path — they are orthogonal selectors passed alongside (mirrors Git, OCI, package managers).
- Fragment (`#`) carries the query path for subtree extraction.
- Entity type is singular in the URI (`character`, not `characters`).

### 1.2 Cross-Reference Grid

Build a web of references between entities using `px://` URIs as values:

```bash
# Create locations
px create location andys-room -u toystory -n "Andy's Room"
px create location alderaan -u toystory -n "Alderaan"
px create location deathstar -u toystory -n "Death Star"

# Create a scene
px create scene pizza-planet -u toystory -n "Pizza Planet Scene"

# Cross-reference everything with px:// URIs
px set px://toystory/character/woody homeworld "px://toystory/location/andys-room"
px set px://toystory/character/woody affiliation "toybox_alliance"
px set px://toystory/character/woody master "px://toystory/character/mrpotatohead"

px set px://toystory/location/andys-room climate "desert"
px set px://toystory/location/andys-room moons "2"

px set px://toystory/scene/pizza-planet setting "px://toystory/location/andys-room"
px set px://toystory/scene/pizza-planet participants \
  '["px://toystory/character/woody", "px://toystory/character/buzzlightyear"]'
```

The resulting manifest for Woody looks like this:

```yaml
id: px://toystory/character/woody
name: Woody
entity_type: character
version: 5
properties:
  homeworld: "px://toystory/location/andys-room"
  toy_type: human
  affiliation: toybox_alliance
  master: "px://toystory/character/mrpotatohead"
references: {}
head: a72c9f3b...
```

**Use case — story bible automation:** A script can traverse every `px://` URI in a manifest and verify it resolves. If someone deletes `px://toystory/character/mrpotatohead`, the broken reference is caught immediately.

### 1.3 Fragment Queries for Precision Addressing

Address sub-parts of a resource with `#fragment` syntax — ideal for AI agents, integration pipelines, and script generators:

```bash
# Get a single field
px resolve px://toystory/character/woody#properties.homeworld
# → "px://toystory/location/andys-room"

# Get a reference array
px resolve px://toystory/scene/pizza-planet#properties.participants

# Chain into nested objects
px resolve px://toystory/character/woody#representations.reference_image.hash

# Array index access
px resolve px://toystory/character/woody#references.appears_in.0
```

**Use case — AI context window optimization:** Instead of feeding an LLM a 40K-token manifest, pull exactly the 500 tokens it needs:

```bash
# An AI writing a scene only needs participants and setting
px query px://toystory/scene/pizza-planet properties.participants -f json
px query px://toystory/scene/pizza-planet properties.setting -f json
```

**Use case — CI/CD validation:** Verify every cross-reference resolves:

```bash
px resolve px://toystory/character/woody#references.appears_in \
  | jq -r '.[]' \
  | xargs -I{} px resolve {}
```

### 1.4 Versioned Addressing for Canon Management

Branch and tag are **orthogonal selectors** — never in the URI. Address the same resource at different points in its timeline:

```bash
# Create branches for alternate canon tracks
px branch toystory legends
px branch toystory canon
px branch toystory "what-if"

# Tag major releases
px tag toystory episode-4
px tag toystory episode-5
px tag toystory episode-6

# Resolve at specific points in time
px resolve px://toystory/character/woody --branch legends
px resolve px://toystory/character/woody --tag episode-4
px resolve px://toystory/character/woody --commit a72c9f3b
```

**Use case — divergent timelines:** In a "What If" branch, Woody joins the Toybox Corp. The `canon` branch has `affiliation: toybox_alliance`; the `what-if` branch has `affiliation: toybox_corp`. Both resolve from the same URI — only the selector differs. The manifests diverge silently, and the resolver picks the right one based on context.

```bash
px resolve px://toystory/character/woody#properties.affiliation
# → toybox_alliance

px resolve px://toystory/character/woody#properties.affiliation \
  --branch what-if
# → toybox_corp
```

---

## 2. Portability

### 2.1 Repository-as-Repo: Move Your World Anywhere

Every PX repository is **files + Git** — zero runtime dependencies. This means it works with every transport and storage system:

```bash
# Archive an entire repository as a tarball
tar czf toystory.px toystory/

# Ship it via any medium — S3, Dropbox, scp, USB drive
scp -r toystory/ user@server:/repositorys/

# Clone across teams
git clone git@github.com:studio/toystory-px.git

# Sync to shared drives
rsync -avz toystory/ /shared/drive/projects/

# Mount in cloud storage
aws s3 sync toystory/ s3://studio-assets/repositorys/toystory/
```

**Use case — multi-studio collaboration:** Studio A builds characters, Studio B builds locations, Studio C builds scenes. Each works in their own Git branch, and PX URIs are the contract between them. When they merge, the references resolve across all three.

```bash
# Studio A works on characters
git clone git@github.com:studio/toystory-px.git
px create character slinky -u toystory -n "Slinky Dog"

# Studio B works on locations
git clone git@github.com:studio/toystory-px.git
px create location deathstar -u toystory -n "Death Star"

# On merge, Studio A's character can reference Studio B's location
px set px://toystory/character/slinky base "px://toystory/location/deathstar"
```

**Use case — offline fieldwork:** A writer on a plane builds an entire repository with no internet, just the `px` binary and a text editor. When they reconnect, `git push` syncs everything.

### 2.2 Content-Addressed Assets

Manifests don't store files — they store **BLAKE3 hashes** pointing to assets. This makes everything verifiable, deduplicatable, and cacheable:

```bash
# Link a reference image by content hash
px add-repr px://toystory/character/woody reference_image \
  ./assets/woody_ref.png --format png
# ✓ Added representation 'reference_image' (png)
#   Hash: blake3:e3b0c44...
```

The manifest now contains:

```yaml
representations:
  reference_image:
    hash: "blake3:e3b0c44..."
    format: png
    uri: "./assets/woody_ref.png"
```

You can attach any asset type — images, 3D meshes, audio, video, ONNX models:

```bash
px add-repr px://toystory/character/woody voice_model \
  ./assets/woody_voice.onnx --format onnx

px add-repr px://toystory/location/andys-room concept_art \
  ./assets/andys-room_concept.png --format png

px add-repr px://toystory/prop/andy-hat mesh \
  ./assets/andy_hat.glb --format glb
```

**Use case — asset pipeline integrity:** A VFX pipeline verifies that the asset on the render farm matches the manifest hash:

```bash
echo "blake3:e3b0c44...  woody_ref.png" | b3sum --check
# woody_ref.png: OK
```

**Use case — CDN caching:** The content hash is the cache key. Same hash = same content, globally. No cache invalidation logic needed.

### 2.3 AI Provenance Tracking

Record which model, prompt, and seed generated a character design — right in the manifest:

```bash
px set px://toystory/character/woody provenance.model "midjourney-v6"
px set px://toystory/character/woody provenance.seed "8675309"
px set px://toystory/character/woody provenance.prompt_hash "blake3:abc123..."
px set px://toystory/character/woody provenance.derived_from \
  "px://toystory/character/woody/v1"
```

The manifest captures full generative lineage:

```yaml
provenance:
  model: "midjourney-v6"
  prompt_hash: "blake3:abc123..."
  seed: "8675309"
  parameters:
    stylize: "1000"
    chaos: "20"
  derived_from: "px://toystory/character/woody/v1"
  created_at: "2026-06-09T20:00:00Z"
```

**Use case — rights & attribution:** When a model is deprecated or a license changes, you can identify every asset generated with it:

```bash
px query px://toystory/character/woody provenance.model
# → midjourney-v6
```

**Use case — reproducibility:** Given the same model, prompt hash, and seed, you can regenerate an identical asset.

### 2.4 Manifest as Durable Artifact

The `.yaml` manifest is simultaneously **human-editable**, **machine-readable**, and **agent-readable**. A toybox-builder opens it in VS Code; a CI pipeline validates it; an AI agent queries it:

```yaml
# characters/slinky.yaml
id: px://toystory/character/slinky
name: "Slinky Dog"
entity_type: character
version: 3
properties:
  toy_type: human
  affiliation: toybox_corp
  accessory: red
  master: "px://toystory/character/palpatine"
  apprentice: "px://toystory/character/woody"
representations:
  voice_actor:
    hash: "blake3:f8a2b1..."
    format: wav
    uri: "gs://assets/toystory/slinky/voice.wav"
head: "f7e3d2c1a..."
```

You can commit this directly to Git, review it in PRs, diff changes — it's a first-class citizen in your development workflow.

```bash
git diff toystory/characters/slinky.yaml
# -  accessory: red
# +  accessory: blue
```

---

## 3. Resolution

### 3.1 Subtree Query Engine

The `query` command extracts exactly the data you need from deep manifest trees — no full-file parsing required:

```bash
# Get the first scene a character appears in
px query px://toystory/character/woody references.appears_in.0

# Get just image hashes across all characters (for caching)
px query px://toystory/character/woody representations.reference_image.hash

# List available keys for tab completion / introspection
px resolve px://toystory/character/woody#representations

# Different output formats
px query px://toystory/character/woody properties -f json
px query px://toystory/character/woody properties -f yaml
```

**Use case — AI story generator:** A GPT agent builds a scene by querying the setting, participants, and mood, then generates appropriate dialog — all from fragment queries:

```bash
# Agent gathers context into variables
SETTING=$(px query px://toystory/scene/pizza-planet properties -f json)
MOOD=$(px query px://toystory/scene/pizza-planet properties.mood -f json)
PARTICIPANTS=$(px query px://toystory/scene/pizza-planet properties.participants -f json)

# Agent generates scene using only the relevant data
echo "Setting: $SETTING"
echo "Participants: $PARTICIPANTS"
```

**Use case — API response size optimization:** A mobile client fetching character info only needs the `properties` subtree, not the full manifest (which may include provenance data, representations metadata, references arrays, etc.):

```bash
px query px://toystory/character/woody properties -f json
# Returns ~200 bytes instead of ~2000
```

### 3.2 REST API for Integration

The `px-server` exposes the full resolver as an HTTP API — ideal for web UIs, game engines, and microservices:

```bash
# Start the server (defaults to port 3100)
cargo run -p px-server

# Custom port and base path
PX_PORT=8080 PX_BASE_PATH=/path/to/repositorys px-server
```

#### API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/resolve/{repository}/{entity_type}/{entity_id}` | Resolve a manifest |
| `GET` | `/resolve/{repository}/{entity_type}/{entity_id}?branch=canon` | Resolve at a branch |
| `POST` | `/commit/{repository}/{entity_type}/{entity_id}` | Commit changes |
| `GET` | `/history/{repository}/{entity_type}/{entity_id}` | Get commit history |
| `GET` | `/repositorys` | List all repositorys |
| `GET` | `/repositorys/{repository}/entities` | List entities in a repository |
| `GET` | `/health` | Health check |

Resolution query parameters: `branch`, `commit`, `tag`, `path` (subtree query).

#### Examples

```bash
# Resolve a manifest
curl http://localhost:3100/resolve/toystory/character/woody

# With branch selector
curl "http://localhost:3100/resolve/toystory/character/woody?branch=canon"

# Subtree query via API
curl "http://localhost:3100/resolve/toystory/character/woody?path=properties.toy_type"

# List everything
curl http://localhost:3100/repositorys
curl http://localhost:3100/repositorys/toystory/entities?type=character

# Commit changes via API
curl -X POST http://localhost:3100/commit/toystory/character/woody \
  -H "Content-Type: application/json" \
  -d '{
    "message": "update toy_type",
    "author": "dev@studio.com",
    "properties": {
      "toy_type": "plush"
    }
  }'
```

**Use case — game engine integration:** A Unity or Unreal plugin queries the PX server at build time to populate character data, spawn points, and prop manifests:

```csharp
// Unity example — fetch character data at editor time
string json = new WebClient().DownloadString(
    "http://localhost:3100/resolve/toystory/character/woody?path=properties"
);
CharacterData data = JsonUtility.FromJson<CharacterData>(json);
```

**Use case — web dashboard:** A worldbuilding wiki resolves manifests on the fly to render character sheets, location maps, and scene timelines:

```javascript
// React example — resolve character for profile page
const { data } = await fetch(
  `/api/resolve/${repository}/character/${characterId}`
);
```

### 3.3 Commit History as Audit Trail

Every change is content-addressed and versioned. Trace exactly how a character evolved and who made each change:

```bash
# View commit history
px history px://toystory/character/woody -n 20
# a72c9f3 2026-06-09T20:15:00Z — set toy_type to human — alice
# b83d1a2 2026-06-09T20:10:00Z — set homeworld — alice
# c94e2b1 2026-06-09T20:05:00Z — added reference_image — bob
# d05f3c0 2026-06-09T20:00:00Z — Create character 'Woody' — alice

# Resolve what the manifest looked like at a specific commit
px resolve px://toystory/character/woody --commit b83d1a2

# View history via API
curl http://localhost:3100/history/toystory/character/woody
```

**Use case — canon dispute resolution:** When two writers disagree on whether Woody's hair color changed between drafts, the commit log shows exactly when and by whom it was modified:

```bash
px history px://toystory/character/woody | grep "hair"
# f7a2b1c 2026-06-08T14:30:00Z — set hair_color to brown — bob
# e8d3c2b 2026-06-07T09:15:00Z — set hair_color to blond — alice
```

**Use case — rollback:** Revert a character to a known good state:

```bash
git -C toystory revert b83d1a2
```

### 3.4 Cross-Repository Discovery

Discover what repositorys and entities are available:

```bash
# List all repositorys in the base directory
px list
# px://toystory/
# px://toystory/
# px://middleearth/

# List all entities in a repository
px list toystory
# character:
#   px://toystory/character/woody
#   px://toystory/character/slinky
# location:
#   px://toystory/location/andys-room
#   px://toystory/location/deathstar
# scene:
#   px://toystory/scene/pizza-planet

# Filter by type
px list toystory -t character
# character:
#   px://toystory/character/woody
#   px://toystory/character/slinky
```

---

## 4. World-Building

### 4.1 Full World-Building Workflow

Build out a repository from scratch with a structured workflow:

```bash
# Step 1: Initialize the repository
px init myworld

# Step 2: Define the world metadata
px set px://myworld/world/myworld canon_level "canon"
px set px://myworld/world/myworld timeline "Age of Discovery"
px set px://myworld/world/myworld theme "exploration vs exploitation"

# Step 3: Create factions / groups as properties on the world
px set px://myworld/world/myworld factions \
  '["The Commonwealth", "The Outer Rim Syndicate", "The Core"]'

# Step 4: Create characters
px create character captain-rex -u myworld -n "Captain Rex"
px create character admiral-torres -u myworld -n "Admiral Torres"
px create character lyra -u myworld -n "Lyra"

# Step 5: Flesh out character properties
px set px://myworld/character/captain-rex rank "Captain"
px set px://myworld/character/captain-rex affiliation "The Commonwealth"
px set px://myworld/character/captain-rex ship "px://myworld/prop/valkyrie"

# Step 6: Create locations
px create location kyra-prime -u myworld -n "Kyra Prime"
px set px://myworld/location/kyra-prime type "colonial_capital"
px set px://myworld/location/kyra-prime controlled_by "The Commonwealth"

# Step 7: Create scenes that connect everything
px create scene first-contact -u myworld -n "First Contact"
px set px://myworld/scene/first-contact setting "px://myworld/location/kyra-prime"
px set px://myworld/scene/first-contact participants \
  '[
    "px://myworld/character/captain-rex",
    "px://myworld/character/lyra"
  ]'
px set px://myworld/scene/first-contact mood "tense"
px set px://myworld/scene/first-contact outcome "alliance_formed"

# Step 8: Add reference images
px add-repr px://myworld/character/captain-rex reference_image \
  ./concept/rex.png --format png

# Step 9: Commit everything
px commit myworld -m "Complete first act world-building" -a "writer@studio.com"
```

### 4.2 Entity Types Reference

| Type | URI Pattern | What it models | Example properties |
|---|---|---|---|
| `world` | `px://<name>/world/<name>` | The repository itself — rules, canon level, metadata | `canon_level`, `timeline`, `theme`, `factions` |
| `character` | `px://<name>/character/<id>` | Persistent character with identity across scenes | `homeworld`, `toy_type`, `affiliation`, `master`, `apprentice` |
| `location` | `px://<name>/location/<id>` | Spatial setting | `climate`, `type`, `controlled_by`, `population` |
| `scene` | `px://<name>/scene/<id>` | Narrative moment — participants, timeline, events | `setting`, `participants`, `mood`, `outcome`, `time_of_day` |
| `prop` | `px://<name>/prop/<id>` | Physical object with materials, variants, ownership | `owner`, `material`, `weight`, `color` |

**World manifest** (`repository.yaml` — created automatically):

```yaml
id: px://myworld/world/myworld
name: myworld Repository
entity_type: world
version: 3
properties:
  canon_level: canon
  timeline: "Age of Discovery"
  theme: "exploration vs exploitation"
  factions:
    - "The Commonwealth"
    - "The Outer Rim Syndicate"
representations: {}
references: {}
head: a72c9f3b...
```

### 4.3 Cross-Reference Graph

The `references` field builds a directed graph between entities. This enables rich queries across your repository:

```bash
# Character → scenes they appear in
px set px://toystory/character/woody references.appears_in \
  '["px://toystory/scene/pizza-planet", "px://toystory/scene/trenchrun"]'

# Location → scenes set there
px set px://toystory/location/andys-room references.appears_in \
  '["px://toystory/scene/pizza-planet"]'

# Character → relationships
px set px://toystory/character/woody references.relationships \
  '[
    {"target": "px://toystory/character/slinky", "type": "father"},
    {"target": "px://toystory/character/jessie", "type": "sister"},
    {"target": "px://toystory/character/rex", "type": "friend"}
  ]'

# Prop → owner
px set px://toystory/prop/andy-hat references.owner "px://toystory/character/andy"
```

**Graph traversal examples:**

```bash
# Find all scenes a character appears in
px query px://toystory/character/woody references.appears_in

# Find all characters that visit a location
# (resolve scene participants for each scene set at the location)
px resolve px://toystory/scene/pizza-planet#properties.participants

# Find a character's relationships
px query px://toystory/character/woody references.relationships -f json
```

### 4.4 Multi-Repository Portfolio

Manage multiple fictional worlds under one resolver:

```bash
# Create repositorys side by side
px init toystory
px init toystory
px init middleearth

# Every repository is independently addressable
px create character buzzlightyear -u toystory -n "Buzz Lightyear"
px create location andysroom -u toystory -n "Andy's Room"
px create character frodo -u middleearth -n "Frodo Baggins"
px create location theshire -u middleearth -n "The Shire"

# List all repositorys
px list
# px://toystory/
# px://toystory/
# px://middleearth/

# Each repository has its own Git history, branches, tags
px branch middleearth canon
px tag middleearth fellowship-of-the-ring
```

**Repository directory layout (default: `~/.px/`):**

```
~/.px/
├── toystory/              ← independent Git repo
│   ├── .px/config.yaml
│   ├── repository.yaml
│   ├── characters/
│   ├── locations/
│   ├── scenes/
│   └── props/
├── toystory/              ← independent Git repo
│   ├── .px/config.yaml
│   ├── repository.yaml
│   ├── characters/
│   ├── locations/
│   ├── scenes/
│   └── props/
└── middleearth/           ← independent Git repo
    ├── .px/config.yaml
    ├── repository.yaml
    ├── characters/
    ├── locations/
    ├── scenes/
    └── props/
```

### 4.5 Generative AI Provenance

Track every AI-generated asset with full lineage:

```bash
# After generating a character design with Midjourney
px set px://toystory/character/woody provenance.model "midjourney-v6"
px set px://toystory/character/woody provenance.seed "8675309"
px set px://toystory/character/woody provenance.prompt_hash "blake3:abc123..."
px set px://toystory/character/woody provenance.parameters.stylize "1000"

# After iterating with an LLM
px set px://toystory/character/woody provenance.derived_from \
  "px://toystory/character/woody/v1"

# Set the creation timestamp
px set px://toystory/character/woody provenance.created_at "2026-06-09T20:00:00Z"
```

The provenance block captures complete generative lineage:

```yaml
provenance:
  model: "midjourney-v6"
  prompt_hash: "blake3:abc123..."
  seed: "8675309"
  parameters:
    stylize: "1000"
    chaos: "20"
  derived_from: "px://toystory/character/woody/v1"
  created_at: "2026-06-09T20:00:00Z"
```

**Use case — rights & attribution:** When a model is deprecated or a license changes, identify every asset generated with it:

```bash
# Find all entities generated with a specific model
px query px://toystory/character/woody provenance.model
# → midjourney-v6
```

**Use case — reproducibility:** Given the same model, prompt hash, seed, and parameters, you can regenerate an identical asset for A/B testing or re-rendering at higher resolution.

---

## Pro Tips

### Composition

```bash
# Initialize a repository (defaults to ~/.px/)
px init myrepository

# Override with any directory
px init myrepository -d /path/to/shared/repositorys
px init myrepository -d ~/Dropbox/TeamWorldbuilding

# Point the resolver at a specific path
px -d /mnt/nas/repositorys list
px -d /mnt/nas/repositorys resolve px://toystory/character/woody
```

### Output Formats

```bash
# JSON for programmatic consumption
px resolve px://toystory/character/woody -f json | jq '.properties'

# YAML for human review and editing
px resolve px://toystory/character/woody -f yaml
```

### Debugging

```bash
# Verbose mode shows tracing output
px -v resolve px://toystory/character/woody

# Check server health
curl http://localhost:3100/health
# {"status":"ok","protocol":"PX","version":"0.1.0"}
```

### Quick Bootstrap a New Repository

```bash
# One-liner to seed a new repository
px init mynovel \
  && px create character hero -u mynovel -n "The Hero" \
  && px create location village -u mynovel -n "Home Village" \
  && px set px://mynovel/character/hero homeworld "px://mynovel/location/village" \
  && px set px://mynovel/character/hero archetype "reluctant hero"
```

### Working Without Git

While PX uses Git for version control, all of your data is plain YAML files. You can:

- Edit manifests directly in any text editor
- Version them with any VCS (Fossil, Mercurial, Jujutsu)
- Sync them via Dropbox, Google Drive, or any file sync tool
- Process them with any YAML toolchain

---

## Repository Layout Reference

```
toystory/                    ← repository root (Git repo)
├── .px/
│   └── config.yaml          ← PX repository configuration
├── repository.yaml            ← world manifest
├── characters/
│   ├── woody.yaml
│   └── slinky.yaml
├── locations/
│   ├── andys-room.yaml
│   └── deathstar.yaml
├── scenes/
│   └── pizza-planet.yaml
└── props/
```

---

## CLI Command Reference

| Command | Description |
|---|---|
| `px init <repository>` | Initialize a new repository repository (prompts for provider on first run) |
| `px init <repository> --provider <type>` | Initialize repository + configure provider |
| `px init --provider <type>` | Configure provider only (no repository) |
| `px create <type> <id> -u <repository> -n <name>` | Create a new entity manifest |
| `px resolve <uri>` | Resolve a PX URI to a manifest or subtree |
| `px query <uri> <path>` | Query a subtree from a manifest |
| `px set <uri> <key> <value>` | Set a property on an entity |
| `px add-repr <uri> <key> <file> --format <fmt>` | Add a content-addressed representation |
| `px commit <repository> -m <message>` | Commit changes to the VCS |
| `px history <uri>` | View commit history for an entity |
| `px list [repository]` | List repositorys or entities |
| `px branch <repository> [name]` | Create or list branches |
| `px switch <repository> <branch>` | Switch to a branch |
| `px tag <repository> [name]` | Create or list tags |
| `px validate <uri>` | Validate a manifest against the PX schema |
| `px publish <repository>` | Push current branch to origin (convenience shortcut) |
| `px push <repository>` | Push to a configurable remote (use `--remote` and `--branch`) |
| `px pull <url-or-name>` | Clone from URL or pull an existing repository |
| `px sync <repository>` | Pull current branch from default remote |
| `px remote add <repository> <name> <url>` | Add a remote |
| `px remote ls <repository>` | List remotes |
| `px remote rm <repository> <name>` | Remove a remote |
| `px revert <repository> -c <hash>` | Revert a commit |
| `px diff <base> <candidate>` | Diff two manifest files |
| `px merge <base> <current> <proposed>` | Three-way merge |
| `px content-hash <file>` | Compute BLAKE3 content hash |
| `px schema <name>` | Print JSON schema (manifest or commit) |
| `px sign <uri>` | Sign a manifest (stub in v0) |
| `px verify <uri>` | Verify a manifest signature (stub in v0) |
| `px doctor [--repair]` | Run diagnostics and optionally auto-repair |
| `px status` | Show system status and provider info |
| `px choose backend --provider <type>` | Switch backend provider |

### Command Notes

**`publish` vs `push`:** `publish` is a convenience alias that always pushes the current branch to the `origin` remote. `push` is the full form — use `--remote` and `--branch` to control the target.

**`sync` vs `pull`:** `sync` pulls the current branch from the default remote. `pull` accepts either a repository name (same as `sync`) or a URL (which clones the repository).

**`init` vs `choose`:** `init` can both create a repository and configure a provider. `choose` only switches providers — use it when you want to change providers without creating a repository.

Global options: `-d/--base-dir <path>` (default `~/.px`), `-v/--verbose`, `-f/--format <yaml|json>`

Repository repositories are stored in `~/.px/<repository>/` by default. Use `-d` (or `--base-dir`) to point to a different directory.

---

## Design Principles

- **Manifest is current state. History is external.** Manifests store only `head` — a pointer to the latest commit. Full history lives in the VCS, preventing unbounded manifest growth.
- **Version/branch/tag are NEVER in the URI.** They are orthogonal selectors passed alongside the URI (mirrors Git, OCI, package managers).
- **Content-address everything.** Every representation is identified by its BLAKE3 hash. Manifests are content-hashable for signing and verification.
- **Subtree queries are first-class.** AI systems, CLI tools, and HTTP clients all use the same query engine. Fragment queries enable efficient data access without fetching entire manifests.

---

*PX is in v0 (prototype). The core data model and resolution engine are functional. Advanced features (signing, verification, distributed resolution) are planned for future iterations.*
