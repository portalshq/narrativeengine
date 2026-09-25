---
generated: "true"
generator: px-docgen
version: 0.9.0
source: clap
---


# px history
View commit history for an entity or repository file


## Synopsis
```bash
px history [OPTIONS] <URI>
```


## Description
View commit history for an entity or repository file.

The target follows the normal repository/entity convention: `px history repo/type/id` shows the entity manifest, while `px history repo/type/id/asset.png` shows an asset in that entity's directory. A `.yaml` suffix on the entity form is accepted.


## Arguments

| Name | Description | Required |
|---|---|---|
| uri | PX URI or repository-relative entity/file target | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
| -n, --limit | Maximum number of commits to show | 20 |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help (see more with '--help') |


## Source
`crates/px-cli/src/main.rs` — `history` command

