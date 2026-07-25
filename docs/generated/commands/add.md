---
generated: "true"
generator: nap-docgen
version: 0.5.3
source: clap
---


# nap add
Add a representation to an entity manifest


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
nap add nap://starwars/scene/cantina clip-01 ./cantina-clip-01.mp4 --format mp4 -m "Add cantina scene clip"

# Save another take under a distinct representation key
nap add nap://starwars/scene/cantina clip-02 ./cantina-clip-02.mp4 --format mp4 -m "Add alternate cantina scene clip"

# Inspect the scene and direct representation provenance
nap resolve nap://starwars/scene/cantina --provenance
```

## Source
`crates/nap-cli/src/main.rs` — `add` command

