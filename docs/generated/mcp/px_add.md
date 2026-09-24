---
generated: "true"
generator: px-docgen
version: 0.8.25
source: mcp
---


# px_add
Add a file representation to an entity manifest


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| author | string | No | px | Author identifier |
| file | string | Yes |  | File path to the asset |
| format | string | Yes |  | Asset format. e.g., "png", "glb" |
| key | string | Yes |  | Representation key. e.g., "reference\_image" |
| message | string | No |  | Commit message |
| replace | boolean | No | false | Replace an existing representation only when its content differs |
| uri | string | Yes |  | PX URI |

