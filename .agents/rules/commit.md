---
trigger: always_on
---

## Git Commit Attribution

All AI-assisted commits must include attribution for the agent app and model used. This ensures transparency and traceability of AI contributions.

### Required Format

Add the following footer to every commit message when AI assistance was used:

```
Co-Authored-By: <agent-app> (<model>)
```

### Examples

```
feat: add user authentication flow

Co-Authored-By: opencode (nemotron-3-ultra-free)
```

```
fix: resolve race condition in data sync

Co-Authored-By: codex (gpt-4o)
```

```
refactor: extract shared validation logic

Co-Authored-By: devin (devin-1.0)
```

### Agent App Values

Use one of these standard identifiers:
- `opencode` - OpenCode CLI agent
- `codex` - GitHub Copilot / Codex
- `devin` - Devin AI
- `claude` - Claude Code / Claude Desktop
- `cursor` - Cursor IDE
- `windsurf` - Windsurf IDE
- `other` - Any other AI agent (specify)

### Model Values

Use the actual model identifier (e.g., `nemotron-3-ultra-free`, `gpt-4o`, `claude-3.5-sonnet`, `devin-1.0`, etc.)
