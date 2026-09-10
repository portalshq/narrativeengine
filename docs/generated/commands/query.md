---
generated: "true"
generator: px-docgen
version: 0.8.17
source: clap
---


# px query
Query a subtree from a manifest


## Synopsis
```bash
px query [OPTIONS] <URI> <PATH>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| path | Dot-notation path. e.g., "appearances.audienceVotes" | Yes |
| uri | PX URI | Yes |


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
| PX\_OUTPUT | Override for --format |


## Examples
```bash
# Query a subtree
px query px://toystory/character/woody properties

# Query nested properties
px query px://toystory/character/woody properties.toy_type

# Query as YAML
px query px://toystory/character/woody properties -f yaml
```

## Source
`crates/px-cli/src/main.rs` — `query` command

