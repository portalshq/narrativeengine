---
generated: "true"
generator: px-docgen
version: 0.8.25
source: clap
---


# px unset
Remove one or more properties or representations from an entity manifest


## Synopsis
```bash
px unset [OPTIONS] <URI> <KEYS>...
```


## Arguments

| Name | Description | Required |
|---|---|---|
| keys | Keys to remove. \`representations.<key>\` removes a representation | Yes |
| uri | PX URI | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -a, --author | Author identifier | px |
| -m, --message | Commit message |  |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/px-cli/src/main.rs` — `unset` command

