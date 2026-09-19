---
name: px-cli-reference
description: Read-only reference for px CLI syntax, for humans asking what the command for something is (e.g. how to resolve a URI from a host shell). Never use for execution — agents MUST still execute all PX operations via px-mcp-server (see px-repo, px-resolve, px-update).
metadata:
  author: portals
  version: "0.8.25"
---

# PX CLI Reference (read-only)

## When to Apply

Reference these guidelines when a human asks about `px` CLI syntax: what command to run in a host shell, what flags a command accepts, or how a shell example works.

## When NOT to Apply

Never use this skill for agentic execution. All PX operations performed by an agent MUST go through the `px-mcp-server` MCP tools (`px_<command>`) per `px-repo`, `px-resolve`, and `px-update`. The `px` CLI is not available for agentic use — do not shell out, even when the exact CLI equivalent is documented below. If the task is to initialize, create, resolve, update, branch, pull, or commit anything, route to the corresponding execution skill instead.

## Primitives CLI Examples

Command-line examples for the [core primitives](./primitives.md), intended for humans working in a host shell. Agents must not execute these — agentic execution goes exclusively through `px-mcp-server` (see `px-repo`, `px-resolve`, `px-update`).

```bash
px create scene pizza-planet -u toystory -n "Pizza Planet"
px add px://toystory/scene/pizza-planet clip-01 ./pizza-planet-clip-01.mp4 --format mp4 -m "Add pizza-planet scene clip"
```

```bash
px resolve px://toystory/scene/pizza-planet --provenance
```



# PX CLI Reference
The `px` command-line interface (v0.8.25) provides tools for creating, resolving, and managing narrative resources using the PX protocol.


## Command Overview

| Command | Description |
|---|---|
| [\`px add\`](docs/generated/commands/add.md) | Add a file representation to an entity manifest |
| [\`px auth\`](docs/generated/commands/auth.md) | Manage secure authentication for the configured Lore provider |
| [\`px branch\`](docs/generated/commands/branch.md) | Create or list branches |
| [\`px commit\`](docs/generated/commands/commit.md) | Commit changes to a repository repository |
| [\`px completions\`](docs/generated/commands/completions.md) | Generate shell completions for \`px\` |
| [\`px configure\`](docs/generated/commands/configure.md) | Configure version-control backend (unified: replaces \`px choose\` + \`px backend\`) |
| [\`px content-hash\`](docs/generated/commands/content-hash.md) | Compute the BLAKE3 content hash of a file |
| [\`px create\`](docs/generated/commands/create.md) | Create a new entity manifest |
| [\`px diff\`](docs/generated/commands/diff.md) | Show diff between two manifest files or versions |
| [\`px doctor\`](docs/generated/commands/doctor.md) | Run diagnostics and repair |
| [\`px head\`](docs/generated/commands/head.md) | Show the current HEAD commit hash |
| [\`px history\`](docs/generated/commands/history.md) | View commit history for an entity |
| [\`px init\`](docs/generated/commands/init.md) | Initialize a repository repository and/or configure the backend provider |
| [\`px install\`](docs/generated/commands/install.md) | Install required dependencies |
| [\`px list\`](docs/generated/commands/list.md) | List repositories or entities within a repository |
| [\`px merge\`](docs/generated/commands/merge.md) | Three-way merge of JSON/YAML values |
| [\`px presign\`](docs/generated/commands/presign.md) | Create a time-limited public URL for a committed representation |
| [\`px pull\`](docs/generated/commands/pull.md) | Clone or pull a repository from a remote |
| [\`px push\`](docs/generated/commands/push.md) | Push the current branch to its configured upstream remote |
| [\`px query\`](docs/generated/commands/query.md) | Query a subtree from a manifest |
| [\`px remote\`](docs/generated/commands/remote.md) | Manage remotes on a repository |
| [\`px resolve\`](docs/generated/commands/resolve.md) | Resolve a PX URI to its manifest or a subtree |
| [\`px revert\`](docs/generated/commands/revert.md) | Revert a commit by hash (undoes all changes in that commit) |
| [\`px schema\`](docs/generated/commands/schema.md) | Print a JSON Schema for manifest or commit types |
| [\`px set\`](docs/generated/commands/set.md) | Set a property on an entity manifest |
| [\`px status\`](docs/generated/commands/status.md) | Show system status |
| [\`px switch\`](docs/generated/commands/switch.md) | Switch to a branch |
| [\`px sync\`](docs/generated/commands/sync.md) | Sync with remote |
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




# Global Options
These options are available on all `px` commands.


| Flag | Description | Default |
|---|---|---|
|     --local | Resolve repository reads from an explicitly checked-out local working tree |  |
|     --remote | Resolve repository reads through the configured Lore server (the default) |  |
| -d, --base-dir | Base directory for repository repositories. Defaults to $PX\_DIR, or ~/.px if unset |  |
| -v, --verbose | Enable verbose debug logging |  |




# Environment Variables
The following environment variables are recognized by `px`.


| Variable | Description |
|---|---|
| PX\_OUTPUT | Override for --format |


