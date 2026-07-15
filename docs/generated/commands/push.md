---
generated: "true"
generator: nap-docgen
version: 0.4.5
git_sha: 5db190b
source: clap
---


# nap push
Push the current branch to its configured upstream remote


## Synopsis
```bash
nap push [OPTIONS] <UNIVERSE>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| universe | Universe name | Yes |


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

