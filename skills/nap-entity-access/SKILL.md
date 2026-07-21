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


