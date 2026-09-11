---
generated: "true"
generator: px-docgen
version: 0.8.20
source: clap
---


# px add
Add a file representation to an entity manifest


## Synopsis
```bash
px add [OPTIONS] --format <FORMAT> <URI> <KEY> <FILE>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| file | File path to the asset | Yes |
| key | Representation key. e.g., "reference\_image" | Yes |
| uri | PX URI | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --format | Asset format. e.g., "png", "glb" |  |
| -a, --author | Author identifier | px |
| -m, --message | Commit message | add representation |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Examples
```bash
# Save a generated scene clip as a video representation
px add px://toystory/scene/pizza-planet clip-01 ./pizza-planet-clip-01.mp4 --format mp4 -m "Add pizza-planet scene clip"

# Save another take under a distinct representation key
px add px://toystory/scene/pizza-planet clip-02 ./pizza-planet-clip-02.mp4 --format mp4 -m "Add alternate pizza-planet scene clip"

# Inspect the scene and direct representation provenance
px resolve px://toystory/scene/pizza-planet --provenance
```

## Source
`crates/px-cli/src/main.rs` — `add` command

