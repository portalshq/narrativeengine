## Installation

### Installation Script

```bash
curl -fsSL https://github.com/portalshq/narrativeengine/releases/latest/download/install.sh | bash
```

The installation script installs both `nap` and `nap-mcp-server`. The MCP server is dormant by default; agent clients start it on demand over stdio so sandboxed agents can use NAP through host-side CLI proxy calls.

### Skills Install

Install these skills to use NAP with agent workflows, including entity-aware prompts, generation templates, and the resolve/update steps that keep character and scene output consistent.

```bash
npx skills add portalshq/narrativeengine
```

<!-- ### CLI & Server (Rust — compile from source)

```bash
git clone https://github.com/cinematiccanvas/nap.git
cd nap
cargo build --release

# Binaries land in target/release/
#   nap          — CLI tool
#   nap-server   — HTTP resolver server
```

### Python SDK (prebuilt wheel, no Rust needed)

```bash
pip install narrativeengine
```

```python
from narrativeengine import create_block, generate_candidate, render_lore_summary

block = create_block("char-1", "A brave adventurer")
candidate = generate_candidate(block)
```

### TypeScript SDK (prebuilt binary, no Rust needed)

```bash
npm install @portalshq/narrativeengine
```

```typescript
import { createBlock } from "@portalshq/narrativeengine";

const block = createBlock("char-1", "A brave adventurer");
``` -->

---

## Quick Start

```bash
# Initialize a repository (prompts for provider on first run)
nap init toystory

# Initialize with local provider
nap init toystory --provider local

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
nap init toystory

# See what you created
ls toystory/
# → .nap/  repository.yaml  characters/  locations/  scenes/  props/
```

### Create & Inspect Entities

```bash
# Create a character
nap create character woody -u toystory -n "Woody"

# Create a location
nap create location andys-room -u toystory -n "Andy's Room"

# Set properties
nap set nap://toystory/character/woody toy_type human
nap set nap://toystory/character/woody homeworld "nap://toystory/location/andys-room"

# Resolve a manifest
nap resolve nap://toystory/character/woody

# Query a specific field
nap resolve nap://toystory/character/woody#properties.toy_type
# → human

# Query a subtree
nap query nap://toystory/character/woody properties
```

### Version Control

```bash
# View commit history
nap history nap://toystory/character/woody

# Create branches
nap branch toystory canon

# Sync with remote
nap sync toystory

# Publish to remote
nap publish toystory
```

### Output Formats

```bash
nap resolve nap://toystory/character/woody -f json
nap resolve nap://toystory/character/woody -f yaml
```
