---
name: px-resolve
description: Create PX entities, resolve PX URIs, query entity context, and establish active entity continuity so later refinements automatically persist through px-update.
---

# PX Resolve
 
Use this skill to create entities, resolve PX URIs, and gather entity context for creative workflows.
 
## When to Apply
 
Reference these guidelines when:
- Creating new entities (e.g., characters, locations, items, events)
- Resolving PX URIs into manifests
- Querying subtree data for creative workflows

## Core Commands
 
Create an entity:
 
```bash
px create character atlas -u bears -n "Atlas"
```
 
Resolve a manifest:
 
```bash
px resolve px://bears/character/atlas --branch main
```
 
Query a subtree:
 
```bash
px query px://bears/character/atlas properties
```
 
## Entity Creation
 
When creating a new entity:
 
1. Create the entity on the branch the user is working from (default `main` if none was specified — see "Target Branch" below).
2. Report the exact URI.
3. Establish active task context: URI, repository, entity type, entity ID, target branch, and default revision branch.
4. Create or switch to the revision branch:
   ```text
   revision-<entity-type>-<entity-id>
   ```
 
5. If the creation turn also generates a visual, text, audio, or other representation, immediately use `px-update` to commit that first accepted revision on the revision branch.

## Target Branch
 
Establish the **target branch** — the branch accepted revisions will eventually promote to — at creation/first-resolve time, and carry it forward for the rest of the task:
 
1. If the user named a branch for this work, that's the target.
2. Otherwise the branch the entity is created on or first resolved from is the target.
3. Otherwise default to `main`.
`px-update` uses this value for every promotion; it is not always `main`.
 
## Active Entity Continuity
 
After an entity URI is established, later turns that refine the same entity are continuity work. They must trigger `px-update` even if the user does not say "PX", "save", "commit", or the URI again.
 
Carry forward stable representation keys and identity constraints. Examples:
 
- `character_sheet`
- `face_sheet`
- `portrait`
- `reference_image`
- `voice_reference`

## Generation Context
 
Before generating from an entity:
 
1. Resolve the entity explicitly from the relevant branch (target branch for canonical state, revision branch for iterative work).
2. Gather properties that affect identity, narrative role, style, behavior, continuity, and exclusions.
3. Gather relevant `representations` and `references`.
4. Treat image/video/audio representations as source-of-truth for observable appearance or sound. Text properties support and constrain them.
5. Inspect flexible negative-constraint keys such as `negative_constraints`, `exclusions`, `avoid`, `forbidden`, or project-specific equivalents.
6. Keep multi-entity context separated so attributes do not bleed between entities.

## Branch Semantics
 
Resolve from the target branch for canonical state.
 
Resolve from `revision-<entity-type>-<entity-id>` for iterative work.
 
Use explicit `--branch` or MCP-equivalent arguments. Do not rely on whichever branch happens to be checked out.
 
Do not store VCS branch-head data in manifests. Branch heads and commit history belong to PX/Lore version control.
 
## Guardrails

Checking the current workspace is not required for this skill. Px usually stores all repos in a centralized directory unless configured otherwise.


# PX CLI Reference
The `px` command-line interface (v0.8.21) provides tools for creating, resolving, and managing narrative resources using the PX protocol.


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
