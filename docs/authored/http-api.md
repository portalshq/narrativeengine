## HTTP Server

The NAP resolver server provides a REST API for resolution and commits.

```bash
# Start the server (defaults to port 3100, base path = current directory)
nap-server

# Custom port and base path
NAP_PORT=8080 NAP_BASE_PATH=/path/to/universes nap-server
```

---

## Configuration

NAP core uses environment variables for configuration. All variables serve specific purposes with minimal overlap.

### Storage Configuration

| Variable | Purpose | Default | Required |
|----------|---------|---------|----------|
| `NAP_STORAGE_BACKEND` | Storage backend selection (`local` or `s3`) | `local` | No |
| `NAP_DIR` | Base directory for local storage | `~/.nap` | No (local) |
| `NAP_S3_BUCKET` | S3 bucket name | — | Yes (s3) |
| `AWS_ACCESS_KEY_ID` | AWS/R2 access key | — | Yes (s3) |
| `AWS_SECRET_ACCESS_KEY` | AWS/R2 secret key | — | Yes (s3) |
| `AWS_REGION` | AWS region | — | Yes (s3) |
| `AWS_ENDPOINT_URL_S3` | Custom S3 endpoint (R2, MinIO) | — | No (s3) |
| `AWS_ENDPOINT_URL` | Fallback S3 endpoint if `AWS_ENDPOINT_URL_S3` unset | — | No (s3) |

### Lore VCS Configuration

| Variable | Purpose | Default | Required |
|----------|---------|---------|----------|
| `NAP_LORE_URL_BASE` | Lore server URL base | `lore://localhost:8700` | No |
| `NAP_WORKSPACE_ID` | Workspace identifier for multi-tenancy | `default` | No |
| `NAPLORE_CLI` | Path to lore CLI binary | `lore` (from PATH) | No |
| `NAP_LORE_GRPC_ENDPOINT` | gRPC endpoint for branch ref sync | — | No (optional) |
| `NAP_LORE_GRPC_TOKEN` | JWT bearer token for gRPC auth | — | No (optional) |
| `NAP_LORE_GRPC_RID` | Repository ID (hex-encoded) for gRPC | — | No (optional) |
| `NAP_LORE_GRPC_INSECURE` | Skip TLS verification (`1`/`true`/`yes`) | `0` | No (optional) |

### Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `NAP_DIR` (const) | `.nap` | Metadata directory name within repositories |

**Note:** The environment variable `NAP_DIR` (storage base directory) and the constant `NAP_DIR` (metadata directory name) serve different purposes and do not overlap.

### Endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/resolve/{universe}/{entity_type}/{entity_id}` | Resolve a manifest |
| `GET` | `/resolve/{universe}/{entity_type}/{entity_id}?branch=canon` | Resolve at a branch |
| `POST` | `/commit/{universe}/{entity_type}/{entity_id}` | Commit changes |
| `GET` | `/history/{universe}/{entity_type}/{entity_id}` | Get commit history |
| `GET` | `/universes` | List all universes |
| `GET` | `/universes/{universe}/entities` | List entities in a universe |
| `GET` | `/health` | Health check |

Query parameters for resolution: `branch`, `commit`, `tag`, `path` (subtree query).
