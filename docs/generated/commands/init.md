---
generated: "true"
generator: nap-docgen
version: 0.4.5
git_sha: daa092c
source: clap
---


# nap init
Initialize a universe repository and/or configure the backend provider


## Synopsis
```bash
nap init [OPTIONS] [UNIVERSE]
```


## Description
Initialize a universe repository and/or configure the backend provider.

When a universe name is provided, creates the repository structure (directories, config, universe manifest, initial Git commit). When --provider is given (or no provider is configured), sets up the backend provider. Both can be combined:

nap init starwars                     # create universe nap init starwars --provider local    # create universe + configure provider nap init --provider local             # configure provider only


## Arguments

| Name | Description | Required |
|---|---|---|
| universe | Universe name. If provided, initializes a new universe repository | No |


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
| -h, --help | Print help (see more with '--help') |


## Examples
```bash
# Initialize a new universe
nap init starwars

# Initialize with local provider
nap init starwars --provider local

# Initialize with remote provider
nap init --provider remote --remote-url lore://localhost:41337 --workspace-id my-workspace

# Configure provider only (no universe creation)
nap init --provider local

# Initialize with a remote origin
nap init starwars --remote git@github.com:user/starwars.git
```

## Source
`crates/nap-cli/src/main.rs` — `init` command

