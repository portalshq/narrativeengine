---
name: nap-resolve
description: Create NAP entities, resolve NAP URIs, query entity context, and establish active entity continuity so later refinements automatically persist through nap-update.
---

# NAP Resolve
 
Use this skill to create entities, resolve NAP URIs, and gather entity context for creative workflows.
 
## When to Apply
 
Reference these guidelines when:
- Creating new entities (e.g., characters, locations, items, events)
- Resolving NAP URIs into manifests
- Querying subtree data for creative workflows

## Core Commands
 
Create an entity:
 
```bash
nap create character atlas -u bears -n "Atlas"
```
 
Resolve a manifest:
 
```bash
nap resolve nap://bears/character/atlas --branch main
```
 
Query a subtree:
 
```bash
nap query nap://bears/character/atlas properties
```
 
## Entity Creation
 
When creating a new entity:
 
1. Create the entity on the branch the user is working from (default `main` if none was specified — see "Target Branch" below).
2. Report the exact URI.
3. Establish active task context: URI, repository, entity type, entity ID, target branch, and default revision branch.
4. Create or switch to the revision branch:
   ```text
   revision-<entity-type>-<entity-id>
   ```
 
5. If the creation turn also generates a visual, text, audio, or other representation, immediately use `nap-update` to commit that first accepted revision on the revision branch.

## Target Branch
 
Establish the **target branch** — the branch accepted revisions will eventually promote to — at creation/first-resolve time, and carry it forward for the rest of the task:
 
1. If the user named a branch for this work, that's the target.
2. Otherwise the branch the entity is created on or first resolved from is the target.
3. Otherwise default to `main`.
`nap-update` uses this value for every promotion; it is not always `main`.
 
## Active Entity Continuity
 
After an entity URI is established, later turns that refine the same entity are continuity work. They must trigger `nap-update` even if the user does not say "NAP", "save", "commit", or the URI again.
 
Carry forward stable representation keys and identity constraints. Examples:
 
- `character_sheet`
- `face_sheet`
- `portrait`
- `reference_image`
- `voice_reference`

## Generation Context
 
Before generating from an entity:
 
1. Resolve the entity explicitly from the relevant branch (target branch for canonical state, revision branch for iterative work).
2. Gather properties that affect identity, narrative role, style, behavior, continuity, and exclusions.
3. Gather relevant `representations` and `references`.
4. Treat image/video/audio representations as source-of-truth for observable appearance or sound. Text properties support and constrain them.
5. Inspect flexible negative-constraint keys such as `negative_constraints`, `exclusions`, `avoid`, `forbidden`, or project-specific equivalents.
6. Keep multi-entity context separated so attributes do not bleed between entities.

## Branch Semantics
 
Resolve from the target branch for canonical state.
 
Resolve from `revision-<entity-type>-<entity-id>` for iterative work.
 
Use explicit `--branch` or MCP-equivalent arguments. Do not rely on whichever branch happens to be checked out.
 
Do not store VCS branch-head data in manifests. Branch heads and commit history belong to NAP/Lore version control.
 
## Guardrails

Checking the current workspace is not required for this skill. Nap usually stores all repos in a centralized directory unless configured otherwise.

{{include docs/generated/cli.md}}

{{include docs/authored/mcp/overview.md}}
