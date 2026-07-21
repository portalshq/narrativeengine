---
generated: "true"
generator: nap-docgen
version: 0.4.5
git_sha: fa06fff
source: clap
---


# NAP CLI Reference
The `nap` command-line interface (v0.4.5) provides tools for creating, resolving, and managing narrative resources using the Narrative Addressing Protocol.


## Command Overview

| Command | Description |
|---|---|


## Global Options

| Flag | Description | Default |
|---|---|---|
| -d, --base-dir <BASE\_DIR> |  |  |
| -v, --verbose <VERBOSE> |  |  |


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

