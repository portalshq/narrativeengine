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
