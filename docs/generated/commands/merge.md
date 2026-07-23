---
generated: "true"
generator: nap-docgen
version: 0.5.1
source: clap
---


# nap merge
Three-way merge of JSON/YAML values


## Synopsis
```bash
nap merge [OPTIONS] <BASE> <CURRENT> <PROPOSED>
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
`crates/nap-cli/src/main.rs` — `merge` command

