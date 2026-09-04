---
generated: "true"
generator: nap-docgen
version: 0.8.3
source: clap
---


# nap presign
Create a time-limited public URL for a committed representation


## Synopsis
```bash
nap presign [OPTIONS] <URI> <REPRESENTATION>
```


## Arguments

| Name | Description | Required |
|---|---|---|
| representation | Representation key. e.g., "reference\_image" | Yes |
| uri | NAP URI. Fragments are not supported | Yes |


## Options

| Flag | Description | Default |
|---|---|---|
|     --branch | Resolve at a specific branch |  |
|     --commit | Resolve at a specific commit hash |  |
|     --http-url | Explicit Lore HTTP origin, such as http://127.0.0.1:41339 |  |
|     --token-env | Environment variable containing a repository-scoped bearer token |  |
|     --ttl-seconds | Requested lifetime in seconds; Lore enforces its configured bounds |  |


## Flags

| Flag | Description |
|---|---|
| -h, --help | Print help |


## Source
`crates/nap-cli/src/main.rs` — `presign` command

