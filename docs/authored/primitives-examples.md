## Primitives CLI Examples

Command-line examples for the [core primitives](./primitives.md), intended for humans working in a host shell. Agents must not execute these — agentic execution goes exclusively through `px-mcp-server` (see `px-repo`, `px-resolve`, `px-update`).

```bash
px create scene pizza-planet -u toystory -n "Pizza Planet"
px add px://toystory/scene/pizza-planet clip-01 ./pizza-planet-clip-01.mp4 --format mp4 -m "Add pizza-planet scene clip"
```

```bash
px resolve px://toystory/scene/pizza-planet --provenance
```
