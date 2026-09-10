---
generated: "true"
generator: px-docgen
version: 0.8.19
source: clap
---


# px create
Create a new entity manifest


## Synopsis
```bash
px create [OPTIONS] --repository <REPOSITORY> --name <NAME> <ENTITY_TYPE> <ENTITY_ID>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| entity\_id | Entity ID (slug). e.g., "woody" | Yes |
| entity\_type | Entity type (any non-empty string, e.g. character, location, custom-type) | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -a, --author | Author identifier | px |
| -n, --name | Human-readable name |  |
| -u, --repository | Repository name |  |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Examples
```bash
# Create a character
px create character woody -u toystory -n "Woody"

# Create a location
px create location andys-room -u toystory -n "Andy's Room"

# Create with a specific author
px create character jessie -u toystory -n "Jessie" -a "toybox-builder"
```

## Source
`crates/px-cli/src/main.rs` — `create` command

