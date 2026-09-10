---
generated: "true"
generator: px-docgen
version: 0.8.18
source: clap
---


# px resolve
Resolve a PX URI to its manifest or a subtree


## Synopsis
```bash
px resolve [OPTIONS] <URI>
```


## Description
Resolve a PX URI to its manifest or a subtree.

Fragment queries are supported via the URI: px resolve px://toystory/character/woody#references.appears_in


## Arguments

| Name | Description | Required |
|---|---|---|
| uri | PX URI. e.g., "px://toystory/character/woody" | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --branch | Resolve at a specific branch |  |
|     --commit | Resolve at a specific commit hash |  |
| -f, --format | Output format: yaml, json | yaml |


## Flags

| Flag | Description |
|---|---|
|     --include-blobs | Hydrate known readable provenance artifacts such as prompts and run records |
|     --provenance | Include condensed per-file provenance for the manifest and direct representations |
| -h, --help | Print help (see more with '--help') |


## Environment Variables

| Variable | Description |
|---|---|
| PX\_OUTPUT | Override for --format |


## Examples
```bash
# Resolve a full manifest
px resolve px://toystory/character/woody

# Resolve as JSON
px resolve px://toystory/character/woody -f json

# Resolve at a specific branch
px resolve px://toystory/character/woody --branch canon

# Resolve a subtree via fragment query
px resolve px://toystory/character/woody#properties.toy_type
# → human

# Resolve a nested subtree
px resolve px://toystory/character/woody#references.appears_in
```

## Source
`crates/px-cli/src/main.rs` — `resolve` command

