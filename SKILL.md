---
name: nap
description: NAP (Narrative Addressing Protocol) — identity, addressing, resolution, and attribution for narrative resources.
metadata:
  author: portals
  version: "0.4.5"
---

# NAP Coding Assistant


# NAP CLI Reference
The `nap` command-line interface (v0.4.5) provides tools for creating, resolving, and managing narrative resources using the Narrative Addressing Protocol.


## Command Overview

| Command | Description |
|---|---|
| [\`nap add-repr\`](docs/generated/commands/add-repr.md) | Add a representation to an entity manifest |
| [\`nap branch\`](docs/generated/commands/branch.md) | Create or list branches |
| [\`nap choose\`](docs/generated/commands/choose.md) | Choose backend provider |
| [\`nap commit\`](docs/generated/commands/commit.md) | Commit changes to a universe repository |
| [\`nap content-hash\`](docs/generated/commands/content-hash.md) | Compute the SHA-256 content hash of a file |
| [\`nap create\`](docs/generated/commands/create.md) | Create a new entity manifest |
| [\`nap diff\`](docs/generated/commands/diff.md) | Show diff between two manifest files or versions |
| [\`nap doctor\`](docs/generated/commands/doctor.md) | Run diagnostics and repair |
| [\`nap head-hash\`](docs/generated/commands/head-hash.md) | Show the current HEAD commit hash |
| [\`nap history\`](docs/generated/commands/history.md) | View commit history for an entity |
| [\`nap init\`](docs/generated/commands/init.md) | Initialize a universe repository and/or configure the backend provider |
| [\`nap install\`](docs/generated/commands/install.md) | Install required dependencies |
| [\`nap list\`](docs/generated/commands/list.md) | List universes or entities within a universe |
| [\`nap merge\`](docs/generated/commands/merge.md) | Three-way merge of JSON/YAML values |
| [\`nap publish\`](docs/generated/commands/publish.md) | Publish changes to remote |
| [\`nap pull\`](docs/generated/commands/pull.md) | Clone or pull a universe from a remote |
| [\`nap push\`](docs/generated/commands/push.md) | Push the current branch to its configured upstream remote |
| [\`nap query\`](docs/generated/commands/query.md) | Query a subtree from a manifest |
| [\`nap remote\`](docs/generated/commands/remote.md) | Manage git remotes on a universe |
| [\`nap resolve\`](docs/generated/commands/resolve.md) | Resolve a NAP URI to its manifest or a subtree |
| [\`nap revert\`](docs/generated/commands/revert.md) | Revert a commit by hash (undoes all changes in that commit) |
| [\`nap schema\`](docs/generated/commands/schema.md) | Print a JSON Schema for manifest or commit types |
| [\`nap set\`](docs/generated/commands/set.md) | Set a property on an entity manifest |
| [\`nap sign\`](docs/generated/commands/sign.md) | Sign a manifest (stub for v0) |
| [\`nap status\`](docs/generated/commands/status.md) | Show system status |
| [\`nap switch\`](docs/generated/commands/switch.md) | Switch to a branch |
| [\`nap sync\`](docs/generated/commands/sync.md) | Sync with remote |
| [\`nap tag\`](docs/generated/commands/tag.md) | Create or list tags |
| [\`nap validate\`](docs/generated/commands/validate.md) | Validate a manifest against the NAP schema |
| [\`nap verify\`](docs/generated/commands/verify.md) | Verify a manifest signature (stub for v0) |


## Global Options

| Flag | Description | Default |
|---|---|---|
| -d, --base-dir <BASE\_DIR> | Base directory for universe repositories. Defaults to $NAP\_DIR, or ~/.nap if unset |  |
| -v, --verbose <VERBOSE> | Enable verbose debug logging |  |


## Output Formats
Most commands support `--format` (`-f`) with values `yaml` (default) or `json`.

When stdout is not a terminal, JSON is used automatically. Override with `$NAP_OUTPUT`.


## Common Examples
```bash
# Initialize a universe
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




# Global Options
These options are available on all `nap` commands.


| Flag | Description | Default |
|---|---|---|
| -d, --base-dir <BASE\_DIR> | Base directory for universe repositories. Defaults to $NAP\_DIR, or ~/.nap if unset |  |
| -v, --verbose <VERBOSE> | Enable verbose debug logging |  |




# Environment Variables
The following environment variables are recognized by `nap`.


| Variable | Description |
|---|---|
| NAP\_OUTPUT | Override for --format |



## Repository Guidance

A NAP universe is a Git repository containing narrative entities (characters, locations, scenes, props, groups, worlds). Each entity is a YAML manifest file identified by a `nap://` URI.

### Directory Structure

```text
.universe/
    config.yaml
universe.yaml
characters/
    lukeskywalker.yaml
    leiaorgana.yaml
locations/
    tatooine.yaml
scenes/
    cantina.yaml
```

### Key Conventions

- Every piece of content must be indexed by its BLAKE3 hash.
- Use `nap set` to modify entity properties. This creates a commit automatically.
- Use `nap resolve` to read manifests. Append `#path.to.field` for subtree queries.
- The underlying Lore VCS does not support tags. Use branches for named checkpoints.
- When piped, all output switches to JSON automatically. Use `-f yaml` to override.
- Author identifiers default to `nap-cli`. Set `-a` to record provenance.

## Entity Types

| Type | Description |
|---|---|
| `character` | Persistent character with identity across scenes |
| `location` | Spatial location within a fictional universe |
| `scene` | Narrative scene — participants, timeline, events |
| `prop` | Physical object with materials, variants, ownership |
| `group` | Mixed-media groups combining entities and assets |
| `world` | The universe itself — rules, canon, top-level metadata |
