---
generated: "true"
generator: px-docgen
version: 0.8.25
source: mcp
---


# px_init
Initialize a repository repository and/or configure the backend provider


## Parameters

| Name | Type | Required | Default | Description |
|---|---|---|---|---|
| provider | string | No |  | Provider type: local, portals-cloud, or remote |
| remote | string | No |  | Remote URL to add as origin after init |
| remote\_url | string | No |  | Remote URL (required for remote provider) |
| repository | string | No |  | Repository name. If provided, initializes a new repository repository |
| reset | boolean | No | false | Reset the provider configuration file |
| workspace\_id | string | No |  | Workspace ID (for remote provider) |

