---
generated: "true"
generator: px-docgen
version: 0.8.25
source: clap
---


# px commit
Commit all repository changes, or only one entity when given its URI


## Synopsis
```bash
px commit [OPTIONS] --message <MESSAGE> <TARGET>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| target | Repository name or PX entity URI | Yes |


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

