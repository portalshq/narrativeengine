---
name: nap-update
description: Persist changes to existing NAP entities, including narrative property updates and every user-visible creative iteration. Use whenever a user refines, regenerates, selects, rejects, endorses, or promotes an entity whose NAP URI was established earlier in the task, even if the user does not mention NAP again.
---

# NAP Update

Use this skill whenever existing NAP entity content changes, especially during iterative creative work.

## When to Apply

Reference these guidelines when:
- Making changes to entity properties in a workflow
- Generating new assets as part of a workflow
- Storing assets back into the entity manifest
- Refining, regenerating, selecting, rejecting, endorsing, or promoting an entity whose NAP URI was established earlier in the task

## Continuity Rule

Once a NAP entity URI is established in a task, carry forward:

- the active URI, repository, entity type, and entity ID
- the active revision branch
- stable representation keys such as `character_sheet`, `face_sheet`, `portrait`, or `model_sheet`
- user-approved identity constraints and negative constraints

Later turns that keep refining the same entity must still invoke this skill even when the user does not mention NAP again.

## Revision Branches

Use a revision branch for ordinary iteration:

```text
revision-<entity-type>-<entity-id>
```

Example: `revision-character-atlas`.

Commit every accepted user-visible iteration to the revision branch in the same turn. In this skill, an accepted iteration means a concrete candidate the agent generated or revised in response to the user's current refinement request, unless the user explicitly said not to save it, asked only to brainstorm, or the generation failed. Multiple tiny corrections in one turn may be grouped into one accepted-revision commit; do not wait across many turns to batch revisions.

Rejected or superseded versions should remain traceable in branch history. Do not delete historical assets or replace history to make a rejected variant disappear.

Promote to `main` only when the user explicitly asks for canonical status, such as "lock it in", "definitive", "canonical", "make this the latest main", or another clear instruction. "Best so far" or "use #2" endorses the revision branch tip but does not automatically promote to `main` unless the user says so.

If the user selects an older variant, restore that content as a new commit at the tip of the revision branch rather than rewriting history. If a reference such as "#2" is ambiguous or stale, resolve the mapping from the visible candidate set or ask a brief clarification before persisting the selection or promoting it.

## Update Pipeline

1. Resolve the entity explicitly from the active revision branch, not from implicit defaults:

   ```bash
   nap resolve nap://repo/type/id --branch revision-type-id
   ```

2. Gather every relevant property, representation, reference, and negative constraint that affects identity, continuity, style, exclusions, or the requested medium.
3. Generate or edit the requested content using the resolved entity as the source of truth.
4. Persist the result in the same turn:
   - use `nap add --format <format> -m "<revision summary>" <URI> <representation_key> <asset>` for asset revisions
   - use `nap set <URI> <property_key> <value>` for simple property-only updates
   - when several files/properties must be one logical revision, update the structured manifest and make one `nap commit -m "<revision summary>" <repository>`
5. Store assets by BLAKE3 content hash. Do not use SHA-256. `nap content-hash` should return a `blake3:` value.
6. Record generation provenance with the revision:
   - `model`
   - `prompt_hash`
   - `parameters` when relevant
   - `derived_from` source URIs, commits, or asset hashes
   - `created_at` when available
7. Verify branch-specific resolution after the commit. Check that the intended representation key, hash, and latest description match the accepted revision.
8. In the final response, report persistence in one concise line.

## Data Placement

Use `properties` for narrative facts and durable identity constraints.

Use `representations` for current addressable assets. Keep stable semantic keys, such as updating `character_sheet` across Atlas character-sheet revisions.

Use commit messages for revision notes such as `revision_summary`. Do not create an append-only `revision_log` property; NAP/Lore history is the revision log.

Use `metadata` only for extension data that is not narrative canon, such as per-representation provenance when the manifest format cannot express it directly.

## Branch and Version Truth

Treat branch heads and commit history as VCS state. Do not store or repair VCS head pointers inside entity manifests. If a NAP implementation exposes branch head data in command output, treat it as derived status only, not entity content.

Do not use commit `parent` fields as validation truth unless the current NAP/Lore version documents them as reliable. Count commit IDs/messages and verify branch-specific resolves instead.

Be aware that some NAP commands auto-commit. For example, `nap add` and `nap set` create commits. Do not run a second `nap commit` afterward unless you intentionally made additional uncommitted changes.

## Main Promotion

When promoting to `main`:

1. Resolve the endorsed revision branch at its current tip.
2. Switch to `main`.
3. Apply the same content-addressed representations/properties from the endorsed revision.
4. Commit with a promotion message that names the source revision branch and commit.
5. Resolve `main` and verify that representation hashes match the endorsed revision.

If promotion cannot be completed without overwriting unrelated dirty content, stop and report that promotion was blocked. Do not reset, revert, or discard user-authored content.

## Sandbox Integration

In sandboxed agents, prefer NAP MCP tools when available for host-local or networked repositories. Direct CLI commands are appropriate for local repositories and test fixtures when the CLI is available in the sandbox.

## Reporting

Successful revision example:

```text
NAP: Atlas revision saved on revision-character-atlas at a1b2c3d; character_sheet updated; main unchanged.
```

Successful promotion example:

```text
NAP: Atlas promoted to main at d4e5f6a from revision-character-atlas a1b2c3d.
```

Failure example:

```text
NAP persistence failed: Atlas revision was generated but not committed.
```
