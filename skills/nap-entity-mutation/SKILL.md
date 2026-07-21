---
name: nap-entity-mutation
description: Update entity properties in a creative workflow and properly store creative assets back into the entity manifest, ensuring proper provenance.
metadata:
  author: portals
  version: "0.4.5"
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



## Global Options


# Global Options
These options are available on all `nap` commands.


| Flag | Description | Default |
|---|---|---|
| -d, --base-dir <BASE\_DIR> |  |  |
| -v, --verbose <VERBOSE> |  |  |



## Environment Variables


# Environment Variables
The following environment variables are recognized by `nap`.


| Variable | Description |
|---|---|


