---
name: nap-repo
description: Initialize NAP repositories, clone/pull repositories, and create branches at the repository level. Use for repository-lifecycle operations (nap init, nap pull, nap branch) — not for creating or revising individual entities; see nap-resolve and nap-update for those.
metadata:
  author: portals
  version: "0.8.3"
---

# NAP Skill: Repository Management
 
A repository is the top-level container that holds entities (characters, locations, assets, etc.) and their NAP/Lore version history.
 
## When to Apply
 
Reference these guidelines when:
- Initializing a new NAP repository
- Cloning or pulling an existing repository
- Creating a new branch at the repository level
For creating or resolving individual entities, use `nap-resolve`. For revising entity content and persisting iterations, use `nap-update`.
 
## Core Commands
 
* **Initialize:** `nap init <universe_name>` — creates a directory with a `.nap/` config folder, a `repository.yaml` manifest, and subdirectories per entity type.
  * Example: `nap init toystory`
* **Branch:** `nap branch <universe_name> <branch_name>` — creates a new timeline/snapshot.
  * Example: `nap branch toystory classic`
* **Clone/pull:** `nap pull <remote> <universe_name>` — clones or pulls a repository from a remote.

## Guardrails
 
 Unless the user explicitly requests a different provider or storage location:
- Run nap init <repository> with no --provider and no --base-dir.
- Preserve the configured provider and default NAP directory.
- Never infer --provider local from an example.
- Never choose a workspace-local --base-dir merely to isolate a repository.
Use --provider only when the user explicitly requests a provider change.
Use --base-dir only when the user explicitly names a storage location.

* **No tagging.** Do not use `nap tag` or append tags to URIs — Lore VCS has no native tag support. Branches are the only mechanism for human-readable names on a revision point.


# NAP CLI Reference
The `nap` command-line interface (v0.8.3) provides tools for creating, resolving, and managing narrative resources using the Narrative Addressing Protocol.


## Command Overview

