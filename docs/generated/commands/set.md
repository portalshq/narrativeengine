---
generated: "true"
generator: px-docgen
version: 0.8.21
source: clap
---


# px set
Set a property on an entity manifest


## Synopsis
```bash
px set [OPTIONS] <URI> <KEY> <VALUE>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| key | Property key (dot-notation) | Yes |
| uri | PX URI | Yes |
| value | Property value | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -a, --author | Author identifier | px |
| -m, --message | Commit message | set property |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/px-cli/src/main.rs` — `set` command

