---
generated: "true"
generator: px-docgen
version: 0.8.19
source: clap
---


# px commit
Commit changes to a repository repository


## Synopsis
```bash
px commit [OPTIONS] --message <MESSAGE> <REPOSITORY>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| repository | Repository name | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -a, --author | Author identifier | px |
| -m, --message | Commit message |  |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Examples
```bash
# Commit all changes in a repository
px commit toystory -m "Add Woody character"

# Commit with a specific author
px commit toystory -m "Update Andy's Room properties" -a "toybox-builder"
```

## Source
`crates/px-cli/src/main.rs` — `commit` command

