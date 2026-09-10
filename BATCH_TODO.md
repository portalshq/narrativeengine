# Sequential Story Batch Generation

## Goal

Increase canonical episode-generation throughput without treating dependent
story blocks as independent batch requests. Retrieve story context once, then
run a single model-assisted sequential loop that produces ordered blocks:

```text
retrieval snapshot -> block 1 -> block 2 -> block 3 -> ... -> persistence/media
```

Each later block must see the prior generated block(s), so the loop preserves
narrative causality while avoiding a full RAG/PX retrieval pass per block.

## Non-goals

- Do not switch canonical sessions to `NarrativeEngine.generateBlocksBatch`.
  Its contexts are built before earlier draft outputs exist, so it is suitable
  only for independent requests.
- Do not parallelize dependent canonical text generation.
- Do not change the current `generateBlocksBatch` implementation yet.

## Proposed capability

Add an explicit sequential-generation API, conceptually:

```ts
generateSequentialBlocks({
  channelId,
  seedContext,
  count,
  resolveLastBlock: true,
  retrieval: "once",
  onBlock,
  signal,
})
```

The implementation may use one model call with a tool loop, or repeated model
calls with a compact mutable turn ledger. The contract matters more than the
transport:

1. Build a retrieval snapshot once: chronological blocks, lore, PX entities,
   representations, and any warnings.
2. Generate exactly one next canonical block.
3. Validate it against the existing story-block schema.
4. Append a compact canonical record of that block to the mutable generation
   context (not another full RAG query).
5. Repeat until `count` is reached; mark only the final block as resolving.
6. Persist and generate media per successful block, retaining partial progress
   if a later block fails.

## Context design

- Keep immutable retrieval context separate from the mutable generated-turn
  ledger. This makes the prompt bounded and exposes what changed on each turn.
- Include recent generated blocks verbatim, then summarize older generated
  blocks when the token budget requires it.
- Carry required PX entities/representations from the initial snapshot across
  every turn. PX remains complementary: a PX error must not discard RAG.
- Define a hard prompt/token budget and report compaction in structured logs.
- Preserve the current fallback behavior if the initial retrieval snapshot
  fails; never silently replace a successful snapshot mid-loop.

## Reliability and observability

- One cancellation signal applies to the full loop.
- Persist a progress checkpoint after every canonical text block, before or
  alongside media work, so retries can resume rather than restart an episode.
- Log a batch/turn id, retrieval-snapshot id, block ordinal, context size,
  compaction events, model call timing, and fallback/circuit-breaker errors.
- Prompt logs should record the immutable retrieval snapshot reference plus the
  generated ledger used for that turn; avoid duplicating large payloads.
- Define retry policy at the individual-turn level. A transient model failure
  retries that ordinal; schema-invalid output must not advance the ledger.

## Integration plan

1. Add a pure sequential-context builder and tests for ordering, bounded
   context, resolution-on-last-block, and checkpoint recovery.
2. Add the generation-loop service behind a feature flag; it should return the
   same block shape consumed by the existing media and persistence pipeline.
3. Run a shadow/dry-run comparison against current sequential generation:
   continuity, token use, latency, and provider error rate.
4. Wire the flagged service into session pre-generation only after comparison
   results are acceptable. Ambient generation stays independent.
5. Add metrics and a rollback switch before enabling it for production.

## Decisions needed before implementation

- Model tool loop versus client-orchestrated repeated model calls.
- Exact compact-ledger/summarization policy and maximum context budget.
- Checkpoint storage schema and resume semantics.
- Whether media work overlaps the next text turn after the text checkpoint is
  durable.
- Feature-flag owner, rollout threshold, and success metrics.
