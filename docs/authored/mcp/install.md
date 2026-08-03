## MCP Server Configuration

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

## Connect with Codex

Codex stores MCP configuration in `~/.codex/config.toml` alongside the rest of its config. The Codex CLI, the ChatGPT desktop app, and the IDE extension all share that MCP configuration, so you only need to register `nap` once.

Add the server with the CLI:

```bash
codex mcp add nap --env NAP_DIR="$HOME/.nap" -- /bin/sh -lc 'exec nap-mcp-server'
```

If `nap-mcp-server` is not on `PATH`, use the full installed path instead, usually `~/.local/bin/nap-mcp-server` or `/usr/local/bin/nap-mcp-server`.

You can also configure it manually in `~/.codex/config.toml`:

```toml
[mcp_servers.nap]
command = "/bin/sh"
args = ["-lc", "NAP_DIR=\"$HOME/.nap\" exec nap-mcp-server"]
enabled = true
```

Project-scoped config works too for trusted projects:

```toml
[mcp_servers.nap]
command = "/bin/sh"
args = ["-lc", "NAP_DIR=\"$HOME/.nap\" exec nap-mcp-server"]
enabled = true
```

Use the same block in `.codex/config.toml` inside a trusted project if you want the server scoped to that repository.

## Other MCP Clients

Claude Desktop and other MCP clients use the same stdio pattern. Add a server entry that runs the bundled `nap-mcp-server` command on demand, and keep `NAP_DIR` pointed at your NAP workspace if you need a non-default data directory.

Example host-side launch command:

```bash
/bin/sh -lc 'NAP_DIR="$HOME/.nap" exec nap-mcp-server'
```

Use the same command/args form in any client that supports stdio MCP servers.

Inside sandboxes, use the MCP tools instead of shelling out to `nap` directly for network-backed operations. Direct `nap` CLI commands remain the right choice for humans and host-local shells.
