---
name: nap-repo-management
description: Initialize NAP repositories, including creating new repositories, cloning repositories, and branching existing ones.
metadata:
  author: portals
  version: "{{version}}"
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

{{include docs/generated/cli.md}}

## Global Options

{{include docs/generated/options.md}}

## Environment Variables

{{include docs/generated/environment.md}}
