---
generated: "true"
generator: px-docgen
version: 0.8.16
source: clap
---


# px pull
Clone or pull a repository from a remote


## Synopsis
```bash
px pull <URL_OR_NAME>
```


## Description
Clone or pull a repository from a remote.

If the argument is a URL, the repo is cloned (name is read from the repo's own config).  If it's a repository name, the repo must already exist locally and will be updated via pull.


## Arguments

| Name | Description | Required |
|---|---|---|
| url\_or\_name | URL (clone) or repository name (pull existing) | Yes |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help (see more with '--help') |


## Source
`crates/px-cli/src/main.rs` — `pull` command

