## Repository Context (`repository.yaml`)

`repository.yaml` is the repository's world manifest and the sole source of truth for project-wide context. It owns durable global canon, visual or narrative style, reusable asset conventions, global exclusions, and canonical references through its `properties`, `representations`, and `references`.

Before creating an entity or generating content, resolve the repository's world manifest from the same target branch as the entity work (via `px_resolve` on the repository URI). Read its `properties`, `representations`, and `references`, and resolve only the referenced resources relevant to the requested work. Treat repository context as the project-wide baseline; entity-specific identity, behavior, and representation data apply as refinements. Keep entity manifests focused on identity and entity-specific facts; do not add a repository-level summary or reference for every entity.

If project and entity instructions truly conflict, pause and ask the user for direction rather than choosing one. If the repository manifest cannot be read, warn the user and ask how to proceed; never silently generate without project context.

Update `repository.yaml` only when the user explicitly defines or approves a project-wide property or reference. If entity work reveals a potentially reusable global fact, present it as a proposed repository update and wait for user approval before writing it. Preserve existing global context when adding an approved change. Never promote inferred project-wide facts into `repository.yaml` as part of an entity update.

Unless the user explicitly requests a different provider or storage location, call `px_init` with no provider argument and preserve the configured provider and default PX directory. Never infer a local provider from an example. Never choose an isolated storage location merely to isolate a repository. Pass a provider only when the user explicitly requests a provider change, and a storage location only when the user explicitly names one.

**No tagging.** Lore VCS has no native tag support — branches are the only mechanism for human-readable names on a revision point. Do not append tags to URIs.

Checking the current workspace is not required: PX usually stores all repositories in a centralized directory unless configured otherwise.
