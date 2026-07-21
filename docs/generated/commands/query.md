---
generated: "true"
generator: nap-docgen
version: 0.5.0
source: clap
---


# nap query
Query a subtree from a manifest


## Synopsis
```bash
nap query [OPTIONS] <URI> <PATH>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| path | Dot-notation path. e.g., "appearances.audienceVotes" | Yes |
| uri | NAP URI | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -f, --format | Output format: yaml, json | json |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Environment Variables

| Variable | Description |
|---|---|
| NAP\_OUTPUT | Override for --format |


## Examples
```bash
# Query a subtree
nap query nap://starwars/character/lukeskywalker properties

# Query nested properties
nap query nap://starwars/character/lukeskywalker properties.species

# Query as YAML
nap query nap://starwars/character/lukeskywalker properties -f yaml
```

## Source
`crates/nap-cli/src/main.rs` — `query` command

