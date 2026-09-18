## Promotion to the Target Branch

Promotion moves the endorsed tip of a revision branch onto the target branch. **Promotion always requires a signal of acceptance** — either the user stating it explicitly, or the agent proactively asking and getting a yes, or (in exactly one case below) an auto-promotion that is disclosed to the user. Silence is never acceptance.

There are five points in a workflow where acceptance is checked. Each is a **proactive ask**, except dependency-triggered promotion, which is an **auto-promotion with disclosure**.

### 1. End-of-turn ask
After committing a revision to the revision branch, close the turn by asking, briefly, whether to promote it to the target branch or keep iterating.

> "Atlas updated on the revision branch. Should I promote this to `<target-branch>`, or do we need more changes?"

### 2. Sentiment-triggered ask
Treat conversational affirmations ("looks great," "perfect," "that's the one," "thanks") as a signal the user is likely satisfied — but this alone is **not** acceptance. Ask explicitly before promoting.

> "Glad you like it! Want me to promote this to `<target-branch>`?"

### 3. Context-switch ask
If the user pivots to a new PX URI or a different task while the current entity has an unpromoted revision-branch tip, pause and ask before executing the switch.

> "Before we move on to the spaceship engine — want me to merge the recent Atlas revisions into `<target-branch>` first?"

### 4. Dependency-triggered auto-promotion (the one exception)
If the user asks to use a currently-revised entity in a new downstream context (e.g., "generate a scene using Atlas" while Atlas sits on a revision branch), auto-promote the endorsed revision-branch tip to the target branch immediately, without asking first — downstream generation must read from the target branch for continuity. Always disclose that this happened; do not promote silently.

> "Using the latest Atlas revision — I've promoted it to `<target-branch>` so the scene generation stays consistent."

### 5. Milestone bulk-merge ask
When the user indicates a session, task, or milestone is complete, check for any unpromoted `revision-*` branches across entities touched in the task. Summarize them and ask for one bulk approval.

> "You have unmerged revisions for Atlas and the Spaceship. Shall I promote both to `<target-branch>` before we wrap up?"

### Explicit user statement
The user can always state acceptance directly, and this satisfies the acceptance requirement immediately — at any of the checkpoints above, in reply to one of the proactive asks, or unprompted. Do not require a fixed magic phrase like "lock it in": interpret the intent behind whatever wording the user actually uses. This includes, at minimum:

- **Direct commands:** "lock it in," "make it canonical," "promote it," "merge it," "commit it to `<target-branch>`," "ship it."
- **Direct affirmatives in answer to a proactive ask:** "yes," "yep," "do it," "go ahead," "please," "sure," a thumbs-up-equivalent reply — any of these said in direct response to one of the five checkpoint questions counts as acceptance for that specific promotion.
- **Instructions that presuppose promotion:** "move to the next entity" (after being asked whether to promote first), "that's final," "we're done with Atlas," "use that version going forward."
If a reply is ambiguous as acceptance (e.g., it's unclear whether "yes" answers the promotion question or something else asked in the same turn), resolve it from context or ask a one-line clarification rather than guessing either way. Explicit statement is a valid mechanism but, per the above, is never the *only* mechanism the agent relies on — the agent must still proactively ask per checkpoints 1–3 and 5, and disclose per checkpoint 4.

### If declined or unresolved
If the user says not yet, keep working on the revision branch and re-ask at the next natural checkpoint (points 1–5 above). Never promote to the target branch without a yes from one of these five paths.

## Executing a Promotion

Once acceptance is confirmed (by any path above):

1. Resolve the endorsed revision branch at its current tip via `px_resolve`.
2. Switch to the target branch via `px_switch`.
3. Apply the same content-addressed representations/properties from the endorsed revision.
4. Commit with a promotion message naming the source revision branch and commit via `px_commit`.
5. Resolve the target branch and verify representation hashes match the endorsed revision.
If promotion cannot complete without overwriting unrelated dirty content on the target branch, stop and report that promotion was blocked. Do not reset, revert, or discard user-authored content.

## Reporting

Revision saved, awaiting acceptance:

```text
PX: Atlas revision saved on revision-character-atlas at a1b2c3d; character_sheet updated; <target-branch> unchanged.
Should I promote this to <target-branch>, or do we need more changes?
```

Promoted:

```text
PX: Atlas promoted to <target-branch> at d4e5f6a from revision-character-atlas a1b2c3d.
```

Auto-promoted for a dependency:

```text
PX: Atlas revision auto-promoted to <target-branch> at d4e5f6a to satisfy downstream generation.
```

Failed:

```text
PX persistence failed: Atlas revision was generated but not committed.
```
