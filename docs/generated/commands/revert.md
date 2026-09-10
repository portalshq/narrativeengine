---
generated: "true"
generator: px-docgen
version: 0.8.16
source: clap
---


# px revert
Revert a commit by hash (undoes all changes in that commit)


## Synopsis
```bash
px revert [OPTIONS] --commit <COMMIT> <REPOSITORY>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| repository | Repository name | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -a, --author | Author identifier | px |
| -c, --commit | Commit hash to revert |  |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/px-cli/src/main.rs` — `revert` command

