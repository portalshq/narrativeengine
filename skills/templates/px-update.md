---
name: px-update
description: Persist changes to existing PX entities via px-mcp-server (px_add, px_set, px_commit), including narrative property updates and every user-visible creative iteration, and manage promotion of accepted revisions to the current working branch. The px CLI is not available for agentic use. Use whenever a user refines, regenerates, selects, rejects, endorses, or promotes an entity whose PX URI was established earlier in the task, even if the user does not mention PX again.
---

# PX Update

Use this skill whenever existing PX entity content changes, especially during iterative creative work.

## When to Apply

Reference these guidelines when:

- Making changes to entity properties in a workflow
- Generating new assets as part of a workflow
- Storing assets back into the entity manifest
- Refining, regenerating, selecting, rejecting, endorsing, or promoting an entity whose PX URI was established earlier in the task
For creating or first resolving entities, use `px-resolve`. For repository-level init/pull/branch, use `px-repo`. For questions about `px` CLI syntax from humans, use `px-cli-reference` (read-only; never execute CLI commands).

{{include docs/authored/mcp/overview.md}}

{{include docs/authored/workflows/repository-stewardship.md}}

{{include docs/authored/workflows/target-branch.md}}

{{include docs/authored/workflows/continuity.md}}

{{include docs/authored/workflows/revision-branches.md}}

{{include docs/authored/workflows/promotion.md}}

{{include docs/authored/workflows/update-workflow.md}}

{{include docs/generated/mcp/px_resolve.md}}

{{include docs/generated/mcp/px_query.md}}

{{include docs/generated/mcp/px_add.md}}

{{include docs/generated/mcp/px_set.md}}

{{include docs/generated/mcp/px_commit.md}}

{{include docs/generated/mcp/px_content_hash.md}}

{{include docs/generated/mcp/px_switch.md}}
