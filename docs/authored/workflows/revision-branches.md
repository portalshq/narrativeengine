## Revision Branches

Use a revision branch for ordinary iteration:

```text
revision-<entity-type>-<entity-id>
```

Example: `revision-character-atlas`.

Commit every accepted user-visible iteration to the revision branch in the same turn. An accepted iteration means a concrete candidate the agent generated or revised in response to the user's current refinement request, unless the user explicitly said not to save it, asked only to brainstorm, or the generation failed. Multiple tiny corrections in one turn may be grouped into one accepted-revision commit; do not wait across many turns to batch revisions.

Rejected or superseded versions should remain traceable in branch history. Do not delete historical assets or replace history to make a rejected variant disappear.

If the user selects an older variant, restore that content as a new commit at the tip of the revision branch rather than rewriting history. If a reference such as "#2" is ambiguous or stale, resolve the mapping from the visible candidate set or ask a brief clarification before persisting the selection.
