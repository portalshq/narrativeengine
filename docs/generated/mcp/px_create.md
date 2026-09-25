---
generated: "true"
generator: px-docgen
version: 0.9.0
source: mcp
---


# px_create
Create a new entity manifest


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| author | string | No | px | Author identifier |
| entity\_id | string | Yes |  | Entity ID (slug). e.g., "woody" |
| entity\_type | string | Yes |  | Entity type (any non-empty string, e.g. character, location, custom-type) |
| message | string | No |  | Commit message |
| name | string | Yes |  | Human-readable name |
| properties | string | No |  | Initial property, as key=value. May be repeated |
| repository | string | Yes |  | Repository name |

