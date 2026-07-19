---
generated: "true"
generator: nap-docgen
version: 0.4.5
git_sha: 01f23ae
source: clap
---


# nap set
Set a property on an entity manifest


## Synopsis
```bash
nap set [OPTIONS] <URI> <KEY> <VALUE>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| key | Property key (dot-notation) | Yes |
| uri | NAP URI | Yes |
| value | Property value | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -a, --author | Author identifier | nap-cli |
| -m, --message | Commit message | set property |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/nap-cli/src/main.rs` — `set` command

