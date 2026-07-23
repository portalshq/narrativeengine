---
generated: "true"
generator: nap-docgen
version: 0.5.2
source: clap
---


# nap init
Initialize a repository repository and/or configure the backend provider


## Synopsis
```bash
nap init [OPTIONS] [REPOSITORY]
```


## Description
Initialize a repository repository and/or configure the backend provider.

When a repository name is provided, creates the repository structure (directories, config, repository manifest, initial commit). When --provider is given (or no provider is configured), sets up the backend provider. Both can be combined:

nap init starwars                     # create repository nap init starwars --provider local    # create repository + configure provider nap init --provider local             # configure provider only


## Arguments

| Name | Description | Required |
|---|---|---|
| repository | Repository name. If provided, initializes a new repository repository | No |


## Options

| Flag | Description | Default |
|---|---|---|
|     --provider | Provider type: local, portals-cloud, or remote |  |
|     --remote | Remote URL to add as origin after init |  |
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
nap init starwars

# Initialize with local provider
nap init starwars --provider local

# Initialize with remote provider
nap init --provider remote --remote-url lore://localhost:41337 --workspace-id my-workspace

# Configure provider only (no repository creation)
nap init --provider local

# Initialize with a remote origin
nap init starwars --remote git@github.com:user/starwars.git
```

## Source
`crates/nap-cli/src/main.rs` — `init` command

