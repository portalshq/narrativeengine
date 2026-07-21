---
name: nap-entity-mutation
description: Update entity properties in a creative workflow and properly store creative assets back into the entity manifest, ensuring proper provenance.
metadata:
  author: portals
  version: "0.5.0"
---

# NAP Skill: Entity Mutation and Creative Workflow Integration

Update entity properties in a creative workflow and properly store creative assets back into the entity manifest, ensuring proper provenance.

## When to Apply

Reference these guidelines when:
- Making changes to entity properties in a creative workflow
- Generating new assets as part of a creative workflow
- Storing assets back into the entity manifest

## Core Commands

* **Set a Property:** To modify or add a property to an entity's manifest, use `nap set <URI> <property_key> <value>`.
  * *Example:* `nap set nap://toystory/character/woody toy_type pullstring_cowboy`.
  * *Example:* `nap set nap://toystory/character/woody location "nap://toystory/location/andysroom"`.

## Creative Workflow Pipeline
When using a creative tool (like a text model, Midjourney, or video generation platform) to generate assets for a NAP entity, you must follow this exact sequence:

1. **Resolve/Query:** Fetch necessary context using `nap resolve` or `nap query` (see Entity Access Skill).
2. **Generate:** Use your available creative tools to generate the text, image, or video based on the context.
3. **Store Representation:** When saving the generated asset back to the entity's manifest, you must track its provenance. Ensure the manifest is updated with the following structured YAML/JSON:
   * **Hash:** Every piece of content must be strictly indexed by its BLAKE3 hash. Do not use SHA-256.
   * **Provenance Tracking:** Record the AI generation metadata, including the `model` used (e.g., "midjourney-v6" or the specific LLM), the `prompt_hash`, and any `derived_from` URIs.

*Example of updating a manifest's representations block via CLI/script editing:*
```yaml
representations:
  ai_description:
    hash: "blake3:abc123def..." 
    format: text
    provenance:
      model: "claude-3-opus"
      prompt_hash: "blake3:def456..."
      derived_from: "nap://toystory/character/woody"
```

## CLI Reference


# NAP CLI Reference
The `nap` command-line interface (v0.5.0) provides tools for creating, resolving, and managing narrative resources using the Narrative Addressing Protocol.


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


