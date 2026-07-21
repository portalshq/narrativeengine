---
name: nap-repo-management
description: Initialize NAP repositories, including creating new repositories, cloning repositories, and branching existing ones.
metadata:
  author: portals
  version: "0.4.5"
---

# NAP Skill: Repository Management

A repository is the top-level repository that contains entities like characters, locations, and assets. 

## When to Apply

Reference these guidelines when:
- Initializing a new NAP repository
- Branching a NAP repository

## Core Commands

* **Initialize a Repository:** To create a new repository repository in the current directory, use `nap init <universe_name>`.
  * *Example:* `nap init toystory`.
  * *Note:* This creates a directory containing a `.nap/` configuration folder, a `repository.yaml` manifest, and subdirectories for entity types (characters, locations, etc.).

* **Branching:** To create a new timeline or snapshot, use `nap branch <universe_name> <branch_name>`.
  * *Example:* `nap branch toystory classic`.

## Critical Guardrails & Context
* **No Tagging:** Do not attempt to use `nap tag` or append tags to URIs. The underlying Lore VCS does not natively support tags. Branches are the primary and only way to apply a human-readable name to a specific point in the revision history.

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


