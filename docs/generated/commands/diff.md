---
generated: "true"
generator: nap-docgen
version: 0.4.7
source: clap
---


# nap diff
Show diff between two manifest files or versions


## Synopsis
```bash
nap diff [OPTIONS] <BASE_FILE> <CANDIDATE_FILE>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| base\_file | Base (left) manifest file | Yes |
| candidate\_file | Candidate (right) manifest file | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -f, --format | Output format: json, yaml | yaml |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/nap-cli/src/main.rs` — `diff` command

