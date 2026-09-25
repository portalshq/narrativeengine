---
generated: "true"
generator: px-docgen
version: 0.9.0
source: mcp
---


# px_merge
Three-way merge of JSON/YAML values


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| base | string | Yes |  | Base (common ancestor) file |
| current | string | Yes |  | Current (ours) file |
| format | string | No | yaml | Output format: json, yaml |
| proposed | string | Yes |  | Proposed (theirs) file |

