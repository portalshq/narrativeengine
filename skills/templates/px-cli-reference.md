---
name: px-cli-reference
description: Read-only reference for px CLI syntax, for humans asking what the command for something is (e.g. how to resolve a URI from a host shell). Never use for execution — agents MUST still execute all PX operations via px-mcp-server (see px-repo, px-resolve, px-update).
metadata:
  author: portals
  version: "{{version}}"
---

# PX CLI Reference (read-only)

## When to Apply

Reference these guidelines when a human asks about `px` CLI syntax: what command to run in a host shell, what flags a command accepts, or how a shell example works.

## When NOT to Apply

Never use this skill for agentic execution. All PX operations performed by an agent MUST go through the `px-mcp-server` MCP tools (`px_<command>`) per `px-repo`, `px-resolve`, and `px-update`. The `px` CLI is not available for agentic use — do not shell out, even when the exact CLI equivalent is documented below. If the task is to initialize, create, resolve, update, branch, pull, or commit anything, route to the corresponding execution skill instead.

{{include docs/authored/primitives-examples.md}}

{{include docs/generated/cli.md}}

{{include docs/generated/options.md}}

{{include docs/generated/environment.md}}
