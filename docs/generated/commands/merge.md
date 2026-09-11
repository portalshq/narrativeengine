---
generated: "true"
generator: px-docgen
version: 0.8.20
source: clap
---


# px merge
Three-way merge of JSON/YAML values


## Synopsis
```bash
px merge [OPTIONS] <BASE> <CURRENT> <PROPOSED>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| base | Base (common ancestor) file | Yes |
| current | Current (ours) file | Yes |
| proposed | Proposed (theirs) file | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -f, --format | Output format: json, yaml | yaml |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/px-cli/src/main.rs` — `merge` command

