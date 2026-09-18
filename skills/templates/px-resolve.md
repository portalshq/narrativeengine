---
name: px-resolve
description: Create PX entities, resolve PX URIs, and query entity context via px-mcp-server (px_create, px_resolve, px_query), establishing active entity continuity so later refinements automatically persist through px-update. The px CLI is not available for agentic use.
---

# PX Resolve

Use this skill to create entities, resolve PX URIs, and gather entity context for creative workflows.

## When to Apply

Reference these guidelines when:

- Creating new entities (e.g., characters, locations, items, events)
- Resolving PX URIs into manifests
- Querying subtree data for creative workflows
For revising entity content and persisting iterations, use `px-update`. For repository-level init/pull/branch, use `px-repo`. For questions about `px` CLI syntax from humans, use `px-cli-reference` (read-only; never execute CLI commands).

{{include docs/authored/mcp/overview.md}}

{{include docs/authored/primitives.md}}

{{include docs/authored/workflows/repository-stewardship.md}}

{{include docs/authored/workflows/target-branch.md}}

{{include docs/authored/workflows/continuity.md}}

{{include docs/authored/workflows/resolve-workflow.md}}

{{include docs/generated/mcp/px_create.md}}

{{include docs/generated/mcp/px_resolve.md}}

{{include docs/generated/mcp/px_query.md}}
