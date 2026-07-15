---
generated: "true"
generator: nap-docgen
version: 0.4.5
git_sha: 5db190b
source: clap
---


# nap pull
Clone or pull a universe from a remote


## Synopsis
```bash
nap pull <URL_OR_NAME>
```


## Description
Clone or pull a universe from a remote.

If the argument is a URL, the repo is cloned (name is read from the repo's own config).  If it's a universe name, the repo must already exist locally and will be updated via `git pull`.


## Arguments

| Name | Description | Required |
|---|---|---|
| url\_or\_name | URL (clone) or universe name (pull existing) | Yes |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help (see more with '--help') |


## Source
`crates/nap-cli/src/main.rs` — `pull` command

