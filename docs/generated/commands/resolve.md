---
generated: "true"
generator: nap-docgen
version: 0.4.5
git_sha: 01f23ae
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

Fragment queries are supported via the URI: nap resolve nap://starwars/character/lukeskywalker#references.appears_in


## Arguments

| Name | Description | Required |
|---|---|---|
| uri | NAP URI. e.g., "nap://starwars/character/lukeskywalker" | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --branch | Resolve at a specific branch |  |
|     --commit | Resolve at a specific commit hash |  |
|     --tag | Resolve at a specific tag |  |
| -f, --format | Output format: yaml, json | yaml |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help (see more with '--help') |


## Environment Variables

| Variable | Description |
|---|---|
| NAP\_OUTPUT | Override for --format |


## Examples
```bash
# Resolve a full manifest
nap resolve nap://starwars/character/lukeskywalker

# Resolve as JSON
nap resolve nap://starwars/character/lukeskywalker -f json

# Resolve at a specific branch
nap resolve nap://starwars/character/lukeskywalker --branch canon

# Resolve a subtree via fragment query
nap resolve nap://starwars/character/lukeskywalker#properties.species
# → human

# Resolve a nested subtree
nap resolve nap://starwars/character/lukeskywalker#references.appears_in
```

## Source
`crates/nap-cli/src/main.rs` — `resolve` command

