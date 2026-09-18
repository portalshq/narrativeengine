## Target Branch

The **target branch** is whichever branch the entity's accepted work is meant to land on. It is **not always `main`** — resolve it per task, in this order:

1. If the user named a branch for this work (e.g., "we're doing this on the `classic` branch"), that branch is the target.
2. Otherwise, the branch the entity was created on or first resolved from is the target.
3. Otherwise, default to `main`.

Establish the target branch at creation/first-resolve time and carry it forward for the rest of the task. Every promotion promotes to the **resolved target branch**, never a hardcoded `main`. When reporting or asking about promotion, name the target branch explicitly (e.g., "promote to `classic`") rather than saying "main" generically.
