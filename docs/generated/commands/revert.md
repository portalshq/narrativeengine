---
generated: "true"
generator: nap-docgen
version: 0.4.5
git_sha: daa092c
source: clap
---


# nap revert
Revert a commit by hash (undoes all changes in that commit)


## Synopsis
```bash
nap revert [OPTIONS] --commit <COMMIT> <UNIVERSE>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| universe | Universe name | Yes |


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

