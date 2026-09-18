## MCP Server Configuration

Add to your agent's MCP configuration (e.g., `~/.codex/config.json`):

```json
{
  "mcpServers": {
    "px": {
      "command": "/bin/sh",
      "args": [
        "-lc",
        "PX_DIR=\"$HOME/.px\" exec px-mcp-server"
      ]
    }
  }
}
```

## Connect with Codex

Codex stores MCP configuration in `~/.codex/config.toml` alongside the rest of its config. The Codex CLI, the ChatGPT desktop app, and the IDE extension all share that MCP configuration, so you only need to register `px` once.

Add the server with the CLI:

```bash
codex mcp add px --env PX_DIR="$HOME/.px" -- /bin/sh -lc 'exec px-mcp-server'
```

If `px-mcp-server` is not on `PATH`, use the full installed path instead, usually `~/.local/bin/px-mcp-server` or `/usr/local/bin/px-mcp-server`.

You can also configure it manually in `~/.codex/config.toml`:

```toml
[mcp_servers.px]
command = "/bin/sh"
args = ["-lc", "PX_DIR=\"$HOME/.px\" exec px-mcp-server"]
enabled = true
```

Project-scoped config works too for trusted projects:

```toml
[mcp_servers.px]
command = "/bin/sh"
args = ["-lc", "PX_DIR=\"$HOME/.px\" exec px-mcp-server"]
enabled = true
```

Use the same block in `.codex/config.toml` inside a trusted project if you want the server scoped to that repository.

## Connect with Claude Code

Register the server for your user account with the Claude Code CLI:

```bash
claude mcp add px --env PX_DIR="$HOME/.px" --scope user -- /bin/sh -lc 'exec px-mcp-server'
```

If `px-mcp-server` is not on `PATH`, use the full installed path instead, usually `~/.local/bin/px-mcp-server` or `/usr/local/bin/px-mcp-server`.

## Other MCP Clients

Claude Desktop and other MCP clients use the same stdio pattern. Add a server entry that runs the bundled `px-mcp-server` command on demand, and keep `PX_DIR` pointed at your PX workspace if you need a non-default data directory.

Example host-side launch command:

```bash
/bin/sh -lc 'PX_DIR="$HOME/.px" exec px-mcp-server'
```

Use the same command/args form in any client that supports stdio MCP servers.

Inside sandboxes, use the MCP tools for all PX operations. Direct `px` CLI commands are for humans in host-local shells only and MUST NOT be used as agent instructions.
