---
name: nap-entity-access
description: Create new narrative entities, resolve NAP URIs into manifests, and perform subtree queries to extract specific context for creative AI workflows.
metadata:
  author: portals
  version: "0.4.5"
---

# NAP Skill: Entity Creation and Resolution

Create new narrative entities, resolve NAP URIs into manifests, and perform subtree queries to extract specific context for creative AI workflows. 

## When to Apply

Reference these guidelines when:
- Creating new entities (e.g., characters, locations, items, events)
- Resolving NAP URIs into manifests
- Querying subtree data for creative workflows

## Core Commands

* **Create an Entity:** Use `nap create <entity_type> <entity_id> -u <repository> -n "<Name>"`.
  * *Example:* `nap create character woody -u toystory -n "Sheriff Woody"`.

* **Resolve a Full Manifest:** To fetch the complete YAML manifest of a resource, use `nap resolve <URI>`.
  * *Example:* `nap resolve nap://toystory/character/woody`.
  * You can specify output formats using the `-f` flag, such as `-f json` or `-f yaml`.

* **Query a Subtree (Crucial for AI workflows):** To fetch only specific data and save token context, append a fragment to the URI or use the query command. Subtree queries let AI agents fetch exactly the data they need, preserving token budget.
  * *Example (Fragment):* `nap resolve nap://toystory/character/woody#properties.toy_type`.
  * *Example (Command):* `nap query nap://toystory/character/woody properties`.

## Creative Workflow Example (Context Gathering)
If tasked with generating an image of a location:
1. Run `nap query nap://toystory/location/andysroom properties` to extract atmospheric details.
2. Run `nap resolve nap://toystory/location/andysroom#references.appears_in` to find associated scenes.
3. Use this targeted data to construct your generation prompt.

## CLI Reference


# NAP CLI Reference
The `nap` command-line interface (v0.4.5) provides tools for creating, resolving, and managing narrative resources using the Narrative Addressing Protocol.


## Command Overview

| Command | Description |
|---|---|
| [\`nap add-repr\`](docs/generated/commands/add-repr.md) | Add a representation to an entity manifest |
| [\`nap branch\`](docs/generated/commands/branch.md) | Create or list branches |
| [\`nap choose\`](docs/generated/commands/choose.md) | Choose backend provider |
| [\`nap commit\`](docs/generated/commands/commit.md) | Commit changes to a repository repository |
| [\`nap content-hash\`](docs/generated/commands/content-hash.md) | Compute the SHA-256 content hash of a file |
| [\`nap create\`](docs/generated/commands/create.md) | Create a new entity manifest |
| [\`nap diff\`](docs/generated/commands/diff.md) | Show diff between two manifest files or versions |
| [\`nap doctor\`](docs/generated/commands/doctor.md) | Run diagnostics and repair |
| [\`nap head-hash\`](docs/generated/commands/head-hash.md) | Show the current HEAD commit hash |
| [\`nap history\`](docs/generated/commands/history.md) | View commit history for an entity |
| [\`nap init\`](docs/generated/commands/init.md) | Initialize a repository repository and/or configure the backend provider |
| [\`nap install\`](docs/generated/commands/install.md) | Install required dependencies |
| [\`nap list\`](docs/generated/commands/list.md) | List repositories or entities within a repository |
| [\`nap merge\`](docs/generated/commands/merge.md) | Three-way merge of JSON/YAML values |
| [\`nap publish\`](docs/generated/commands/publish.md) | Publish changes to remote |
| [\`nap pull\`](docs/generated/commands/pull.md) | Clone or pull a repository from a remote |
| [\`nap push\`](docs/generated/commands/push.md) | Push the current branch to its configured upstream remote |
| [\`nap query\`](docs/generated/commands/query.md) | Query a subtree from a manifest |
| [\`nap remote\`](docs/generated/commands/remote.md) | Manage git remotes on a repository |
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
| -d, --base-dir <BASE\_DIR> | Base directory for repository repositories. Defaults to $NAP\_DIR, or ~/.nap if unset |  |
| -v, --verbose <VERBOSE> | Enable verbose debug logging |  |


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



## Global Options


# Global Options
These options are available on all `nap` commands.


| Flag | Description | Default |
|---|---|---|
| -d, --base-dir <BASE\_DIR> | Base directory for repository repositories. Defaults to $NAP\_DIR, or ~/.nap if unset |  |
| -v, --verbose <VERBOSE> | Enable verbose debug logging |  |



## Environment Variables


# Environment Variables
The following environment variables are recognized by `nap`.


| Variable | Description |
|---|---|
| NAP\_OUTPUT | Override for --format |


