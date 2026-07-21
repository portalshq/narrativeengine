---
generated: "true"
generator: nap-docgen
version: 0.4.5
source: clap
---


# nap push
Push the current branch to its configured upstream remote


## Synopsis
```bash
nap push [OPTIONS] <REPOSITORY>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| repository | Repository name | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --branch | Branch to push (default: current branch) |  |
|     --remote | Remote name (default: tracking branch's remote, or "origin") | origin |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/nap-cli/src/main.rs` — `push` command

