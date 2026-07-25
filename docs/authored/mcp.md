## MCP Server

The NAP installer bundles the native `nap-mcp-server` binary alongside `nap`. The MCP server is not a daemon; agent clients start it on demand over stdio, and it proxies tool calls to the host `nap` CLI.

Use this when you want Codex or another MCP client to talk to NAP from a sandboxed environment:

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

If `nap-mcp-server` is not on `PATH`, replace it with the full path installed by the NAP installer, usually `~/.local/bin/nap-mcp-server` or `/usr/local/bin/nap-mcp-server`.

Inside sandboxes, use the MCP tools instead of shelling out to `nap` directly for network-backed operations. Direct `nap` CLI commands remain the right choice for humans and host-local shells.
