---
name: px-resolve
description: Create PX entities, resolve PX URIs, query entity context, and establish active entity continuity so later refinements automatically persist through px-update.
---

# PX Resolve
 
Use this skill to create entities, resolve PX URIs, and gather entity context for creative workflows.
 
## When to Apply
 
Reference these guidelines when:
- Creating new entities (e.g., characters, locations, items, events)
- Resolving PX URIs into manifests
- Querying subtree data for creative workflows

## Core Commands
 
Create an entity:
 
```bash
px create character atlas -u bears -n "Atlas"
```
 
Resolve a manifest:
 
```bash
px resolve px://bears/character/atlas --branch main
```
 
Query a subtree:
 
```bash
px query px://bears/character/atlas properties
```

## Repository Context

Before creating an entity or establishing creative context for generation,
resolve the repository's world manifest (`repository.yaml`) from the same
target branch as the entity work. Read its `properties`, `representations`,
and `references`, and resolve only the referenced resources relevant to the
requested work.

Treat repository context as the project-wide baseline. Apply entity-specific
identity, behavior, and representation data as refinements. If the two contain
a true contradiction, pause and ask the user for direction rather than choosing
one. If the repository manifest cannot be read, warn the user and ask how to
proceed; never silently substitute empty project context.

Carry the resolved repository context and branch forward to `px-update` for
any generated or revised representation. Do not write inferred project-wide
facts to `repository.yaml`; present them as proposed updates and wait for user
approval.

## Entity Creation

When creating a new entity:

1. Establish the target branch, then resolve and apply repository context from that branch.
2. Create the entity on the target branch (default `main` if none was specified — see "Target Branch" below).
3. Report the exact URI.
4. Establish active task context: URI, repository, entity type, entity ID, target branch, default revision branch, and repository context.
5. Create or switch to the revision branch:
   ```text
   revision-<entity-type>-<entity-id>
   ```

6. If the creation turn also generates a visual, text, audio, or other representation, immediately use `px-update` to commit that first accepted revision on the revision branch.

## Target Branch
 
Establish the **target branch** — the branch accepted revisions will eventually promote to — at creation/first-resolve time, and carry it forward for the rest of the task:
 
1. If the user named a branch for this work, that's the target.
2. Otherwise the branch the entity is created on or first resolved from is the target.
3. Otherwise default to `main`.
`px-update` uses this value for every promotion; it is not always `main`.
 
## Active Entity Continuity
 
After an entity URI is established, later turns that refine the same entity are continuity work. They must trigger `px-update` even if the user does not say "PX", "save", "commit", or the URI again.
 
Carry forward stable representation keys and identity constraints. Examples:
 
- `character_sheet`
- `face_sheet`
- `portrait`
- `reference_image`
- `voice_reference`

## Generation Context
 
Before generating from an entity:

1. Resolve the repository world manifest from the target branch and gather relevant global properties, representations, and references.
2. Resolve the entity explicitly from the relevant branch (target branch for canonical state, revision branch for iterative work).
3. Gather properties that affect identity, narrative role, style, behavior, continuity, and exclusions.
4. Gather relevant entity `representations` and `references`.
5. Treat project and entity image/video/audio representations as source-of-truth for observable appearance or sound. Text properties support and constrain them.
6. Inspect flexible negative-constraint keys such as `negative_constraints`, `exclusions`, `avoid`, `forbidden`, or project-specific equivalents at both scopes.
7. Keep multi-entity context separated so attributes do not bleed between entities.

## Branch Semantics
 
Resolve from the target branch for canonical state.
 
Resolve from `revision-<entity-type>-<entity-id>` for iterative work.
 
Use explicit `--branch` or MCP-equivalent arguments. Do not rely on whichever branch happens to be checked out.
 
Do not store VCS branch-head data in manifests. Branch heads and commit history belong to PX/Lore version control.
 
## Guardrails

Checking the current workspace is not required for this skill. Px usually stores all repos in a centralized directory unless configured otherwise.

{{include docs/generated/cli.md}}

{{include docs/authored/mcp/overview.md}}
