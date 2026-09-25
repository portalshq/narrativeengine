---
generated: "true"
generator: px-docgen
version: 0.9.0
source: clap
---


# px set
Set one or more properties on an entity manifest


## Synopsis
```bash
px set [OPTIONS] <URI> <KEY> <VALUE>...
```


## Arguments

| Name | Description | Required |
|---|---|---|
| uri | PX URI | Yes |
| values | Repeating key/value pairs. Keys support dot-notation | Yes |


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
`crates/px-cli/src/main.rs` — `set` command

