---
name: px-repo
description: Initialize PX repositories, clone/pull repositories, and create branches at the repository level. Use for repository-lifecycle operations (px init, px pull, px branch) — not for creating or revising individual entities; see px-resolve and px-update for those.
metadata:
  author: portals
  version: "{{version}}"
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

{{include docs/generated/cli.md}}


{{include docs/authored/mcp/overview.md}}


{{include docs/generated/options.md}}


{{include docs/generated/environment.md}}
