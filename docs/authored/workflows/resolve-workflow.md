## Entity Creation

When creating a new entity:

1. Establish the target branch (see `target-branch.md`), then resolve and apply repository context from that branch (see `repository-stewardship.md`).
2. Create the entity on the target branch via `px_create` (`entity_type`, `entity_id`, `repository`, `name`; px defaults to `main` if no branch is specified).
3. Report the exact URI.
4. Establish active task context: URI, repository, entity type, entity ID, target branch, default revision branch, and repository context (see `continuity.md`).
5. Create or switch to the revision branch via `px_branch` / `px_switch`:
   ```text
   revision-<entity-type>-<entity-id>
   ```
6. If the creation turn also generates a visual, text, audio, or other representation, immediately use `px-update` to commit that first accepted revision on the revision branch.

## Generation Context

Before generating from an entity:

1. Resolve repository.yaml from the target branch and gather relevant global properties, representations, and references.
2. Resolve the entity explicitly from the relevant branch via `px_resolve` (`uri`, plus `branch`): the target branch for canonical state, the revision branch for iterative work.
3. Gather properties that affect identity, narrative role, style, behavior, continuity, and exclusions.
4. Gather relevant entity `representations` and `references` (use `px_query` with `uri` and `path` for subtrees).
5. Treat project and entity image/video/audio representations as source-of-truth for observable appearance or sound. Text properties support and constrain them.
6. Inspect flexible negative-constraint keys such as `negative_constraints`, `exclusions`, `avoid`, `forbidden`, or project-specific equivalents at both scopes.
7. Keep multi-entity context separated so attributes do not bleed between entities.

## Branch Semantics

Resolve from the target branch for canonical state. Resolve from `revision-<entity-type>-<entity-id>` for iterative work. Pass the `branch` argument explicitly on every `px_resolve` / `px_query` call. Do not rely on whichever branch happens to be checked out. Do not store VCS branch-head data in manifests. Branch heads and commit history belong to PX/Lore version control.
