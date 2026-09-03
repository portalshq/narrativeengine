---
generated: "true"
generator: nap-docgen
version: 0.8.2
source: clap
---


# nap resolve
Resolve a NAP URI to its manifest or a subtree


## Synopsis
```bash
nap resolve [OPTIONS] <URI>
```


## Description
Resolve a NAP URI to its manifest or a subtree.

Fragment queries are supported via the URI: nap resolve nap://toystory/character/woody#references.appears_in


## Arguments

| Name | Description | Required |
|---|---|---|
| uri | NAP URI. e.g., "nap://toystory/character/woody" | Yes |


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
| NAP\_OUTPUT | Override for --format |


## Examples
```bash
# Resolve a full manifest
nap resolve nap://toystory/character/woody

# Resolve as JSON
nap resolve nap://toystory/character/woody -f json

# Resolve at a specific branch
nap resolve nap://toystory/character/woody --branch canon

# Resolve a subtree via fragment query
nap resolve nap://toystory/character/woody#properties.toy_type
# → human

# Resolve a nested subtree
nap resolve nap://toystory/character/woody#references.appears_in
```

## Source
`crates/nap-cli/src/main.rs` — `resolve` command

