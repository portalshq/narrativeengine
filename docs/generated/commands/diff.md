---
generated: "true"
generator: px-docgen
version: 0.8.16
source: clap
---


# px diff
Show diff between two manifest files or versions


## Synopsis
```bash
px diff [OPTIONS] <BASE_FILE> <CANDIDATE_FILE>
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
`crates/px-cli/src/main.rs` — `diff` command

