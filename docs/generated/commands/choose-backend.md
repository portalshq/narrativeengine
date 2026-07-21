---
generated: "true"
generator: nap-docgen
version: 0.4.5
source: clap
---


# nap choose backend
Choose backend provider


## Synopsis
```bash
nap backend [OPTIONS] <PROVIDER>
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
| -h, --help | Print help |


## Source
`crates/nap-cli/src/main.rs` — `backend` command

