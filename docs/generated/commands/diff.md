---
generated: "true"
generator: px-docgen
version: 0.8.25
source: clap
---


# px diff
Show a manifest diff for an entity URI


## Synopsis
```bash
px diff [OPTIONS] <URI>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| uri | PX URI. The px:// prefix is optional | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --base-branch | Base branch |  |
|     --base-commit | Base commit |  |
|     --candidate-branch | Candidate branch |  |
|     --candidate-commit | Candidate commit |  |
| -f, --format | Output format: json, yaml | yaml |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/px-cli/src/main.rs` — `diff` command

