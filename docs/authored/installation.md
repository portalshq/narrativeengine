## Installation

### Installation Script

```bash
curl -fsSL https://github.com/portalshq/narrativeengine/releases/download/v0.4.2/install.sh | bash
```

### CLI & Server (Rust — compile from source)

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
