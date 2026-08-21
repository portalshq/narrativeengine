# Agent Notes

## Documentation

- `README.md` is generated. Do not edit it directly for lasting documentation changes.
- Edit `README.template.md` when changing the README structure or included sections.
- Edit files under `docs/authored/` when changing authored README content.
- Generated CLI documentation lives under `docs/generated/` and is produced from the CLI source by `nap-docgen`.
- Skill docs are generated from `skills/templates/` into `skills/*/SKILL.md`.
- After changing README templates, authored docs, generated-doc inputs, or skill templates, run:

```bash
just docgen
```

- Before finishing documentation work, run:

```bash
just docs-check
```

`just docs-check` reruns doc generation and verifies that generated outputs are current. If it reports stale documentation, inspect the generated diff and commit the intended generated files with the source changes.

## Releasing

**Always use `./scripts/publish-all.sh` to publish a new version.** Do not manually bump versions, commit release tags, or push release commits.

### Workflow

```bash
./scripts/publish-all.sh patch   # 0.5.8 → 0.5.9
./scripts/publish-all.sh minor   # 0.5.8 → 0.6.0
./scripts/publish-all.sh major   # 0.5.8 → 1.0.0
./scripts/publish-all.sh 2.3.0   # explicit version
```

The script handles the entire release:

1. Verifies clean working tree on `main`
2. Bumps versions across Cargo.toml, Cargo.lock, pyproject.toml, and package.json (11 workspace packages)
3. Runs pre-publish validation (version consistency, tag consistency, type freshness)
4. Builds TypeScript type definitions
5. Commits the release
6. Creates annotated tag `vX.Y.Z`
7. Pushes to origin
8. Triggers GitHub Actions publish workflow

### Prerequisites

- Must be on `main` with a clean working tree
- Must have push access to origin (run `gh auth login` if needed)
- If the `production` GitHub environment requires approval, approve the workflow run at https://github.com/DigitalCreationsCo/narrativeengine/actions

### What the script does NOT do

- Does not run tests (run `just test-all` before releasing)
- Does not update CHANGELOG (manual step if needed)

### Release Verification and Security

Nap releases use GitHub OIDC (OpenID Connect) and Sigstore for cryptographic verification. The release workflow (`.github/workflows/cli-release.yml`) generates:

- `SHA256SUMS` - Checksums of all release artifacts
- `SHA256SUMS.sigstore.json` - Sigstore bundle proving the checksums were signed by GitHub Actions
- `release-metadata.json` - Metadata including the pinned Lore client version and artifact digests
- `release-metadata.sigstore.json` - Sigstore bundle proving the metadata was signed by GitHub Actions

#### Cross-Origin Lore Verification

Nap depends on the Lore CLI, which is published in a separate repository (`portalshq/lore`). The release metadata includes:

- Lore version pinned in Nap's source code (`crates/nap-core/src/server/version.rs`)
- Lore artifact manifest URL (expected to be from `portalshq/lore/releases`)
- Lore artifact manifest SHA256 digest
- Lore signature bundle URL (expected to be from `portalshq/lore/releases`)

The parent repository's verification script (`cloud/infra/pulumi/scripts/verify-and-promote-nap-release.sh`) performs **cross-origin verification**:

1. **Verify Nap's authenticity**: Checks that Nap's `SHA256SUMS` and `release-metadata.json` were signed by the `narrativeengine` repository's GitHub OIDC workflow
2. **Verify Nap's artifacts**: Confirms downloaded binaries match the signed checksums
3. **Cross-check Lore claims**: Independently fetches Lore's SHA256SUMS from GitHub and verifies its digest matches what Nap's metadata claims
4. **Verify Lore's authenticity**: Checks that Lore's SHA256SUMS was signed by the `portalshq/lore` repository's GitHub OIDC workflow

This is **not same-origin verification**. Nap's metadata is a *claim* that gets cross-validated against the actual Lore release from an independent source. The security chain:

```
GitHub OIDC (narrativeengine) → Nap release → Nap's Lore claim
                                                 ↓
                                         Independent fetch from GitHub
                                                 ↓
GitHub OIDC (lore) → Lore release → Actual Lore artifacts
```

You cannot forge a Nap release that claims to depend on a malicious Lore version because:
- The malicious Lore version wouldn't exist at the claimed GitHub URL
- Even if it existed, its digest wouldn't match Nap's claim
- Even if the digest matched, it wouldn't be signed by the Lore repo's OIDC

This dual-source-of-trust model requires compromising both repositories independently to break the chain.

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

### When to Apply

Apply this attribution when:
- The agent wrote, modified, or reviewed code
- The agent provided architectural guidance that was implemented
- The agent generated commit messages or PR descriptions
- Any substantial AI contribution to the change

### When NOT to Apply

Do not add attribution for:
- Purely manual commits with no AI involvement
- Commits only updating documentation written by humans
- Routine dependency updates via dependabot/renovate
