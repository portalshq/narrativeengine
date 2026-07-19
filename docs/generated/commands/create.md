---
generated: "true"
generator: nap-docgen
version: 0.4.5
git_sha: 01f23ae
source: clap
---


# nap create
Create a new entity manifest


## Synopsis
```bash
nap create [OPTIONS] --universe <UNIVERSE> --name <NAME> <ENTITY_TYPE> <ENTITY_ID>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| entity\_id | Entity ID (slug). e.g., "lukeskywalker" | Yes |
| entity\_type | Entity type: character, location, scene, prop, world | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -a, --author | Author identifier | nap-cli |
| -n, --name | Human-readable name |  |
| -u, --universe | Universe name |  |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Examples
```bash
# Create a character
nap create character lukeskywalker -u starwars -n "Luke Skywalker"

# Create a location
nap create location tatooine -u starwars -n "Tatooine"

# Create with a specific author
nap create character leiaorgana -u starwars -n "Leia Organa" -a "worldbuilder"
```

## Source
`crates/nap-cli/src/main.rs` — `create` command

