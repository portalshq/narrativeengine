---
generated: "true"
generator: px-docgen
version: 0.9.0
source: clap
---


# px pull
Clone or pull PX manifests from a remote (representation files stay remote)


## Synopsis
```bash
px pull <URL_OR_NAME>
```


## Description
Clone or pull PX manifests from a remote (representation files stay remote).

If the argument is a URL, the repo is cloned (name is read from the repo's own config). If it's a repository name, the repo must already exist locally and its manifests will be updated without downloading assets.


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

