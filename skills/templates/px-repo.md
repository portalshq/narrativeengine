---
name: px-repo
description: Initialize PX repositories, clone/pull repositories, and create branches at the repository level via px-mcp-server (px_init, px_pull, px_branch). The px CLI is not available for agentic use — not for creating or revising individual entities; see px-resolve and px-update for those.
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
For creating or resolving individual entities, use `px-resolve`. For revising entity content and persisting iterations, use `px-update`. For questions about `px` CLI syntax from humans, use `px-cli-reference` (read-only; never execute CLI commands).

{{include docs/authored/mcp/overview.md}}

{{include docs/authored/primitives.md}}

{{include docs/authored/workflows/repository-stewardship.md}}

{{include docs/generated/mcp/px_init.md}}

{{include docs/generated/mcp/px_branch.md}}

{{include docs/generated/mcp/px_pull.md}}

{{include docs/generated/mcp/px_list.md}}

{{include docs/generated/mcp/px_switch.md}}
