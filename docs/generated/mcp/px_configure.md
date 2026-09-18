---
generated: "true"
generator: px-docgen
version: 0.8.24
source: mcp
---


# px_configure
Configure version-control backend (unified: replaces `px choose` + `px backend`)

Subcommands: status


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| initial\_commit | boolean | No | false | Bootstrap existing unversioned repositories with an initial commit without prompting |
| no\_initial\_commit | boolean | No | false | Skip bootstrapping existing repositories |
| provider | string | No |  | Provider type: local, portals-cloud, or remote. Positional for ergonomics; omit to show current config (or use \`px configure status\`) |
| remote\_url | string | No |  | Remote URL (required for \`remote\`). Aliases: --endpoint, --remote\_url |
| reset | boolean | No | false | Reset provider configuration before (re)configuring |
| workspace\_id | string | No |  | Workspace ID (for \`remote\` and \`portals-cloud\`) |

