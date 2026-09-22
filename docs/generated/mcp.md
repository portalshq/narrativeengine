---
generated: "true"
generator: px-docgen
version: 0.8.25
source: mcp
---


# PX MCP Tool Reference
MCP tools exposed by `px-mcp-server`. Agents MUST use these tools; the `px` CLI is not available for agentic use.


| Tool | Description |
|---|---|
| [\`px\_add\`](docs/generated/mcp/px\_add.md) | Add a file representation to an entity manifest |
| [\`px\_auth\_login\`](docs/generated/mcp/px\_auth\_login.md) | Sign in through the configured Lore authentication service |
| [\`px\_auth\_logout\`](docs/generated/mcp/px\_auth\_logout.md) | Remove locally cached Lore credentials |
| [\`px\_auth\_status\`](docs/generated/mcp/px\_auth\_status.md) | Show the currently cached Lore identity without printing tokens |
| [\`px\_auth\`](docs/generated/mcp/px\_auth.md) | Manage secure authentication for the configured Lore provider |
| [\`px\_branch\`](docs/generated/mcp/px\_branch.md) | Create or list branches |
| [\`px\_commit\`](docs/generated/mcp/px\_commit.md) | Commit changes to a repository repository |
| [\`px\_completions\`](docs/generated/mcp/px\_completions.md) | Generate shell completions for \`px\` |
| [\`px\_configure\_status\`](docs/generated/mcp/px\_configure\_status.md) | Show current backend configuration and connectivity (default when no provider is given) |
| [\`px\_configure\`](docs/generated/mcp/px\_configure.md) | Configure version-control backend |
| [\`px\_content\_hash\`](docs/generated/mcp/px\_content\_hash.md) | Compute the BLAKE3 content hash of a file |
| [\`px\_create\`](docs/generated/mcp/px\_create.md) | Create a new entity manifest |
| [\`px\_diff\`](docs/generated/mcp/px\_diff.md) | Show diff between two manifest files or versions |
| [\`px\_doctor\`](docs/generated/mcp/px\_doctor.md) | Run diagnostics and repair |
| [\`px\_head\`](docs/generated/mcp/px\_head.md) | Show the current HEAD commit hash |
| [\`px\_history\`](docs/generated/mcp/px\_history.md) | View commit history for an entity |
| [\`px\_init\`](docs/generated/mcp/px\_init.md) | Initialize a repository repository and/or configure the backend provider |
| [\`px\_install\`](docs/generated/mcp/px\_install.md) | Install required dependencies |
| [\`px\_list\`](docs/generated/mcp/px\_list.md) | List repositories or entities within a repository |
| [\`px\_merge\`](docs/generated/mcp/px\_merge.md) | Three-way merge of JSON/YAML values |
| [\`px\_presign\`](docs/generated/mcp/px\_presign.md) | Create a time-limited public URL for a committed representation |
| [\`px\_pull\`](docs/generated/mcp/px\_pull.md) | Clone or pull a repository from a remote |
| [\`px\_push\`](docs/generated/mcp/px\_push.md) | Push the current branch to its configured upstream remote |
| [\`px\_query\`](docs/generated/mcp/px\_query.md) | Query a subtree from a manifest |
| [\`px\_remote\_add\`](docs/generated/mcp/px\_remote\_add.md) | Add a remote to a repository repository |
| [\`px\_remote\_ls\`](docs/generated/mcp/px\_remote\_ls.md) | List remotes on a repository repository |
| [\`px\_remote\_rm\`](docs/generated/mcp/px\_remote\_rm.md) | Remove a remote from a repository repository |
| [\`px\_remote\`](docs/generated/mcp/px\_remote.md) | Manage remotes on a repository |
| [\`px\_resolve\`](docs/generated/mcp/px\_resolve.md) | Resolve a PX URI to its manifest or a subtree |
| [\`px\_revert\`](docs/generated/mcp/px\_revert.md) | Revert a commit by hash (undoes all changes in that commit) |
| [\`px\_schema\`](docs/generated/mcp/px\_schema.md) | Print a JSON Schema for manifest or commit types |
| [\`px\_set\`](docs/generated/mcp/px\_set.md) | Set a property on an entity manifest |
| [\`px\_status\`](docs/generated/mcp/px\_status.md) | Show system status |
| [\`px\_switch\`](docs/generated/mcp/px\_switch.md) | Switch to a branch |
| [\`px\_sync\`](docs/generated/mcp/px\_sync.md) | Sync with remote |
| [\`px\_validate\`](docs/generated/mcp/px\_validate.md) | Validate a manifest against the PX schema |

