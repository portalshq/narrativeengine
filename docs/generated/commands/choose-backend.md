---
generated: "true"
generator: px-docgen
version: 0.8.19
source: clap
---


# px choose backend
Choose backend provider


## Synopsis
```bash
px backend [OPTIONS] <PROVIDER>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| provider | Provider type: local, portals-cloud, or remote | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --remote-url | Remote URL (required for remote provider) |  |
|     --workspace-id | Workspace ID (for remote provider) |  |


## Flags

| Flag | Description |
|---|---|
|     --reset | Reset the provider configuration file |
| -h, --help | Print help |


## Source
`crates/px-cli/src/main.rs` — `backend` command