| Command | Description |
|---|---|
| [\`nap add\`](docs/generated/commands/add.md) | Add a file representation to an entity manifest |
| [\`nap auth\`](docs/generated/commands/auth.md) | Manage secure Portals Cloud authentication |
| [\`nap backend\`](docs/generated/commands/backend.md) | Configure or inspect the version-control backend |
| [\`nap branch\`](docs/generated/commands/branch.md) | Create or list branches |
| [\`nap choose\`](docs/generated/commands/choose.md) | Choose backend provider |
| [\`nap commit\`](docs/generated/commands/commit.md) | Commit changes to a repository repository |
| [\`nap content-hash\`](docs/generated/commands/content-hash.md) | Compute the BLAKE3 content hash of a file |
| [\`nap create\`](docs/generated/commands/create.md) | Create a new entity manifest |
| [\`nap diff\`](docs/generated/commands/diff.md) | Show diff between two manifest files or versions |
| [\`nap doctor\`](docs/generated/commands/doctor.md) | Run diagnostics and repair |
| [\`nap head-hash\`](docs/generated/commands/head-hash.md) | Show the current HEAD commit hash |
| [\`nap history\`](docs/generated/commands/history.md) | View commit history for an entity |
| [\`nap init\`](docs/generated/commands/init.md) | Initialize a repository repository and/or configure the backend provider |
| [\`nap install\`](docs/generated/commands/install.md) | Install required dependencies |
| [\`nap list\`](docs/generated/commands/list.md) | List repositories or entities within a repository |
| [\`nap merge\`](docs/generated/commands/merge.md) | Three-way merge of JSON/YAML values |
| [\`nap presign\`](docs/generated/commands/presign.md) | Create a time-limited public URL for a committed representation |
| [\`nap publish\`](docs/generated/commands/publish.md) | Publish changes to remote |
| [\`nap pull\`](docs/generated/commands/pull.md) | Clone or pull a repository from a remote |
| [\`nap push\`](docs/generated/commands/push.md) | Push the current branch to its configured upstream remote |
| [\`nap query\`](docs/generated/commands/query.md) | Query a subtree from a manifest |
| [\`nap remote\`](docs/generated/commands/remote.md) | Manage remotes on a repository |
| [\`nap resolve\`](docs/generated/commands/resolve.md) | Resolve a NAP URI to its manifest or a subtree |
| [\`nap revert\`](docs/generated/commands/revert.md) | Revert a commit by hash (undoes all changes in that commit) |
| [\`nap schema\`](docs/generated/commands/schema.md) | Print a JSON Schema for manifest or commit types |
| [\`nap set\`](docs/generated/commands/set.md) | Set a property on an entity manifest |
| [\`nap sign\`](docs/generated/commands/sign.md) | Sign a manifest (stub for v0) |
| [\`nap status\`](docs/generated/commands/status.md) | Show system status |
| [\`nap switch\`](docs/generated/commands/switch.md) | Switch to a branch |
| [\`nap sync\`](docs/generated/commands/sync.md) | Sync with remote |
| [\`nap validate\`](docs/generated/commands/validate.md) | Validate a manifest against the NAP schema |
| [\`nap verify\`](docs/generated/commands/verify.md) | Verify a manifest signature (stub for v0) |


## Global Options

| Flag | Description | Default |
|---|---|---|
|     --local <LOCAL> | Resolve repository reads from an explicitly checked-out local working tree |  |
|     --remote <REMOTE> | Resolve repository reads through the configured Lore server (the default) |  |
| -d, --base-dir <BASE\_DIR> | Base directory for repository repositories. Defaults to $NAP\_DIR, or ~/.nap if unset |  |
| -v, --verbose <VERBOSE> | Enable verbose debug logging |  |


## Output Formats
Most commands support `--format` (`-f`) with values `yaml` (default) or `json`.

When stdout is not a terminal, JSON is used automatically. Override with `$NAP_OUTPUT`.


## Common Examples
```bash
# Initialize a repository
nap init toystory

# Create an entity
nap create character woody -u toystory -n "Woody"

# Resolve a manifest
nap resolve nap://toystory/character/woody

# Query a subtree
nap query nap://toystory/character/woody properties

# View commit history
nap history nap://toystory/character/woody
```




## MCP Server

The standard NAP installer bundles the native `nap-mcp-server` binary with `nap`. If the MCP command is missing or broken, rerun the standard NAP installer from a host shell. 

The MCP server is not a daemon; agent clients start it on demand over stdio, and it proxies tool calls to the host `nap` CLI.

## Agent Sandbox Integration

When running inside a sandboxed environment (e.g., Codex) without outbound network access, use MCP tools instead of shelling out to the `nap` CLI directly. The MCP server runs on the host machine, starts only when the agent/MCP client launches it over stdio, and proxies tool calls to the host `nap` CLI.

Direct `nap` CLI examples in this skill are for humans, host-local shells, and non-sandboxed scripts. In an agent sandbox, use the MCP tools for any operation that may need Lore/cloud/network access.

## Available MCP Tools

All nap CLI commands are available as MCP tools with `nap_` prefix. For example:
- `nap resolve` -> `nap_resolve` tool
- `nap create` -> `nap_create` tool
- `nap set` -> `nap_set` tool

Prefer MCP tools over shell commands when in a sandbox.



# Global Options
These options are available on all `nap` commands.


| Flag | Description | Default |
|---|---|---|
|     --local <LOCAL> | Resolve repository reads from an explicitly checked-out local working tree |  |
|     --remote <REMOTE> | Resolve repository reads through the configured Lore server (the default) |  |
| -d, --base-dir <BASE\_DIR> | Base directory for repository repositories. Defaults to $NAP\_DIR, or ~/.nap if unset |  |
| -v, --verbose <VERBOSE> | Enable verbose debug logging |  |





# Environment Variables
The following environment variables are recognized by `nap`.


| Variable | Description |
|---|---|
| NAP\_OUTPUT | Override for --format |


