---
generated: "true"
generator: nap-docgen
version: 0.5.1
source: clap
---


# nap revert
Revert a commit by hash (undoes all changes in that commit)


## Synopsis
```bash
nap revert [OPTIONS] --commit <COMMIT> <REPOSITORY>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| repository | Repository name | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -a, --author | Author identifier | nap-cli |
| -c, --commit | Commit hash to revert |  |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/nap-cli/src/main.rs` — `revert` command

