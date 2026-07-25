---
name: nap-repo
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

## Agent Sandbox Integration

When running inside a sandboxed environment (e.g., Codex) without outbound network access, use MCP tools instead of shelling out to the `nap` CLI directly. The MCP server runs on the host machine, starts only when the agent/MCP client launches it over stdio, and proxies tool calls to the host `nap` CLI.

Direct `nap` CLI examples in this skill are for humans, host-local shells, and non-sandboxed scripts. In an agent sandbox, use the MCP tools for any operation that may need Lore/cloud/network access.

### Setup

The standard NAP installer bundles the native `nap-mcp-server` binary with `nap`. If the MCP command is missing or broken, rerun the standard NAP installer from a host shell.

### Configuration

Add to your agent's MCP configuration (e.g., `~/.codex/config.json`):

```json
{
  "mcpServers": {
    "nap": {
      "command": "/bin/sh",
      "args": [
        "-lc",
        "NAP_DIR=\"$HOME/.nap\" exec nap-mcp-server"
      ]
    }
  }
}
```

### Available MCP Tools

All nap CLI commands are available as MCP tools with `nap_` prefix. For example:
- `nap resolve` -> `nap_resolve` tool
- `nap create` -> `nap_create` tool
- `nap set` -> `nap_set` tool

Prefer MCP tools over shell commands when in a sandbox.

## CLI Reference

{{include docs/generated/cli.md}}

## Global Options

{{include docs/generated/options.md}}

## Environment Variables

{{include docs/generated/environment.md}}
