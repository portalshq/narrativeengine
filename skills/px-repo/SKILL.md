---
name: px-repo
description: Initialize PX repositories, clone/pull repositories, and create branches at the repository level. Use for repository-lifecycle operations (px init, px pull, px branch) — not for creating or revising individual entities; see px-resolve and px-update for those.
metadata:
  author: portals
  version: "0.8.18"
---

# PX Skill: Repository Management
 
A repository is the top-level container that holds entities (characters, locations, assets, etc.) and their PX/Lore version history.
 
## When to Apply
 
Reference these guidelines when:
- Initializing a new PX repository
- Cloning or pulling an existing repository
- Creating a new branch at the repository level
For creating or resolving individual entities, use `px-resolve`. For revising entity content and persisting iterations, use `px-update`.
 
## Core Commands
 
* **Initialize:** `px init <universe_name>` — creates a directory with a `.px/` config folder, a `repository.yaml` manifest, and subdirectories per entity type.
  * Example: `px init toystory`
* **Branch:** `px branch <universe_name> <branch_name>` — creates a new timeline/snapshot.
  * Example: `px branch toystory classic`
* **Clone/pull:** `px pull <remote> <universe_name>` — clones or pulls a repository from a remote.

## Guardrails
 
 Unless the user explicitly requests a different provider or storage location:
- Run px init <repository> with no --provider and no --base-dir.
- Preserve the configured provider and default PX directory.
- Never infer --provider local from an example.
- Never choose a workspace-local --base-dir merely to isolate a repository.
Use --provider only when the user explicitly requests a provider change.
Use --base-dir only when the user explicitly names a storage location.

* **No tagging.** Do not use `px tag` or append tags to URIs — Lore VCS has no native tag support. Branches are the only mechanism for human-readable names on a revision point.


# PX CLI Reference
The `px` command-line interface (v0.8.18) provides tools for creating, resolving, and managing narrative resources using the PX protocol.


## Command Overview

| Command | Description |
|---|---|
| [\`px add\`](docs/generated/commands/add.md) | Add a file representation to an entity manifest |
| [\`px auth\`](docs/generated/commands/auth.md) | Manage secure Portals Cloud authentication |
| [\`px backend\`](docs/generated/commands/backend.md) | Configure or inspect the version-control backend |
| [\`px branch\`](docs/generated/commands/branch.md) | Create or list branches |
| [\`px choose\`](docs/generated/commands/choose.md) | Choose backend provider |
| [\`px commit\`](docs/generated/commands/commit.md) | Commit changes to a repository repository |
| [\`px content-hash\`](docs/generated/commands/content-hash.md) | Compute the BLAKE3 content hash of a file |
| [\`px create\`](docs/generated/commands/create.md) | Create a new entity manifest |
| [\`px diff\`](docs/generated/commands/diff.md) | Show diff between two manifest files or versions |
| [\`px doctor\`](docs/generated/commands/doctor.md) | Run diagnostics and repair |
| [\`px head-hash\`](docs/generated/commands/head-hash.md) | Show the current HEAD commit hash |
| [\`px history\`](docs/generated/commands/history.md) | View commit history for an entity |
| [\`px init\`](docs/generated/commands/init.md) | Initialize a repository repository and/or configure the backend provider |
| [\`px install\`](docs/generated/commands/install.md) | Install required dependencies |
| [\`px list\`](docs/generated/commands/list.md) | List repositories or entities within a repository |
| [\`px merge\`](docs/generated/commands/merge.md) | Three-way merge of JSON/YAML values |
| [\`px presign\`](docs/generated/commands/presign.md) | Create a time-limited public URL for a committed representation |
| [\`px publish\`](docs/generated/commands/publish.md) | Publish changes to remote |
| [\`px pull\`](docs/generated/commands/pull.md) | Clone or pull a repository from a remote |
| [\`px push\`](docs/generated/commands/push.md) | Push the current branch to its configured upstream remote |
| [\`px query\`](docs/generated/commands/query.md) | Query a subtree from a manifest |
| [\`px remote\`](docs/generated/commands/remote.md) | Manage remotes on a repository |
| [\`px resolve\`](docs/generated/commands/resolve.md) | Resolve a PX URI to its manifest or a subtree |
| [\`px revert\`](docs/generated/commands/revert.md) | Revert a commit by hash (undoes all changes in that commit) |
| [\`px schema\`](docs/generated/commands/schema.md) | Print a JSON Schema for manifest or commit types |
| [\`px set\`](docs/generated/commands/set.md) | Set a property on an entity manifest |
| [\`px sign\`](docs/generated/commands/sign.md) | Sign a manifest (stub for v0) |
| [\`px status\`](docs/generated/commands/status.md) | Show system status |
| [\`px switch\`](docs/generated/commands/switch.md) | Switch to a branch |
| [\`px sync\`](docs/generated/commands/sync.md) | Sync with remote |
| [\`px validate\`](docs/generated/commands/validate.md) | Validate a manifest against the PX schema |
| [\`px verify\`](docs/generated/commands/verify.md) | Verify a manifest signature (stub for v0) |


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




## MCP Server

The standard PX installer bundles the native `px-mcp-server` binary with `px`. If the MCP command is missing or broken, rerun the standard PX installer from a host shell.

The MCP server is not a daemon; agent clients start it on demand over stdio, and it proxies tool calls to the host `px` CLI.

## Agent Sandbox Integration

When running inside a sandboxed environment (e.g., Codex) without outbound network access, use MCP tools instead of shelling out to the `px` CLI directly. The MCP server runs on the host machine, starts only when the agent/MCP client launches it over stdio, and proxies tool calls to the host `px` CLI.

Direct `px` CLI examples in this skill are for humans, host-local shells, and non-sandboxed scripts. In an agent sandbox, use the MCP tools for any operation that may need Lore/cloud/network access.

## Available MCP Tools

All px CLI commands are available as MCP tools with `px_` prefix. For example:
- `px resolve` -> `px_resolve` tool
- `px create` -> `px_create` tool
- `px set` -> `px_set` tool

Prefer MCP tools over shell commands when in a sandbox.



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


