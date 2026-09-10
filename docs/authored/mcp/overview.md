## MCP Server

The standard PX installer bundles the native `px-mcp-server` binary with `px`. If the MCP command is missing or broken, rerun the standard PX installer from a host shell.

The MCP server is not a daemon; agent clients start it on demand over stdio, and it proxies tool calls to the host `px` CLI.

## Agent Sandbox Integration

When running inside a sandboxed environment (e.g., Codex) without outbound network access, use MCP tools instead of shelling out to the `px` CLI directly. The MCP server runs on the host machine, starts only when the agent/MCP client launches it over stdio, and proxies tool calls to the host `px` CLI.

Direct `px` CLI examples in this skill are for humans, host-local shells, and non-sandboxed scripts. In an agent sandbox, use the MCP tools for any operation that may need Lore/cloud/network access.

## Available MCP Tools

All px CLI commands are available as MCP tools with `px_` prefix. For example:
- `px resolve` -> `px_resolve` tool
- `px create` -> `px_create` tool
- `px set` -> `px_set` tool

Prefer MCP tools over shell commands when in a sandbox.