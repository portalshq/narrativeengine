---
generated: "true"
generator: px-docgen
version: 0.8.16
source: clap
---


# px init
Initialize a repository repository and/or configure the backend provider


## Synopsis
```bash
px init [OPTIONS] [REPOSITORY]
```


## Description
Initialize a repository repository and/or configure the backend provider.

When a repository name is provided, creates the repository structure (directories, config, repository manifest, initial commit). When --provider is given (or no provider is configured), sets up the backend provider. Both can be combined:

px init toystory                     # create repository px init toystory --provider local    # create repository + configure provider px init --provider local             # configure provider only


## Arguments

| Name | Description | Required |
|---|---|---|
| repository | Repository name. If provided, initializes a new repository repository | No |


## Options

| Flag | Description | Default |
|---|---|---|
|     --origin | Remote URL to add as origin after init |  |
|     --provider | Provider type: local, portals-cloud, or remote |  |
|     --remote-url | Remote URL (required for remote provider) |  |
|     --workspace-id | Workspace ID (for remote provider) |  |


## Flags

| Flag | Description |
|---|---|
|     --reset | Reset the provider configuration file |
| -h, --help | Print help (see more with '--help') |


## Examples
```bash
# Initialize a new repository
px init toystory

# Initialize with local provider
px init toystory --provider local

# Initialize with remote provider
px init --provider remote --remote-url lore://localhost:41337 --workspace-id my-workspace

# Configure provider only (no repository creation)
px init --provider local

# Initialize with a remote origin
px init toystory --origin lore://localhost:41337/toystory
```

## Source
`crates/px-cli/src/main.rs` — `init` command

