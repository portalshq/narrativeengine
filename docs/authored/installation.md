## Installation

### Installation Script

```bash
curl -fsSL https://github.com/portalshq/narrativeengine/releases/latest/download/install.sh | bash
```

The installation script installs both `px` and `px-mcp-server`. The MCP server is dormant by default; agent clients start it on demand over stdio so sandboxed agents can use PX through host-side CLI proxy calls.

### Skills Install

Install these skills to use PX with agent workflows, including entity-aware prompts, generation templates, and the resolve/update steps that keep character and scene output consistent.

```bash
npx skills add portalshq/narrativeengine
```

<!-- ### CLI & Server (Rust — compile from source)

```bash
git clone https://github.com/cinematiccanvas/px.git
cd px
cargo build --release

# Binaries land in target/release/
#   px          — CLI tool
#   px-server   — HTTP resolver server
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
px init toystory

# Initialize with local provider
px init toystory --provider local

# Configure provider only (no repository)
px init --provider local

# Initialize with remote provider
px init --provider remote --remote-url lore://127.0.0.1:41337 --workspace-id my-workspace

# Initialize with Portals Cloud
px auth login
px init --provider portals-cloud

# Inspect or clear the OS-keyring-backed session
px auth status
px auth logout

# Check system status
px status

# Run diagnostics
px doctor

# Run diagnostics with auto-repair
px doctor --repair
```

Portals Cloud uses `grpcs://lore.portals.works` on standard TLS port 443. Login is
the only interactive VCS step; repository operations remain noninteractive and
return an actionable `px auth login` error when credentials are missing or
expired. Lore automatically exchanges the eight-hour login session for a
five-minute token scoped to the single repository used by init, clone, push,
pull, sync, publish, and locking. CI uses a revocable service-account API key
exchange; do not store long-lived bearer tokens in CI variables.

`px install lore` installs the exact `portalshq/lore` release compiled into
that Px version. It downloads the installer from the same release tag,
verifies its pinned SHA-256 before execution, and explicitly selects the
Portals fork. It never executes the mutable `main` installer or silently falls
back to an upstream Lore binary. Production release metadata binds this Lore
client version to Px's signed checksum manifest.

CI reads the API key from its secret store and passes it to Lore over stdin,
so the secret is absent from process arguments and command logs:

```bash
export PORTALS_CLOUD_API_KEY="${CI_PORTALS_CLOUD_API_KEY}"
px auth login --api-key
```

Use `--api-key-env NAME` to select a different secret environment variable.

### Create a Repository

```bash
# Initialize a new repository
px init toystory

# See what you created
ls toystory/
# → .px/  repository.yaml  characters/  locations/  scenes/  props/
```

### Create & Inspect Entities

```bash
# Create a character
px create character woody -u toystory -n "Woody"

# Create a location
px create location andys-room -u toystory -n "Andy's Room"

# Set properties
px set px://toystory/character/woody toy_type human
px set px://toystory/character/woody homeworld "px://toystory/location/andys-room"

# Resolve a manifest
px resolve px://toystory/character/woody

# Query a specific field
px resolve px://toystory/character/woody#properties.toy_type
# → human

# Query a subtree
px query px://toystory/character/woody properties
```

### Version Control

```bash
# View commit history
px history px://toystory/character/woody

# Create branches
px branch toystory canon

# Sync with remote
px sync toystory

# Publish to remote
px publish toystory
```

### Output Formats

```bash
px resolve px://toystory/character/woody -f json
px resolve px://toystory/character/woody -f yaml
```
