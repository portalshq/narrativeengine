---
generated: "true"
generator: nap-docgen
version: 0.5.12
source: clap
---


# nap commit
Commit changes to a repository repository


## Synopsis
```bash
nap commit [OPTIONS] --message <MESSAGE> <REPOSITORY>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| repository | Repository name | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -a, --author | Author identifier | nap-cli |
| -m, --message | Commit message |  |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Examples
```bash
# Commit all changes in a repository
nap commit toystory -m "Add Woody character"

# Commit with a specific author
nap commit toystory -m "Update Andy's Room properties" -a "toybox-builder"
```

## Source
`crates/nap-cli/src/main.rs` — `commit` command

