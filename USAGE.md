# NAP Usage Guide

> **Narrative Addressing Protocol** — making narrative resources addressable, portable, resolvable, and versioned across tools, storage systems, formats, and AI workflows.

This guide covers practical workflows for four core capabilities of the `nap` CLI:

| Domain | What it does |
|---|---|
| **Addressing** | Give every story element a stable, canonical `nap://` URI |
| **Portability** | Move universes between tools, teams, and storage systems |
| **Resolution** | Look up manifests, query subtrees, trace history via CLI or HTTP API |
| **World-Building** | Model characters, locations, scenes, props, and their relationships |

---

## Table of Contents

- [Quick Reference](#quick-reference)
- [1. Narrative Resource Addressing](#1-narrative-resource-addressing)
  - [1.1 URI Anatomy](#11-uri-anatomy)
  - [1.2 Cross-Reference Grid](#12-cross-reference-grid)
  - [1.3 Fragment Queries for Precision Addressing](#13-fragment-queries-for-precision-addressing)
  - [1.4 Versioned Addressing for Canon Management](#14-versioned-addressing-for-canon-management)
- [2. Portability](#2-portability)
  - [2.1 Universe-as-Repo: Move Your World Anywhere](#21-universe-as-repo-move-your-world-anywhere)
  - [2.2 Content-Addressed Assets](#22-content-addressed-assets)
  - [2.3 AI Provenance Tracking](#23-ai-provenance-tracking)
  - [2.4 Manifest as Durable Artifact](#24-manifest-as-durable-artifact)
- [3. Resolution](#3-resolution)
  - [3.1 Subtree Query Engine](#31-subtree-query-engine)
  - [3.2 REST API for Integration](#32-rest-api-for-integration)
  - [3.3 Commit History as Audit Trail](#33-commit-history-as-audit-trail)
  - [3.4 Cross-Universe Discovery](#34-cross-universe-discovery)
- [4. World-Building](#4-world-building)
  - [4.1 Full World-Building Workflow](#41-full-world-building-workflow)
  - [4.2 Entity Types Reference](#42-entity-types-reference)
  - [4.3 Cross-Reference Graph](#43-cross-reference-graph)
  - [4.4 Multi-Universe Portfolio](#44-multi-universe-portfolio)
  - [4.5 Generative AI Provenance](#45-generative-ai-provenance)
- [Pro Tips](#pro-tips)

---

## Quick Reference

```bash
# Initialize a universe
nap init starwars

# Create entities
nap create character lukeskywalker -u starwars -n "Luke Skywalker"
nap create location tatooine -u starwars -n "Tatooine"
nap create scene cantina -u starwars -n "Cantina Scene"

# Set properties with cross-references
nap set nap://starwars/character/lukeskywalker species human
nap set nap://starwars/character/lukeskywalker homeworld "nap://starwars/location/tatooine"

# Resolve manifests
nap resolve nap://starwars/character/lukeskywalker

# Fragment queries
nap resolve nap://starwars/character/lukeskywalker#properties.homeworld

# Subtree queries
nap query nap://starwars/character/lukeskywalker properties

# Version control
nap history nap://starwars/character/lukeskywalker
nap branch starwars canon
nap tag starwars episode-4

# HTTP server
nap-server
curl http://localhost:3100/resolve/starwars/character/lukeskywalker
```

---

## 1. Narrative Resource Addressing

### 1.1 URI Anatomy

Every narrative resource gets a stable, canonical `nap://` URI:

```
nap://starwars/character/lukeskywalker#properties.homeworld
────┬── ───┬──── ────┬──── ──────┬────── ─────────────┬───────────
 scheme universe  entity_type entity_id          fragment (query)
```

**Key rules:**
- Version, branch, and tag are **never** in the URI path — they are orthogonal selectors passed alongside (mirrors Git, OCI, package managers).
- Fragment (`#`) carries the query path for subtree extraction.
- Entity type is singular in the URI (`character`, not `characters`).

### 1.2 Cross-Reference Grid

Build a web of references between entities using `nap://` URIs as values:

```bash
# Create locations
nap create location tatooine -u starwars -n "Tatooine"
nap create location alderaan -u starwars -n "Alderaan"
nap create location deathstar -u starwars -n "Death Star"

# Create a scene
nap create scene cantina -u starwars -n "Cantina Scene"

# Cross-reference everything with nap:// URIs
nap set nap://starwars/character/lukeskywalker homeworld "nap://starwars/location/tatooine"
nap set nap://starwars/character/lukeskywalker affiliation "rebel_alliance"
nap set nap://starwars/character/lukeskywalker master "nap://starwars/character/yoda"

nap set nap://starwars/location/tatooine climate "desert"
nap set nap://starwars/location/tatooine moons "2"

nap set nap://starwars/scene/cantina setting "nap://starwars/location/tatooine"
nap set nap://starwars/scene/cantina participants \
  '["nap://starwars/character/lukeskywalker", "nap://starwars/character/obiwankenobi"]'
```

The resulting manifest for Luke Skywalker looks like this:

```yaml
id: nap://starwars/character/lukeskywalker
name: Luke Skywalker
entity_type: character
version: 5
properties:
  homeworld: "nap://starwars/location/tatooine"
  species: human
  affiliation: rebel_alliance
  master: "nap://starwars/character/yoda"
references: {}
head: a72c9f3b...
```

**Use case — story bible automation:** A script can traverse every `nap://` URI in a manifest and verify it resolves. If someone deletes `nap://starwars/character/yoda`, the broken reference is caught immediately.

### 1.3 Fragment Queries for Precision Addressing

Address sub-parts of a resource with `#fragment` syntax — ideal for AI agents, integration pipelines, and script generators:

```bash
# Get a single field
nap resolve nap://starwars/character/lukeskywalker#properties.homeworld
# → "nap://starwars/location/tatooine"

# Get a reference array
nap resolve nap://starwars/scene/cantina#properties.participants

# Chain into nested objects
nap resolve nap://starwars/character/lukeskywalker#representations.reference_image.hash

# Array index access
nap resolve nap://starwars/character/lukeskywalker#references.appears_in.0
```

**Use case — AI context window optimization:** Instead of feeding an LLM a 40K-token manifest, pull exactly the 500 tokens it needs:

```bash
# An AI writing a scene only needs participants and setting
nap query nap://starwars/scene/cantina properties.participants -f json
nap query nap://starwars/scene/cantina properties.setting -f json
```

**Use case — CI/CD validation:** Verify every cross-reference resolves:

```bash
nap resolve nap://starwars/character/lukeskywalker#references.appears_in \
  | jq -r '.[]' \
  | xargs -I{} nap resolve {}
```

### 1.4 Versioned Addressing for Canon Management

Branch and tag are **orthogonal selectors** — never in the URI. Address the same resource at different points in its timeline:

```bash
# Create branches for alternate canon tracks
nap branch starwars legends
nap branch starwars canon
nap branch starwars "what-if"

# Tag major releases
nap tag starwars episode-4
nap tag starwars episode-5
nap tag starwars episode-6

# Resolve at specific points in time
nap resolve nap://starwars/character/lukeskywalker --branch legends
nap resolve nap://starwars/character/lukeskywalker --tag episode-4
nap resolve nap://starwars/character/lukeskywalker --commit a72c9f3b
```

**Use case — divergent timelines:** In a "What If" branch, Luke joins the Empire. The `canon` branch has `affiliation: rebel_alliance`; the `what-if` branch has `affiliation: galactic_empire`. Both resolve from the same URI — only the selector differs. The manifests diverge silently, and the resolver picks the right one based on context.

```bash
nap resolve nap://starwars/character/lukeskywalker#properties.affiliation
# → rebel_alliance

nap resolve nap://starwars/character/lukeskywalker#properties.affiliation \
  --branch what-if
# → galactic_empire
```

---

## 2. Portability

### 2.1 Universe-as-Repo: Move Your World Anywhere

Every NAP universe is **files + Git** — zero runtime dependencies. This means it works with every transport and storage system:

```bash
# Archive an entire universe as a tarball
tar czf starwars.nap starwars/

# Ship it via any medium — S3, Dropbox, scp, USB drive
scp -r starwars/ user@server:/universes/

# Clone across teams
git clone git@github.com:studio/starwars-nap.git

# Sync to shared drives
rsync -avz starwars/ /shared/drive/projects/

# Mount in cloud storage
aws s3 sync starwars/ s3://studio-assets/universes/starwars/
```

**Use case — multi-studio collaboration:** Studio A builds characters, Studio B builds locations, Studio C builds scenes. Each works in their own Git branch, and NAP URIs are the contract between them. When they merge, the references resolve across all three.

```bash
# Studio A works on characters
git clone git@github.com:studio/starwars-nap.git
nap create character darthvader -u starwars -n "Darth Vader"

# Studio B works on locations
git clone git@github.com:studio/starwars-nap.git
nap create location deathstar -u starwars -n "Death Star"

# On merge, Studio A's character can reference Studio B's location
nap set nap://starwars/character/darthvader base "nap://starwars/location/deathstar"
```

**Use case — offline fieldwork:** A writer on a plane builds an entire universe with no internet, just the `nap` binary and a text editor. When they reconnect, `git push` syncs everything.

### 2.2 Content-Addressed Assets

Manifests don't store files — they store **SHA-256 hashes** pointing to assets. This makes everything verifiable, deduplicatable, and cacheable:

```bash
# Link a reference image by content hash
nap add-repr nap://starwars/character/lukeskywalker reference_image \
  ./assets/luke_ref.png --format png
# ✓ Added representation 'reference_image' (png)
#   Hash: sha256:e3b0c44...
```

The manifest now contains:

```yaml
representations:
  reference_image:
    hash: "sha256:e3b0c44..."
    format: png
    uri: "./assets/luke_ref.png"
```

You can attach any asset type — images, 3D meshes, audio, video, ONNX models:

```bash
nap add-repr nap://starwars/character/lukeskywalker voice_model \
  ./assets/luke_voice.onnx --format onnx

nap add-repr nap://starwars/location/tatooine concept_art \
  ./assets/tatooine_concept.png --format png

nap add-repr nap://toystory/prop/andy-hat mesh \
  ./assets/andy_hat.glb --format glb
```

**Use case — asset pipeline integrity:** A VFX pipeline verifies that the asset on the render farm matches the manifest hash:

```bash
echo "sha256:e3b0c44...  luke_ref.png" | sha256sum -c
# luke_ref.png: OK
```

**Use case — CDN caching:** The content hash is the cache key. Same hash = same content, globally. No cache invalidation logic needed.

### 2.3 AI Provenance Tracking

Record which model, prompt, and seed generated a character design — right in the manifest:

```bash
nap set nap://starwars/character/lukeskywalker provenance.model "midjourney-v6"
nap set nap://starwars/character/lukeskywalker provenance.seed "8675309"
nap set nap://starwars/character/lukeskywalker provenance.prompt_hash "sha256:abc123..."
nap set nap://starwars/character/lukeskywalker provenance.derived_from \
  "nap://starwars/character/lukeskywalker/v1"
```

The manifest captures full generative lineage:

```yaml
provenance:
  model: "midjourney-v6"
  prompt_hash: "sha256:abc123..."
  seed: "8675309"
  parameters:
    stylize: "1000"
    chaos: "20"
  derived_from: "nap://starwars/character/lukeskywalker/v1"
  created_at: "2026-06-09T20:00:00Z"
```

**Use case — rights & attribution:** When a model is deprecated or a license changes, you can identify every asset generated with it:

```bash
nap query nap://starwars/character/lukeskywalker provenance.model
# → midjourney-v6
```

**Use case — reproducibility:** Given the same model, prompt hash, and seed, you can regenerate an identical asset.

### 2.4 Manifest as Durable Artifact

The `.yaml` manifest is simultaneously **human-editable**, **machine-readable**, and **agent-readable**. A worldbuilder opens it in VS Code; a CI pipeline validates it; an AI agent queries it:

```yaml
# characters/darthvader.yaml
id: nap://starwars/character/darthvader
name: "Darth Vader"
entity_type: character
version: 3
properties:
  species: human
  affiliation: galactic_empire
  lightsaber_color: red
  master: "nap://starwars/character/palpatine"
  apprentice: "nap://starwars/character/lukeskywalker"
representations:
  voice_actor:
    hash: "sha256:f8a2b1..."
    format: wav
    uri: "gs://assets/starwars/vader/voice.wav"
head: "f7e3d2c1a..."
```

You can commit this directly to Git, review it in PRs, diff changes — it's a first-class citizen in your development workflow.

```bash
git diff starwars/characters/darthvader.yaml
# -  lightsaber_color: red
# +  lightsaber_color: blue
```

---

## 3. Resolution

### 3.1 Subtree Query Engine

The `query` command extracts exactly the data you need from deep manifest trees — no full-file parsing required:

```bash
# Get the first scene a character appears in
nap query nap://starwars/character/lukeskywalker references.appears_in.0

# Get just image hashes across all characters (for caching)
nap query nap://starwars/character/lukeskywalker representations.reference_image.hash

# List available keys for tab completion / introspection
nap resolve nap://starwars/character/lukeskywalker#representations

# Different output formats
nap query nap://starwars/character/lukeskywalker properties -f json
nap query nap://starwars/character/lukeskywalker properties -f yaml
```

**Use case — AI story generator:** A GPT agent builds a scene by querying the setting, participants, and mood, then generates appropriate dialog — all from fragment queries:

```bash
# Agent gathers context into variables
SETTING=$(nap query nap://starwars/scene/cantina properties -f json)
MOOD=$(nap query nap://starwars/scene/cantina properties.mood -f json)
PARTICIPANTS=$(nap query nap://starwars/scene/cantina properties.participants -f json)

# Agent generates scene using only the relevant data
echo "Setting: $SETTING"
echo "Participants: $PARTICIPANTS"
```

**Use case — API response size optimization:** A mobile client fetching character info only needs the `properties` subtree, not the full manifest (which may include provenance data, representations metadata, references arrays, etc.):

```bash
nap query nap://starwars/character/lukeskywalker properties -f json
# Returns ~200 bytes instead of ~2000
```

### 3.2 REST API for Integration

The `nap-server` exposes the full resolver as an HTTP API — ideal for web UIs, game engines, and microservices:

```bash
# Start the server (defaults to port 3100)
cargo run -p nap-server

# Custom port and base path
NAP_PORT=8080 NAP_BASE_PATH=/path/to/universes nap-server
```

#### API Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/resolve/{universe}/{entity_type}/{entity_id}` | Resolve a manifest |
| `GET` | `/resolve/{universe}/{entity_type}/{entity_id}?branch=canon` | Resolve at a branch |
| `POST` | `/commit/{universe}/{entity_type}/{entity_id}` | Commit changes |
| `GET` | `/history/{universe}/{entity_type}/{entity_id}` | Get commit history |
| `GET` | `/universes` | List all universes |
| `GET` | `/universes/{universe}/entities` | List entities in a universe |
| `GET` | `/health` | Health check |

Resolution query parameters: `branch`, `commit`, `tag`, `path` (subtree query).

#### Examples

```bash
# Resolve a manifest
curl http://localhost:3100/resolve/starwars/character/lukeskywalker

# With branch selector
curl "http://localhost:3100/resolve/starwars/character/lukeskywalker?branch=canon"

# Subtree query via API
curl "http://localhost:3100/resolve/starwars/character/lukeskywalker?path=properties.species"

# List everything
curl http://localhost:3100/universes
curl http://localhost:3100/universes/starwars/entities?type=character

# Commit changes via API
curl -X POST http://localhost:3100/commit/starwars/character/lukeskywalker \
  -H "Content-Type: application/json" \
  -d '{
    "message": "update species",
    "author": "dev@studio.com",
    "properties": {
      "species": "human"
    }
  }'
```

**Use case — game engine integration:** A Unity or Unreal plugin queries the NAP server at build time to populate character data, spawn points, and prop manifests:

```csharp
// Unity example — fetch character data at editor time
string json = new WebClient().DownloadString(
    "http://localhost:3100/resolve/starwars/character/lukeskywalker?path=properties"
);
CharacterData data = JsonUtility.FromJson<CharacterData>(json);
```

**Use case — web dashboard:** A worldbuilding wiki resolves manifests on the fly to render character sheets, location maps, and scene timelines:

```javascript
// React example — resolve character for profile page
const { data } = await fetch(
  `/api/resolve/${universe}/character/${characterId}`
);
```

### 3.3 Commit History as Audit Trail

Every change is content-addressed and versioned. Trace exactly how a character evolved and who made each change:

```bash
# View commit history
nap history nap://starwars/character/lukeskywalker -n 20
# a72c9f3 2026-06-09T20:15:00Z — set species to human — alice
# b83d1a2 2026-06-09T20:10:00Z — set homeworld — alice
# c94e2b1 2026-06-09T20:05:00Z — added reference_image — bob
# d05f3c0 2026-06-09T20:00:00Z — Create character 'Luke Skywalker' — alice

# Resolve what the manifest looked like at a specific commit
nap resolve nap://starwars/character/lukeskywalker --commit b83d1a2

# View history via API
curl http://localhost:3100/history/starwars/character/lukeskywalker
```

**Use case — canon dispute resolution:** When two writers disagree on whether Luke's hair color changed between drafts, the commit log shows exactly when and by whom it was modified:

```bash
nap history nap://starwars/character/lukeskywalker | grep "hair"
# f7a2b1c 2026-06-08T14:30:00Z — set hair_color to brown — bob
# e8d3c2b 2026-06-07T09:15:00Z — set hair_color to blond — alice
```

**Use case — rollback:** Revert a character to a known good state:

```bash
git -C starwars revert b83d1a2
```

### 3.4 Cross-Universe Discovery

Discover what universes and entities are available:

```bash
# List all universes in the base directory
nap list
# nap://starwars/
# nap://toystory/
# nap://middleearth/

# List all entities in a universe
nap list starwars
# character:
#   nap://starwars/character/lukeskywalker
#   nap://starwars/character/darthvader
# location:
#   nap://starwars/location/tatooine
#   nap://starwars/location/deathstar
# scene:
#   nap://starwars/scene/cantina

# Filter by type
nap list starwars -t character
# character:
#   nap://starwars/character/lukeskywalker
#   nap://starwars/character/darthvader
```

---

## 4. World-Building

### 4.1 Full World-Building Workflow

Build out a universe from scratch with a structured workflow:

```bash
# Step 1: Initialize the universe
nap init myworld

# Step 2: Define the world metadata
nap set nap://myworld/world/myworld canon_level "canon"
nap set nap://myworld/world/myworld timeline "Age of Discovery"
nap set nap://myworld/world/myworld theme "exploration vs exploitation"

# Step 3: Create factions / groups as properties on the world
nap set nap://myworld/world/myworld factions \
  '["The Commonwealth", "The Outer Rim Syndicate", "The Core"]'

# Step 4: Create characters
nap create character captain-rex -u myworld -n "Captain Rex"
nap create character admiral-torres -u myworld -n "Admiral Torres"
nap create character lyra -u myworld -n "Lyra"

# Step 5: Flesh out character properties
nap set nap://myworld/character/captain-rex rank "Captain"
nap set nap://myworld/character/captain-rex affiliation "The Commonwealth"
nap set nap://myworld/character/captain-rex ship "nap://myworld/prop/valkyrie"

# Step 6: Create locations
nap create location kyra-prime -u myworld -n "Kyra Prime"
nap set nap://myworld/location/kyra-prime type "colonial_capital"
nap set nap://myworld/location/kyra-prime controlled_by "The Commonwealth"

# Step 7: Create scenes that connect everything
nap create scene first-contact -u myworld -n "First Contact"
nap set nap://myworld/scene/first-contact setting "nap://myworld/location/kyra-prime"
nap set nap://myworld/scene/first-contact participants \
  '[
    "nap://myworld/character/captain-rex",
    "nap://myworld/character/lyra"
  ]'
nap set nap://myworld/scene/first-contact mood "tense"
nap set nap://myworld/scene/first-contact outcome "alliance_formed"

# Step 8: Add reference images
nap add-repr nap://myworld/character/captain-rex reference_image \
  ./concept/rex.png --format png

# Step 9: Commit everything
nap commit myworld -m "Complete first act world-building" -a "writer@studio.com"
```

### 4.2 Entity Types Reference

| Type | URI Pattern | What it models | Example properties |
|---|---|---|---|
| `world` | `nap://<name>/world/<name>` | The universe itself — rules, canon level, metadata | `canon_level`, `timeline`, `theme`, `factions` |
| `character` | `nap://<name>/character/<id>` | Persistent character with identity across scenes | `homeworld`, `species`, `affiliation`, `master`, `apprentice` |
| `location` | `nap://<name>/location/<id>` | Spatial setting | `climate`, `type`, `controlled_by`, `population` |
| `scene` | `nap://<name>/scene/<id>` | Narrative moment — participants, timeline, events | `setting`, `participants`, `mood`, `outcome`, `time_of_day` |
| `prop` | `nap://<name>/prop/<id>` | Physical object with materials, variants, ownership | `owner`, `material`, `weight`, `color` |

**World manifest** (`universe.yaml` — created automatically):

```yaml
id: nap://myworld/world/myworld
name: myworld Universe
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

The `references` field builds a directed graph between entities. This enables rich queries across your universe:

```bash
# Character → scenes they appear in
nap set nap://starwars/character/lukeskywalker references.appears_in \
  '["nap://starwars/scene/cantina", "nap://starwars/scene/trenchrun"]'

# Location → scenes set there
nap set nap://starwars/location/tatooine references.appears_in \
  '["nap://starwars/scene/cantina"]'

# Character → relationships
nap set nap://starwars/character/lukeskywalker references.relationships \
  '[
    {"target": "nap://starwars/character/darthvader", "type": "father"},
    {"target": "nap://starwars/character/leia", "type": "sister"},
    {"target": "nap://starwars/character/hansolo", "type": "friend"}
  ]'

# Prop → owner
nap set nap://toystory/prop/andy-hat references.owner "nap://toystory/character/andy"
```

**Graph traversal examples:**

```bash
# Find all scenes a character appears in
nap query nap://starwars/character/lukeskywalker references.appears_in

# Find all characters that visit a location
# (resolve scene participants for each scene set at the location)
nap resolve nap://starwars/scene/cantina#properties.participants

# Find a character's relationships
nap query nap://starwars/character/lukeskywalker references.relationships -f json
```

### 4.4 Multi-Universe Portfolio

Manage multiple fictional worlds under one resolver:

```bash
# Create universes side by side
nap init starwars
nap init toystory
nap init middleearth

# Every universe is independently addressable
nap create character buzzlightyear -u toystory -n "Buzz Lightyear"
nap create location andysroom -u toystory -n "Andy's Room"
nap create character frodo -u middleearth -n "Frodo Baggins"
nap create location theshire -u middleearth -n "The Shire"

# List all universes
nap list
# nap://starwars/
# nap://toystory/
# nap://middleearth/

# Each universe has its own Git history, branches, tags
nap branch middleearth canon
nap tag middleearth fellowship-of-the-ring
```

**Universe directory layout:**

```
base_dir/
├── starwars/              ← independent Git repo
│   ├── .nap/config.yaml
│   ├── universe.yaml
│   ├── characters/
│   ├── locations/
│   ├── scenes/
│   └── props/
├── toystory/              ← independent Git repo
│   ├── .nap/config.yaml
│   ├── universe.yaml
│   ├── characters/
│   ├── locations/
│   ├── scenes/
│   └── props/
└── middleearth/           ← independent Git repo
    ├── .nap/config.yaml
    ├── universe.yaml
    ├── characters/
    ├── locations/
    ├── scenes/
    └── props/
```

### 4.5 Generative AI Provenance

Track every AI-generated asset with full lineage:

```bash
# After generating a character design with Midjourney
nap set nap://starwars/character/lukeskywalker provenance.model "midjourney-v6"
nap set nap://starwars/character/lukeskywalker provenance.seed "8675309"
nap set nap://starwars/character/lukeskywalker provenance.prompt_hash "sha256:abc123..."
nap set nap://starwars/character/lukeskywalker provenance.parameters.stylize "1000"

# After iterating with an LLM
nap set nap://starwars/character/lukeskywalker provenance.derived_from \
  "nap://starwars/character/lukeskywalker/v1"

# Set the creation timestamp
nap set nap://starwars/character/lukeskywalker provenance.created_at "2026-06-09T20:00:00Z"
```

The provenance block captures complete generative lineage:

```yaml
provenance:
  model: "midjourney-v6"
  prompt_hash: "sha256:abc123..."
  seed: "8675309"
  parameters:
    stylize: "1000"
    chaos: "20"
  derived_from: "nap://starwars/character/lukeskywalker/v1"
  created_at: "2026-06-09T20:00:00Z"
```

**Use case — rights & attribution:** When a model is deprecated or a license changes, identify every asset generated with it:

```bash
# Find all entities generated with a specific model
nap query nap://starwars/character/lukeskywalker provenance.model
# → midjourney-v6
```

**Use case — reproducibility:** Given the same model, prompt hash, seed, and parameters, you can regenerate an identical asset for A/B testing or re-rendering at higher resolution.

---

## Pro Tips

### Composition

```bash
# Initialize a universe in any directory
nap init myuniverse -d /path/to/shared/universes
nap init myuniverse -d ~/Dropbox/TeamWorldbuilding

# Point the resolver at a remote path
nap -d /mnt/nas/universes list
nap -d /mnt/nas/universes resolve nap://starwars/character/lukeskywalker
```

### Output Formats

```bash
# JSON for programmatic consumption
nap resolve nap://starwars/character/lukeskywalker -f json | jq '.properties'

# YAML for human review and editing
nap resolve nap://starwars/character/lukeskywalker -f yaml
```

### Debugging

```bash
# Verbose mode shows tracing output
nap -v resolve nap://starwars/character/lukeskywalker

# Check server health
curl http://localhost:3100/health
# {"status":"ok","protocol":"NAP","version":"0.1.0"}
```

### Quick Bootstrap a New Universe

```bash
# One-liner to seed a new universe
nap init mynovel \
  && nap create character hero -u mynovel -n "The Hero" \
  && nap create location village -u mynovel -n "Home Village" \
  && nap set nap://mynovel/character/hero homeworld "nap://mynovel/location/village" \
  && nap set nap://mynovel/character/hero archetype "reluctant hero"
```

### Working Without Git

While NAP uses Git for version control, all of your data is plain YAML files. You can:

- Edit manifests directly in any text editor
- Version them with any VCS (Fossil, Mercurial, Jujutsu)
- Sync them via Dropbox, Google Drive, or any file sync tool
- Process them with any YAML toolchain

---

## Repository Layout Reference

```
starwars/                    ← universe root (Git repo)
├── .nap/
│   └── config.yaml          ← NAP repository configuration
├── universe.yaml            ← world manifest
├── characters/
│   ├── lukeskywalker.yaml
│   └── darthvader.yaml
├── locations/
│   ├── tatooine.yaml
│   └── deathstar.yaml
├── scenes/
│   └── cantina.yaml
└── props/
```

---

## CLI Command Reference

| Command | Description |
|---|---|
| `nap init <universe>` | Initialize a new universe repository |
| `nap create <type> <id> -u <universe> -n <name>` | Create a new entity manifest |
| `nap resolve <uri>` | Resolve a NAP URI to a manifest or subtree |
| `nap query <uri> <path>` | Query a subtree from a manifest |
| `nap set <uri> <key> <value>` | Set a property on an entity |
| `nap add-repr <uri> <key> <file> --format <fmt>` | Add a content-addressed representation |
| `nap commit <universe> -m <message>` | Commit changes to the VCS |
| `nap history <uri>` | View commit history for an entity |
| `nap list [universe]` | List universes or entities |
| `nap branch <universe> [name]` | Create or list branches |
| `nap tag <universe> [name]` | Create or list tags |
| `nap sign <uri>` | Sign a manifest (stub in v0) |
| `nap verify <uri>` | Verify a manifest signature (stub in v0) |

Global options: `-d/--base-dir <path>`, `-v/--verbose`, `-f/--format <yaml|json>`

---

## Design Principles

- **Manifest is current state. History is external.** Manifests store only `head` — a pointer to the latest commit. Full history lives in the VCS, preventing unbounded manifest growth.
- **Version/branch/tag are NEVER in the URI.** They are orthogonal selectors passed alongside the URI (mirrors Git, OCI, package managers).
- **Content-address everything.** Every representation is identified by its SHA-256 hash. Manifests are content-hashable for signing and verification.
- **Subtree queries are first-class.** AI systems, CLI tools, and HTTP clients all use the same query engine. Fragment queries enable efficient data access without fetching entire manifests.

---

*NAP is in v0 (prototype). The core data model and resolution engine are functional. Advanced features (signing, verification, distributed resolution) are planned for future iterations.*
