---
generated: "true"
generator: px-docgen
version: 0.8.24
source: clap
---


# px completions
Generate shell completions for `px`


## Synopsis
```bash
px completions <SHELL>
```


## Description
Generate shell completions for `px`.

Usage: px completions bash > ~/.local/share/bash-completion/completions/px px completions zsh > ~/.zfunc/_px px completions fish > ~/.config/fish/completions/px.fish source <(px completions bash)   # ephemeral


## Arguments

| Name | Description | Required |
|---|---|---|
| shell | Shell to generate completions for | Yes |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help (see more with '--help') |


## Source
`crates/px-cli/src/main.rs` — `completions` command

