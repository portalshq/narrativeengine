---
generated: "true"
generator: nap-docgen
version: 0.8.10
source: clap
---


# nap add
Add a file representation to an entity manifest


## Synopsis
```bash
nap add [OPTIONS] --format <FORMAT> <URI> <KEY> <FILE>
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


## Examples
```bash
# Save a generated scene clip as a video representation
nap add nap://toystory/scene/pizza-planet clip-01 ./pizza-planet-clip-01.mp4 --format mp4 -m "Add pizza-planet scene clip"

# Save another take under a distinct representation key
nap add nap://toystory/scene/pizza-planet clip-02 ./pizza-planet-clip-02.mp4 --format mp4 -m "Add alternate pizza-planet scene clip"

# Inspect the scene and direct representation provenance
nap resolve nap://toystory/scene/pizza-planet --provenance
```

## Source
`crates/nap-cli/src/main.rs` — `add` command

