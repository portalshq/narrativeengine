---
generated: "true"
generator: px-docgen
version: 0.9.0
source: mcp
---


# px_presign
Create a time-limited public URL for a committed representation


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| branch | string | No |  | Resolve at a specific branch |
| commit | string | No |  | Resolve at a specific commit hash |
| download | string | No |  | Download the representation after creating its presigned URL. Optionally set its destination |
| http\_url | string | No |  | Explicit Lore HTTP origin, such as http://127.0.0.1:41339 |
| output | string | No |  | Destination for --download. Defaults to the entity asset directory |
| representation | string | Yes |  | Representation name (manifest key), e.g. item. Its URI is relative to the entity's asset directory |
| token\_env | string | No |  | Environment variable containing a repository-scoped bearer token |
| ttl\_seconds | string | No |  | Requested lifetime in seconds; Lore enforces its configured bounds |
| uri | string | Yes |  | Entity ID, e.g. 25th-chapter/character/nathan-gunn. The px:// prefix is optional; fragments are not supported |

