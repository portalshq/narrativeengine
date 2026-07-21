---
generated: "true"
generator: nap-docgen
version: 0.4.5
source: clap
---


# nap add-repr
Add a representation to an entity manifest


## Synopsis
```bash
nap add-repr [OPTIONS] --format <FORMAT> <URI> <KEY> <FILE>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| file | File path to the asset | Yes |
| key | Representation key. e.g., "reference\_image" | Yes |
| uri | NAP URI | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --format | Asset format. e.g., "png", "glb" |  |
| -a, --author | Author identifier | nap-cli |
| -m, --message | Commit message | add representation |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/nap-cli/src/main.rs` — `add-repr` command

