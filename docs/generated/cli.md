---
generated: "true"
generator: px-docgen
version: 0.9.0
source: clap
---


# PX CLI Reference
The `px` command-line interface (v0.9.0) provides tools for creating, resolving, and managing narrative resources using the PX protocol.


## Command Overview

| Command | Description |
|---|---|
| [\`px add\`](docs/generated/commands/add.md) | Add a file representation to an entity manifest |
| [\`px auth\`](docs/generated/commands/auth.md) | Manage secure authentication for the configured Lore provider |
| [\`px branch\`](docs/generated/commands/branch.md) | Create or list branches |
| [\`px commit\`](docs/generated/commands/commit.md) | Commit all repository changes, or only one entity when given its URI |
| [\`px configure\`](docs/generated/commands/configure.md) | Configure version-control backend |
| [\`px content-hash\`](docs/generated/commands/content-hash.md) | Compute the BLAKE3 content hash of a file |
| [\`px create\`](docs/generated/commands/create.md) | Create a new entity manifest |
| [\`px diff\`](docs/generated/commands/diff.md) | Show a manifest diff for an entity URI |
| [\`px doctor\`](docs/generated/commands/doctor.md) | Run diagnostics and repair |
| [\`px head\`](docs/generated/commands/head.md) | Show the current HEAD commit hash |
| [\`px history\`](docs/generated/commands/history.md) | View commit history for an entity or repository file |
| [\`px init\`](docs/generated/commands/init.md) | Initialize a repository repository and/or configure the backend provider |
| [\`px install\`](docs/generated/commands/install.md) | Install required dependencies |
| [\`px list\`](docs/generated/commands/list.md) | List repositories or entities within a repository |
| [\`px merge\`](docs/generated/commands/merge.md) | Three-way merge of JSON/YAML values |
| [\`px presign\`](docs/generated/commands/presign.md) | Create a time-limited public URL for a committed representation |
| [\`px pull\`](docs/generated/commands/pull.md) | Clone or pull PX manifests from a remote (representation files stay remote) |
| [\`px push\`](docs/generated/commands/push.md) | Push the current branch to its configured upstream remote |
| [\`px remote\`](docs/generated/commands/remote.md) | Manage remotes on a repository |
| [\`px resolve\`](docs/generated/commands/resolve.md) | Resolve a PX URI to its manifest or a subtree |
| [\`px revert\`](docs/generated/commands/revert.md) | Revert a commit by hash (undoes all changes in that commit) |
| [\`px schema\`](docs/generated/commands/schema.md) | Print a JSON Schema for manifest or commit types |
| [\`px set\`](docs/generated/commands/set.md) | Set one or more properties on an entity manifest |
| [\`px status\`](docs/generated/commands/status.md) | Show system status, or working-tree status for one repository |
| [\`px switch\`](docs/generated/commands/switch.md) | Switch to a branch |
| [\`px sync\`](docs/generated/commands/sync.md) | Fetch remote manifests and push local commits |
| [\`px unset\`](docs/generated/commands/unset.md) | Remove one or more properties or representations from an entity manifest |
| [\`px validate\`](docs/generated/commands/validate.md) | Validate a manifest against the PX schema |


## Global Options

| Flag | Description | Default |
|---|---|---|
|     --local | Resolve repository reads from an explicitly checked-out local working tree |  |
|     --remote | Resolve repository reads through the configured Lore server (the default) |  |
| -d, --base-dir | Base directory for repository repositories. Defaults to $PX\_DIR, or ~/.px if unset |  |
| -v, --verbose | Enable verbose debug logging |  |


## Output Formats
Most commands support `--format` (`-f`) with values `yaml` (default) or `json`.

When stdout is not a terminal, JSON is used automatically. Override with `$PX_OUTPUT`.


## Common Examples
```bash
# Initialize a repository
px init toystory

# Create an entity
px create character woody -u toystory -n "Woody"

# Resolve a manifest
px resolve px://toystory/character/woody

# Query a subtree
px query px://toystory/character/woody properties

# View commit history
px history px://toystory/character/woody
```

