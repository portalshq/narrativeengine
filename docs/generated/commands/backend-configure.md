---
generated: "true"
generator: nap-docgen
version: 0.6.2
source: clap
---


# nap backend configure
Configure the version-control backend


## Synopsis
```bash
nap configure [OPTIONS] <BACKEND>
```


## Description
Configure the version-control backend.

After configuration, existing unversioned repositories in this NAP home are offered an initial commit so their current filesystem state becomes the repository baseline (unless --no-initial-commit is given).


## Arguments

| Name | Description | Required |
|---|---|---|
| backend | Backend type: local or remote | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --endpoint | Remote endpoint URL (required for remote backend) |  |
|     --workspace-id | Workspace ID (for remote backend) |  |


## Flags

| Flag | Description |
|---|---|
|     --initial-commit | Bootstrap existing repositories with an initial commit without prompting |
|     --no-initial-commit | Skip bootstrapping existing repositories with an initial commit |
| -h, --help | Print help (see more with '--help') |


## Source
`crates/nap-cli/src/main.rs` — `configure` command

