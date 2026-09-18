## Active Entity Continuity

Once a PX entity URI is established in a task, carry forward:

- the active URI, repository, entity type, and entity ID
- the active revision branch and the **target branch** (see `target-branch.md`)
- stable representation keys such as `character_sheet`, `face_sheet`, `portrait`, `model_sheet`, or `reference_image`
- user-approved identity constraints and negative constraints

Later turns that keep refining the same entity are continuity work. They must trigger `px-update` even when the user does not mention PX again and does not say "save" or "commit" or repeat the URI. Carry forward stable representation keys and identity constraints so attributes do not get dropped between turns.
