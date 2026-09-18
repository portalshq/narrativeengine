## MCP Server (mandatory for agents)

Agents MUST issue all PX operations exclusively through the `px-mcp-server` MCP tools (`px_<command>`). The `px` CLI is NOT available for agentic use — never shell out, never follow `px ...` shell examples.

The standard PX installer bundles the native `px-mcp-server` binary with `px`. If the MCP server is missing or broken, rerun the standard PX installer from a host shell.

The MCP server is not a daemon; agent clients start it on demand over stdio (see `docs/authored/mcp/install.md` for client configuration), and it proxies tool calls to the host PX installation.

## Available MCP Tools

Every PX command is available as an MCP tool with a `px_` prefix and dashes/spaces converted to underscores. For example:

- `px_resolve` — resolve a PX URI to its manifest or a subtree
- `px_create` — create a new entity manifest
- `px_query` — query a subtree from a manifest
- `px_set` — set a property on an entity manifest
- `px_add` — add a file representation to an entity manifest
- `px_commit` — commit changes to a repository

Per-tool parameters are documented under `docs/generated/mcp/<tool>.md` (generated from the same command definitions as the MCP server itself). Tool arguments use MCP field names (for example `branch`, `commit`, `format`, `include_blobs`); pass the branch explicitly on every call that accepts one — do not rely on whichever branch happens to be checked out. The CLI reference (`docs/generated/cli.md`, `docs/generated/commands/`) and shell examples elsewhere in these docs are for humans in host shells only and MUST NOT be used as agent instructions.
