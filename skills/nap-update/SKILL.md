---
name: nap-update
description: Persist changes to existing NAP entities, including narrative property updates and every user-visible creative iteration, and manage promotion of accepted revisions to the current working branch. Use whenever a user refines, regenerates, selects, rejects, endorses, or promotes an entity whose NAP URI was established earlier in the task, even if the user does not mention NAP again.
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
- the active revision branch and the **target branch** (see below)
- stable representation keys such as `character_sheet`, `face_sheet`, `portrait`, or `model_sheet`
- user-approved identity constraints and negative constraints
Later turns that keep refining the same entity must still invoke this skill even when the user does not mention NAP again.
 
## Target Branch
 
The **target branch** is whichever branch the entity's accepted work is meant to land on. It is **not always `main`** — resolve it per task, in this order:
 
1. If the user named a branch for this work (e.g., "we're doing this on the `classic` branch"), that branch is the target.
2. Otherwise, the branch the entity was resolved from at the start of the task (per `nap-resolve`) is the target.
3. Otherwise, default to `main`.
Every promotion action in this skill promotes to the **resolved target branch**, never a hardcoded `main`. When reporting or asking about promotion, name the target branch explicitly (e.g., "promote to `classic`") rather than saying "main" generically.
 
## Revision Branches
 
Use a revision branch for ordinary iteration:
 
```text
revision-<entity-type>-<entity-id>
```
 
Example: `revision-character-atlas`.
 
Commit every accepted user-visible iteration to the revision branch in the same turn. An accepted iteration means a concrete candidate the agent generated or revised in response to the user's current refinement request, unless the user explicitly said not to save it, asked only to brainstorm, or the generation failed. Multiple tiny corrections in one turn may be grouped into one accepted-revision commit; do not wait across many turns to batch revisions.
 
Rejected or superseded versions should remain traceable in branch history. Do not delete historical assets or replace history to make a rejected variant disappear.
 
If the user selects an older variant, restore that content as a new commit at the tip of the revision branch rather than rewriting history. If a reference such as "#2" is ambiguous or stale, resolve the mapping from the visible candidate set or ask a brief clarification before persisting the selection.
 
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
If the user pivots to a new NAP URI or a different task while the current entity has an unpromoted revision-branch tip, pause and ask before executing the switch.
 
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
 
## Update Pipeline
 
1. Resolve the entity explicitly from the active revision branch, not from implicit defaults:
   ```bash
   nap resolve nap://repo/type/id --branch revision-type-id
   ```
 
2. Gather every relevant property, representation, reference, and negative constraint that affects identity, continuity, style, exclusions, or the requested medium.
3. Generate or edit the requested content using the resolved entity as the source of truth.
4. Persist the result in the same turn:
   - `nap add --format <format> -m "<revision summary>" <URI> <representation_key> <asset>` for asset revisions
   - `nap set <URI> <property_key> <value>` for simple property-only updates
   - when several files/properties form one logical revision, update the structured manifest and make one `nap commit -m "<revision summary>" <repository>`
5. Store assets by BLAKE3 content hash, not SHA-256 — `nap content-hash` should return a `blake3:` value.
6. Record generation provenance with the revision: `model`, `prompt_hash`, `parameters` (when relevant), `derived_from` (source URIs/commits/hashes), `created_at` (when available).
7. Verify branch-specific resolution after the commit — check that the representation key, hash, and description match the accepted revision.
8. In the final response, report persistence in one concise line, then apply the relevant acceptance checkpoint from above.
## Data Placement
 
- `properties` — narrative facts and durable identity constraints.
- `representations` — current addressable assets, under stable semantic keys (e.g., `character_sheet` across all of Atlas's character-sheet revisions).
- Commit messages — revision notes (`revision_summary`). Do not create an append-only `revision_log` property; NAP/Lore history is the revision log.
- `metadata` — extension data that isn't narrative canon (e.g., per-representation provenance the manifest format can't otherwise express).
## Branch and Version Truth
 
Branch heads and commit history are VCS state — do not store or repair them inside entity manifests. Treat any branch-head data a NAP command exposes as derived status only.
 
Do not use commit `parent` fields as validation truth unless the current NAP/Lore version documents them as reliable. Count commit IDs/messages and verify branch-specific resolves instead.
 
Some NAP commands auto-commit (`nap add`, `nap set`). Do not run a second `nap commit` afterward unless you intentionally made additional uncommitted changes.
 
## Executing a Promotion
 
Once acceptance is confirmed (by any path in "Promotion to the Target Branch"):
 
1. Resolve the endorsed revision branch at its current tip.
2. Switch to the target branch.
3. Apply the same content-addressed representations/properties from the endorsed revision.
4. Commit with a promotion message naming the source revision branch and commit.
5. Resolve the target branch and verify representation hashes match the endorsed revision.
If promotion cannot complete without overwriting unrelated dirty content on the target branch, stop and report that promotion was blocked. Do not reset, revert, or discard user-authored content.

## Reporting
 
Revision saved, awaiting acceptance:
 
```text
NAP: Atlas revision saved on revision-character-atlas at a1b2c3d; character_sheet updated; <target-branch> unchanged.
Should I promote this to <target-branch>, or do we need more changes?
```
 
Promoted:
 
```text
NAP: Atlas promoted to <target-branch> at d4e5f6a from revision-character-atlas a1b2c3d.
```
 
Auto-promoted for a dependency:
 
```text
NAP: Atlas revision auto-promoted to <target-branch> at d4e5f6a to satisfy downstream generation.
```
 
Failed:
 
```text
NAP persistence failed: Atlas revision was generated but not committed.
```

## MCP Server

The standard NAP installer bundles the native `nap-mcp-server` binary with `nap`. If the MCP command is missing or broken, rerun the standard NAP installer from a host shell. 

The MCP server is not a daemon; agent clients start it on demand over stdio, and it proxies tool calls to the host `nap` CLI.

## Agent Sandbox Integration

When running inside a sandboxed environment (e.g., Codex) without outbound network access, use MCP tools instead of shelling out to the `nap` CLI directly. The MCP server runs on the host machine, starts only when the agent/MCP client launches it over stdio, and proxies tool calls to the host `nap` CLI.

Direct `nap` CLI examples in this skill are for humans, host-local shells, and non-sandboxed scripts. In an agent sandbox, use the MCP tools for any operation that may need Lore/cloud/network access.

## Available MCP Tools

All nap CLI commands are available as MCP tools with `nap_` prefix. For example:
- `nap resolve` -> `nap_resolve` tool
- `nap create` -> `nap_create` tool
- `nap set` -> `nap_set` tool

Prefer MCP tools over shell commands when in a sandbox.
