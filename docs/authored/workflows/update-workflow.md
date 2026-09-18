## Update Pipeline

1. Switch to the target branch if not already there.
2. Resolve the repository.yaml and gather the relevant project context (see `repository-stewardship.md`).
3. Resolve the entity explicitly from the active revision branch via `px_resolve` (`uri`, `branch`), not from implicit defaults.
4. Gather every relevant project and entity property, representation, reference, and negative constraint that affects identity, continuity, style, exclusions, or the requested medium (use `px_query` with `uri` and `path` for subtrees).
5. Generate or edit the requested content using the resolved project and entity context as the source of truth.
6. Persist the result in the same turn:
   - `px_add` (`uri`, `key`, `file`, `format`, `message`) for asset revisions
   - `px_set` (`uri`, `key`, `value`) for simple property-only updates
   - when several files/properties form one logical revision, update the structured manifest and make one `px_commit` (`repository`, `message`) after updating
7. Store assets by BLAKE3 content hash, not SHA-256 — `px_content_hash` (`file`) should return a `blake3:` value.
8. Record generation provenance with the revision: `model`, `prompt_hash`, `parameters` (when relevant), `derived_from` (source URIs/commits/hashes), `created_at` (when available).
9. Verify branch-specific resolution after the commit — check that the representation key, hash, and description match the accepted revision.
9. In the final response, report persistence in one concise line, then apply the relevant acceptance checkpoint (see `promotion.md`).

## Data Placement

- `properties` — narrative facts and durable identity constraints.
- `representations` — current addressable assets, under stable semantic keys (e.g., `character_sheet` across all of Atlas's character-sheet revisions).
- Commit messages — revision notes (`revision_summary`). Do not create an append-only `revision_log` property; PX/Lore history is the revision log.
- `metadata` — extension data that isn't narrative canon (e.g., per-representation provenance the manifest format can't otherwise express).

## Branch and Version Truth

Branch heads and commit history are VCS state — do not store or repair them inside entity manifests. Treat any branch-head data a PX tool exposes as derived status only.

Do not use commit `parent` fields as validation truth unless the current PX/Lore version documents them as reliable. Count commit IDs/messages and verify branch-specific resolves instead.

Some PX tools auto-commit (`px_add`, `px_set`). Do not call `px_commit` afterward unless you intentionally made additional uncommitted changes.

## Character Creation and Character Sheets

Treat a request to create a character, character sheet, reference sheet, or other character visual as a persistence workflow. Do not leave generated assets only in the conversation or on a local filesystem.

1. Create or resolve the character entity on its target branch (see `resolve-workflow.md`).
2. Add every accepted generated asset with a stable semantic representation key via `px_add`. Use `character_sheet` for a complete three-view reference sheet and `portrait` for a portrait.
3. Commit the representation and any properties that describe the accepted character in the same turn (`px_add` and `px_set` may commit automatically; otherwise run one `px_commit` after updating the manifest).
4. Resolve the entity from the branch that received the commit and confirm its manifest contains the expected representation key, URI, format, and BLAKE3 hash.

Keep the manifest current whenever a character is created or a character sheet is generated. A file is not complete until it is represented in the entity manifest and committed to PX.

## Entity Variants

When the user asks to create a variant of an existing entity, create a new entity. Never replace the base entity's canonical identity or overwrite its representations to impersonate a variant.

1. Resolve the base entity and its target branch.
2. Create a distinct entity ID that describes the variant via `px_create`, for example `claire-cole-summer-dress` for `px://25th-chapter/character/claire-cole-summer-dress`.
3. Persist the variant's applicable properties and representations via `px_set` / `px_add`, then commit its manifest via `px_commit`.
4. Update the base entity's `properties.variants` value to the complete, deduplicated list of PX URIs for every known variant, including the new URI. Preserve existing entries; do not record local paths or bare IDs.
5. Commit the base-manifest update and resolve both entities to verify the variant and the base entity's `variants` property.

For example, the base entity `px://25th-chapter/character/claire-cole` keeps:

```yaml
properties:
  variants:
    - px://25th-chapter/character/claire-cole-summer-dress
    - px://25th-chapter/character/claire-cole-riot-gear
```

Use the same target branch for the base update and variant unless the user explicitly asks for a different branch. Report both committed entity URIs and their revisions.
