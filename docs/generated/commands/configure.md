---
generated: "true"
generator: px-docgen
version: 0.8.22
source: clap
---


# px configure
Configure version-control backend (unified: replaces `px choose` + `px backend`)


## Synopsis
```bash
px configure [OPTIONS] [PROVIDER] [COMMAND]
```


## Description
Configure version-control backend (unified: replaces `px choose` + `px backend`).

Examples: px configure                          # show current config px configure status                   # show current config px configure local                    # switch to local daemon px configure remote --remote-url lore://192.168.0.27:41337 px configure portals-cloud --workspace-id my-ws px configure --provider remote --remote-url lore://host:41337 --reset


## Arguments

| Name | Description | Required |
|---|---|---|
| provider | Provider type: local, portals-cloud, or remote. Positional for ergonomics; omit to show current config (or use \`px configure status\`) | No |


## Options

| Flag | Description | Default |
|---|---|---|
|     --remote-url | Remote URL (required for \`remote\`). Aliases: --endpoint, --remote\_url |  |
|     --workspace-id | Workspace ID (for \`remote\` and \`portals-cloud\`) |  |


## Flags

| Flag | Description |
|---|---|
|     --initial-commit | Bootstrap existing unversioned repositories with an initial commit without prompting |
|     --no-initial-commit | Skip bootstrapping existing repositories |
|     --reset | Reset provider configuration before (re)configuring |
| -h, --help | Print help (see more with '--help') |


## Aliases
- config


## Subcommands

| Command | Description |
|---|---|
| status | Show current backend configuration and connectivity (default when no provider is given) |


## Source
`crates/px-cli/src/main.rs` — `configure` command

