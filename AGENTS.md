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
