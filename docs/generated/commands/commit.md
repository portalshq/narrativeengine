---
generated: "true"
generator: nap-docgen
version: 0.4.5
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
nap commit starwars -m "Add Luke Skywalker character"

# Commit with a specific author
nap commit starwars -m "Update Tatooine properties" -a "worldbuilder"
```

## Source
`crates/nap-cli/src/main.rs` — `commit` command

