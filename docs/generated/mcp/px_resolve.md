---
generated: "true"
generator: px-docgen
version: 0.8.24
source: mcp
---


# px_resolve
Resolve a PX URI to its manifest or a subtree


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| branch | string | No |  | Resolve at a specific branch |
| commit | string | No |  | Resolve at a specific commit hash |
| format | string | No | yaml | Output format: yaml, json |
| include\_blobs | boolean | No | false | Hydrate known readable provenance artifacts such as prompts and run records |
| provenance | boolean | No | false | Include condensed per-file provenance for the manifest and direct representations |
| uri | string | Yes |  | PX URI. e.g., "px://toystory/character/woody" |

