---
generated: "true"
generator: px-docgen
version: 0.8.22
source: clap
---


# px push
Push the current branch to its configured upstream remote


## Synopsis
```bash
px push [OPTIONS] <REPOSITORY>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| repository | Repository name | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --branch | Branch to push (default: current branch) |  |
|     --remote-name | Remote name (default: tracking branch's remote, or "origin") | origin |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help (see more with '--help') |


## Source
`crates/px-cli/src/main.rs` — `push` command

