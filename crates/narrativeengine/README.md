# NarrativeEngine

NarrativeEngine is a structured-context framework and agentic middleware for applications that generate a sequential narrative. A block may be a story beat, scene, transcript segment, world-state update, or any other indexed unit.

The TypeScript implementation is the canonical implementation for JavaScript applications. The Rust crate remains available for the current Python binding, but the npm package contains no native code.

## What the engine owns

For each request, NarrativeEngine:

1. asks the application's data provider for the current block count, active lore, hybrid-search candidates, notable events, and selected historical blocks;
2. harmonizes recent-weighted history, semantic relevance, and notable events into a deterministic context;
3. optionally asks a PX provider to resolve entities, descriptions, signed representations, references, relationships, and event history;
4. passes that structured context to the application's generation provider;
5. asks the data provider to persist the generated draft; and
6. returns the authoritative persisted block and the exact context used to generate it.

Applications retain full control of their models, prompts, credentials, storage, and PX implementation. NarrativeEngine supplies orchestration and context-selection logic rather than an AI vendor.

```text
[ DATASTORE ] ── history ──► [ NARRATIVE ENGINE ] ── context ──► [ PX ]
                                  │                                │
                                  ◄──────── enrichment ────────────┘
                                  │
                                  └──► [ APPLICATION GENERATOR ]
                                                │
                                                ▼
                                          generated draft
                                                │
                                                ▼
                                           [ DATASTORE ]
                                                │
                                                ▼
                                      persisted block + context
```

## TypeScript API

```ts
import {
  NarrativeEngine,
  type GenerationProvider,
  type NarrativeDataProvider,
  type PxProvider,
} from "@portalshq/narrativeengine";

const engine = new NarrativeEngine({
  dataProvider,
  generationProvider,
  pxProvider,
  config: {
    representationProperties: ["avatar", "characterSheet", "image"],
    maxUniqueEntityRepresentations: 5,
  },
});

const { block, context } = await engine.generateBlock({
  channelId: "story-42",
  inputQuery: "Continue from the council's decision.",
  parameters: { temperature: 0.7 },
});
```

`NarrativeDataProvider` is the storage/RAG contract:

- `getBlockCount(channelId)`
- `getLoreAtoms(channelId)`
- `getHybridSearchCandidates(channelId, query, limit)`
- `getBlocksByIndices(channelId, indices)`
- `getNotableEvents(channelId)`
- `insertBlock(channelId, draft)`
- `getProviderType()`

`GenerationProvider` supplies `generateBlock(request)` and may supply an optimized `generateBlocksBatch(requests)`. The request contains the complete enriched `NarrativeContext`, so the consumer controls how it is converted into model messages.

`PxProvider.enrichContext(request)` may return:

- entities with descriptions;
- signed, model-ready representations;
- references;
- relationships; and
- event history.

PX executes before generation. PX errors continue with a warning by default; set `pxErrorPolicy: "fail"` to make them fatal. Generation and persistence errors are always fatal.

Retrieval-only consumers can call `buildContext()`. The deprecated `generateContext(channelId, inputQuery)` compatibility method returns only the composed prompt.

## Context selection

Historical sampling uses reciprocal spacing: sparse at the beginning of the sequence and increasingly dense near the latest block. The default pipeline combines:

- five reciprocal divisions once at least three blocks exist;
- up to 20 active lore atoms, newest first;
- 20 hybrid candidates with `dense × 0.7 + sparse × 0.3` fusion;
- a `1.5` significance multiplier for notable candidates;
- a `0.65` saliency threshold and three relevance survivors; and
- up to 20 explicitly notable events.

The result is deduplicated and exposed as `chronologicalBlocks` in descending time order. Prompt rendering reverses those blocks internally so the model reads history from oldest to newest.

The context envelope contains the blocks, lore, entities, representations, references, relationships, event history, retrieval metadata, warnings, and composed prompt. A representation has this stable shape:

```ts
interface NarrativeRepresentation {
  format: string;
  uri: string;       // signed, generation-ready URL
  name: string;
  entityName: string;
  id: string;
  description?: string;
  property?: string;
}
```

When `representationProperties` is configured, the engine selects the first available property in that order for each unique entity. It never invents implicit property names.

## Historical block cache

Each engine owns an `InMemoryBlockCache` unless another cache is injected. It persists retrieved blocks across requests in the same process:

- key: channel ID and block index;
- fixed default TTL: five minutes;
- default capacity: 10,000 entries with LRU eviction;
- concurrent misses: single-flight and channel-batched;
- partial hit: only absent or expired indices reach `getBlocksByIndices()`;
- warming: historical, hybrid, notable, and newly persisted blocks; and
- invalidation: `invalidateBlock`, `invalidateChannel`, and `clearCache`.

Block count, lore, hybrid searches, and notable-event lists remain dynamic and are never cached. Missing blocks are not negatively cached. The cache is process-local and is not durable across restarts. Entity caching belongs to the PX provider because PX owns entity-property invalidation.

## Schema boundary

`proto/narrative/v1/narrative.proto` defines language-neutral serializable DTOs. TypeScript DTOs are generated with `ts-proto`. Async provider callbacks and application-specific generics remain handwritten native-language interfaces; protobuf is not used as an in-process RPC layer.
