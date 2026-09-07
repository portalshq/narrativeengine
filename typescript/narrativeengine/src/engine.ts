import { InMemoryBlockCache, type BlockCache } from "./cache.js";
import {
  MemoryProvider,
  type GenerationProvider,
  type NarrativeDataProvider,
  type PxProvider,
} from "./provider.js";
import { generateHistoricalIndices } from "./sequence.js";
import type {
  BuildContextRequest,
  GenerateBlockRequest,
  GenerateBlockResult,
  GenerationProviderRequest,
  NarrativeBlock,
  NarrativeBlockInput,
  NarrativeContext,
  NarrativeEngineConfig,
  NarrativeEvent,
  NarrativeLore,
  NarrativeReference,
  NarrativeRelationship,
  NarrativeRepresentation,
  PxEnrichment,
  ResolvedNarrativeEngineConfig,
  ScoredHybridCandidate,
} from "./types.js";

export type NarrativeEngineErrorCode =
  | "INVALID_CONFIG"
  | "GENERATION_PROVIDER_REQUIRED"
  | "PX_FAILED"
  | "GENERATION_FAILED"
  | "PERSISTENCE_FAILED"
  | "BATCH_RESULT_MISMATCH";

export class NarrativeEngineError extends Error {
  readonly code: NarrativeEngineErrorCode;

  constructor(code: NarrativeEngineErrorCode, message: string, cause?: unknown) {
    super(message, cause === undefined ? undefined : { cause });
    this.name = "NarrativeEngineError";
    this.code = code;
  }
}

export class BatchGenerationError<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
> extends NarrativeEngineError {
  readonly completed: readonly GenerateBlockResult<TBlock, TLore>[];

  constructor(
    message: string,
    completed: readonly GenerateBlockResult<TBlock, TLore>[],
    cause?: unknown,
  ) {
    super("PERSISTENCE_FAILED", message, cause);
    this.name = "BatchGenerationError";
    this.completed = completed;
  }
}

export interface NarrativeEngineOptions<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
  TBlockInput extends NarrativeBlockInput = NarrativeBlockInput,
  TParameters = unknown,
> {
  dataProvider: NarrativeDataProvider<TBlock, TLore, TBlockInput>;
  generationProvider?: GenerationProvider<TBlockInput, TBlock, TLore, TParameters>;
  pxProvider?: PxProvider<TBlock, TLore>;
  config?: NarrativeEngineConfig;
  blockCache?: BlockCache<TBlock>;
}

interface DeferredBlock<TBlock extends NarrativeBlock> {
  promise: Promise<TBlock | undefined>;
  resolve: (block: TBlock | undefined) => void;
  reject: (error: unknown) => void;
}

interface PendingChannelBatch {
  indices: Set<number>;
  scheduled: boolean;
}

const DEFAULT_CONFIG: ResolvedNarrativeEngineConfig = Object.freeze({
  historicalSampleDivisions: 5,
  minimumBlocksForHistoricalSampling: 3,
  hybridSearchCandidateLimit: 20,
  maximumSearchResults: 3,
  minimumSearchRelevanceScore: 0.65,
  vectorSearchWeight: 0.7,
  notableSearchResultBonus: 1.5,
  maximumLoreItems: 20,
  maximumNotableBlocks: 20,
  maximumEntityRepresentations: 5,
  preferredEntityRepresentationProperties: Object.freeze([]),
  maximumConcurrentGenerationTasks: 4,
  enrichmentFailureBehavior: "continue",
  includeTemporalContextLabels: true,
  contextProse: Object.freeze({}),
});

function isDataProvider(value: unknown): value is NarrativeDataProvider {
  if (typeof value !== "object" || value === null) return false;
  const candidate = value as Partial<NarrativeDataProvider>;
  return (
    typeof candidate.getBlockCount === "function" &&
    typeof candidate.getLoreAtoms === "function" &&
    typeof candidate.getHybridSearchCandidates === "function" &&
    typeof candidate.getBlocksByIndices === "function" &&
    typeof candidate.getNotableEvents === "function" &&
    typeof candidate.getProviderType === "function" &&
    typeof candidate.insertBlock === "function"
  );
}

function resolveConfig(config: NarrativeEngineConfig = {}): ResolvedNarrativeEngineConfig {
  const resolved: ResolvedNarrativeEngineConfig = {
    historicalSampleDivisions: config.historicalSampleDivisions ?? config.reciprocalDivisions ?? DEFAULT_CONFIG.historicalSampleDivisions,
    minimumBlocksForHistoricalSampling: config.minimumBlocksForHistoricalSampling ?? config.minimumBlocks ?? DEFAULT_CONFIG.minimumBlocksForHistoricalSampling,
    hybridSearchCandidateLimit: config.hybridSearchCandidateLimit ?? config.hybridCandidateLimit ?? DEFAULT_CONFIG.hybridSearchCandidateLimit,
    maximumSearchResults: config.maximumSearchResults ?? config.hybridTopK ?? DEFAULT_CONFIG.maximumSearchResults,
    minimumSearchRelevanceScore: config.minimumSearchRelevanceScore ?? config.saliencyThreshold ?? DEFAULT_CONFIG.minimumSearchRelevanceScore,
    vectorSearchWeight: config.vectorSearchWeight ?? config.weightDense ?? DEFAULT_CONFIG.vectorSearchWeight,
    notableSearchResultBonus: config.notableSearchResultBonus ?? config.significanceCoefficient ?? DEFAULT_CONFIG.notableSearchResultBonus,
    maximumLoreItems: config.maximumLoreItems ?? config.maxLoreAtoms ?? DEFAULT_CONFIG.maximumLoreItems,
    maximumNotableBlocks: config.maximumNotableBlocks ?? config.maxNotableEvents ?? DEFAULT_CONFIG.maximumNotableBlocks,
    maximumEntityRepresentations: config.maximumEntityRepresentations ?? config.maxUniqueEntityRepresentations ?? DEFAULT_CONFIG.maximumEntityRepresentations,
    preferredEntityRepresentationProperties: Object.freeze([...(config.preferredEntityRepresentationProperties ?? config.representationProperties ?? [])]),
    maximumConcurrentGenerationTasks: config.maximumConcurrentGenerationTasks ?? config.maxConcurrency ?? DEFAULT_CONFIG.maximumConcurrentGenerationTasks,
    enrichmentFailureBehavior: config.enrichmentFailureBehavior ?? config.pxErrorPolicy ?? DEFAULT_CONFIG.enrichmentFailureBehavior,
    includeTemporalContextLabels: config.includeTemporalContextLabels ?? config.temporalPhrasing ?? DEFAULT_CONFIG.includeTemporalContextLabels,
    blockRetrieval: config.blockRetrieval,
    contextProse: Object.freeze({ ...(config.contextProse ?? {}) }),
    renderContext: config.renderContext,
  };

  const positiveIntegers: Array<[string, number]> = [
    ["historicalSampleDivisions", resolved.historicalSampleDivisions],
    ["minimumBlocksForHistoricalSampling", resolved.minimumBlocksForHistoricalSampling],
    ["hybridSearchCandidateLimit", resolved.hybridSearchCandidateLimit],
    ["maximumSearchResults", resolved.maximumSearchResults],
    ["maximumLoreItems", resolved.maximumLoreItems],
    ["maximumNotableBlocks", resolved.maximumNotableBlocks],
    ["maximumConcurrentGenerationTasks", resolved.maximumConcurrentGenerationTasks],
  ];
  for (const [name, value] of positiveIntegers) {
    if (!Number.isInteger(value) || value <= 0) {
      throw new NarrativeEngineError("INVALID_CONFIG", `${name} must be a positive integer.`);
    }
  }
  if (
    !Number.isInteger(resolved.maximumEntityRepresentations) ||
    resolved.maximumEntityRepresentations < 0
  ) {
    throw new NarrativeEngineError(
      "INVALID_CONFIG",
      "maximumEntityRepresentations must be a non-negative integer.",
    );
  }
  if (resolved.vectorSearchWeight < 0 || resolved.vectorSearchWeight > 1) {
    throw new NarrativeEngineError("INVALID_CONFIG", "vectorSearchWeight must be between zero and one.");
  }
  if (!Number.isFinite(resolved.minimumSearchRelevanceScore)) {
    throw new NarrativeEngineError("INVALID_CONFIG", "minimumSearchRelevanceScore must be finite.");
  }
  if (!Number.isFinite(resolved.notableSearchResultBonus) || resolved.notableSearchResultBonus < 0) {
    throw new NarrativeEngineError(
      "INVALID_CONFIG",
      "notableSearchResultBonus must be a non-negative finite number.",
    );
  }
  if (resolved.blockRetrieval) {
    if (!Number.isInteger(resolved.blockRetrieval.maximumBlocks) || resolved.blockRetrieval.maximumBlocks <= 0) {
      throw new NarrativeEngineError("INVALID_CONFIG", "blockRetrieval.maximumBlocks must be a positive integer.");
    }
    for (const step of resolved.blockRetrieval.steps) {
      const count = "takeNewestBlocks" in step
        ? step.takeNewestBlocks
        : "addNotableBlocksUntilThereAre" in step
          ? step.addNotableBlocksUntilThereAre
          : undefined;
      if (count !== undefined && (!Number.isInteger(count) || count <= 0)) {
        throw new NarrativeEngineError("INVALID_CONFIG", "blockRetrieval step counts must be positive integers.");
      }
    }
  }
  return resolved;
}

function uniqueIndices(indices: readonly number[]): number[] {
  return [...new Set(indices.filter((index) => Number.isInteger(index) && index > 0))];
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

async function mapWithConcurrency<TInput, TOutput>(
  values: readonly TInput[],
  concurrency: number,
  mapper: (value: TInput, index: number) => Promise<TOutput>,
): Promise<TOutput[]> {
  const results = new Array<TOutput>(values.length);
  let cursor = 0;

  const workers = Array.from({ length: Math.min(concurrency, values.length) }, async () => {
    while (cursor < values.length) {
      const index = cursor;
      cursor += 1;
      const value = values[index];
      if (value !== undefined) results[index] = await mapper(value, index);
    }
  });
  await Promise.all(workers);
  return results;
}

export class NarrativeEngine<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
  TBlockInput extends NarrativeBlockInput = NarrativeBlockInput,
  TParameters = unknown,
> {
  private readonly dataProvider: NarrativeDataProvider<TBlock, TLore, TBlockInput>;
  /** @deprecated Prefer constructor injection; exposed for legacy lab tooling. */
  readonly provider: NarrativeDataProvider<TBlock, TLore, TBlockInput>;
  private readonly generationProvider:
    | GenerationProvider<TBlockInput, TBlock, TLore, TParameters>
    | undefined;
  private readonly pxProvider: PxProvider<TBlock, TLore> | undefined;
  private readonly blockCache: BlockCache<TBlock>;
  private config: ResolvedNarrativeEngineConfig;
  private readonly pendingByChannel = new Map<string, PendingChannelBatch>();
  private readonly inFlightBlocks = new Map<string, DeferredBlock<TBlock>>();

  constructor(
    options?:
      | NarrativeEngineOptions<TBlock, TLore, TBlockInput, TParameters>
      | NarrativeDataProvider<TBlock, TLore, TBlockInput>,
  ) {
    if (isDataProvider(options)) {
      this.dataProvider = options as NarrativeDataProvider<TBlock, TLore, TBlockInput>;
      this.provider = this.dataProvider;
      this.generationProvider = undefined;
      this.pxProvider = undefined;
      this.config = resolveConfig();
      this.blockCache = new InMemoryBlockCache<TBlock>();
      return;
    }

    const resolvedOptions = options ?? {
      dataProvider: new MemoryProvider() as unknown as NarrativeDataProvider<
        TBlock,
        TLore,
        TBlockInput
      >,
    };
    this.dataProvider = resolvedOptions.dataProvider;
    this.provider = this.dataProvider;
    this.generationProvider = resolvedOptions.generationProvider;
    this.pxProvider = resolvedOptions.pxProvider;
    this.config = resolveConfig(resolvedOptions.config);
    this.blockCache = resolvedOptions.blockCache ?? new InMemoryBlockCache<TBlock>();
  }

  /** Return the fully resolved configuration currently used by this engine. */
  getLabConfig(): ResolvedNarrativeEngineConfig {
    return {
      ...this.config,
      preferredEntityRepresentationProperties: [
        ...this.config.preferredEntityRepresentationProperties,
      ],
    };
  }

  /** Merge, validate, and activate configuration changes for future requests. */
  setLabConfig(config: NarrativeEngineConfig): void {
    this.config = resolveConfig({ ...this.config, ...config });
  }

  /** Retrieve and render context without generating or persisting a story block. */
  async buildContext(request: BuildContextRequest): Promise<NarrativeContext<TBlock, TLore>> {
    const { channelId, inputQuery } = request;
    const recipe = this.config.blockRetrieval;
    const requestsSearchResults = !recipe || recipe.steps.some(
      (step) => "fillRemainingSpaceWithSearchResults" in step,
    );
    // A recipe retrieves notable blocks only after its earlier steps have run.
    // This lets a recent-first recipe avoid a notable query when it already
    // contains enough notable blocks.
    const requestsNotableBlocks = !recipe;
    const [totalBlockCount, rawLore, rawCandidates, rawNotableEvents] = await Promise.all([
      this.dataProvider.getBlockCount(channelId),
      this.dataProvider.getLoreAtoms(channelId),
      requestsSearchResults
        ? this.dataProvider.getHybridSearchCandidates(
            channelId,
            inputQuery,
            this.config.hybridSearchCandidateLimit,
          )
        : Promise.resolve([]),
      requestsNotableBlocks ? this.dataProvider.getNotableEvents(channelId) : Promise.resolve([]),
    ]);

    const loreAtoms = [...rawLore]
      .filter((atom) => atom.isActive !== false)
      .sort((left, right) => right.happenedAt - left.happenedAt)
      .slice(0, this.config.maximumLoreItems);
    let sortedNotableEvents = [...rawNotableEvents]
      .sort((left, right) => right.happenedAt - left.happenedAt || right.index - left.index)
    let notableEvents = sortedNotableEvents.slice(0, this.config.maximumNotableBlocks);
    const scoredCandidates = this.scoreCandidates(rawCandidates);
    const survivors = scoredCandidates
      .filter((candidate) => candidate.scoreFinalFused >= this.config.minimumSearchRelevanceScore)
      .sort(
        (left, right) =>
          right.scoreFinalFused - left.scoreFinalFused ||
          right.block.happenedAt - left.block.happenedAt ||
          right.block.index - left.block.index,
      )
      .slice(0, this.config.maximumSearchResults);

    this.warmBlocks(channelId, rawCandidates.map((candidate) => candidate.block));
    this.warmBlocks(channelId, notableEvents);

    let historicalIndices =
      totalBlockCount >= this.config.minimumBlocksForHistoricalSampling
        ? generateHistoricalIndices(totalBlockCount, this.config.historicalSampleDivisions)
        : [];
    let chronologicalBlocks: TBlock[];
    if (!recipe) {
      const historicalBlocks = await this.loadBlocks(channelId, historicalIndices);
      chronologicalBlocks = this.mergeBlocksNewestFirst([
        ...historicalBlocks,
        ...notableEvents,
        ...survivors.map((candidate) => candidate.block),
      ]);
    } else {
      historicalIndices = [];
      const selected: TBlock[] = [];
      const selectedIds = new Set<string>();
      const add = (block: TBlock): void => {
        if (selected.length >= recipe.maximumBlocks || selectedIds.has(String(block.id))) return;
        selected.push(block);
        selectedIds.add(String(block.id));
      };
      for (const step of recipe.steps) {
        if ("takeNewestBlocks" in step) {
          const count = Math.min(step.takeNewestBlocks, recipe.maximumBlocks);
          const recentBlocks = this.dataProvider.getNewestBlocks
            ? await this.dataProvider.getNewestBlocks(channelId, count)
            : await (async () => {
                const firstIndex = Math.max(1, totalBlockCount - count + 1);
                const recentIndices = Array.from(
                  { length: Math.max(0, totalBlockCount - firstIndex + 1) },
                  (_, index) => firstIndex + index,
                );
                return await this.loadBlocks(channelId, recentIndices);
              })();
          historicalIndices.push(...recentBlocks.map((block) => block.index));
          recentBlocks.forEach(add);
        } else if ("addNotableBlocksUntilThereAre" in step) {
          let notableCount = selected.filter((block) => block.isNotable === true).length;
          if (notableCount < step.addNotableBlocksUntilThereAre && sortedNotableEvents.length === 0) {
            const needed = Math.min(
              step.addNotableBlocksUntilThereAre - notableCount,
              recipe.maximumBlocks - selected.length,
            );
            const extraNotableBlocks = this.dataProvider.getNewestNotableBlocks
              ? await this.dataProvider.getNewestNotableBlocks(
                  channelId,
                  needed,
                  [...selectedIds],
                )
              : await this.dataProvider.getNotableEvents(channelId);
            sortedNotableEvents = [...extraNotableBlocks]
              .sort((left, right) => right.happenedAt - left.happenedAt || right.index - left.index);
            notableEvents = sortedNotableEvents;
            this.warmBlocks(channelId, notableEvents);
          }
          for (const block of sortedNotableEvents) {
            if (notableCount >= step.addNotableBlocksUntilThereAre || selected.length >= recipe.maximumBlocks) break;
            if (!selectedIds.has(String(block.id))) {
              add(block);
              notableCount += 1;
            }
          }
        } else {
          survivors.forEach((candidate) => add(candidate.block));
        }
      }
      chronologicalBlocks = this.mergeBlocksNewestFirst(selected);
    }

    const warnings: string[] = [];
    let enrichment: PxEnrichment = {};
    if (this.pxProvider) {
      try {
        enrichment = await this.pxProvider.enrichContext({
          channelId,
          inputQuery,
          chronologicalBlocks,
          loreAtoms,
          representationProperties: this.config.preferredEntityRepresentationProperties,
          maxUniqueEntityRepresentations: this.config.maximumEntityRepresentations,
        });
      } catch (error) {
        if (this.config.enrichmentFailureBehavior === "fail") {
          throw new NarrativeEngineError("PX_FAILED", "PX context enrichment failed.", error);
        }
        warnings.push(`PX context enrichment failed: ${errorMessage(error)}`);
      }
    }

    const entities = [...(enrichment.entities ?? [])];
    const representations = this.selectRepresentations(enrichment.representations ?? []);
    const references: NarrativeReference[] = [...(enrichment.references ?? [])];
    const relationships: NarrativeRelationship[] = [...(enrichment.relationships ?? [])];
    const eventHistory: NarrativeEvent[] = [...(enrichment.eventHistory ?? [])];
    const metadata = {
      providerType: this.dataProvider.getProviderType(),
      totalBlockCount,
      historicalIndices,
      hybridCandidateCount: rawCandidates.length,
      hybridSurvivorCount: survivors.length,
      notableEventCount: notableEvents.length,
      retrievedBlockCount: chronologicalBlocks.length,
    };
    const contextWithoutPrompt = {
      channelId,
      inputQuery,
      chronologicalBlocks,
      loreAtoms,
      entities,
      representations,
      references,
      relationships,
      eventHistory,
      metadata,
      warnings,
    };

    return {
      ...contextWithoutPrompt,
      prompt: this.config.renderContext
        ? this.config.renderContext(contextWithoutPrompt)
        : this.composePrompt(contextWithoutPrompt),
    };
  }

  /** @deprecated Build context and return only its rendered prompt text. */
  async generateContext(channelId: string, inputQuery: string): Promise<string> {
    return (await this.buildContext({ channelId, inputQuery })).prompt;
  }

  /** Build context, generate one block, persist it, and return both block and context. */
  async generateBlock(
    request: GenerateBlockRequest<TParameters>,
  ): Promise<GenerateBlockResult<TBlock, TLore>> {
    const generationProvider = this.requireGenerationProvider();
    const context = await this.buildContext(request);
    let draft: TBlockInput;
    try {
      draft = await generationProvider.generateBlock(this.toGenerationRequest(context, request));
    } catch (error) {
      throw new NarrativeEngineError("GENERATION_FAILED", "Block generation failed.", error);
    }

    try {
      const block = await this.dataProvider.insertBlock(request.channelId, draft);
      this.warmBlocks(request.channelId, [block]);
      return { block, context };
    } catch (error) {
      throw new NarrativeEngineError("PERSISTENCE_FAILED", "Generated block persistence failed.", error);
    }
  }

  /** Generate and persist an ordered batch of story blocks. */
  async generateBlocksBatch(
    requests: readonly GenerateBlockRequest<TParameters>[],
  ): Promise<readonly GenerateBlockResult<TBlock, TLore>[]> {
    if (requests.length === 0) return [];
    const generationProvider = this.requireGenerationProvider();
    const contexts = await mapWithConcurrency(
      requests,
      this.config.maximumConcurrentGenerationTasks,
      async (request) => await this.buildContext(request),
    );
    const generationRequests = contexts.map((context, index) => {
      const request = requests[index];
      if (request === undefined) throw new Error("Batch request index mismatch.");
      return this.toGenerationRequest(context, request);
    });

    let drafts: readonly TBlockInput[];
    try {
      drafts = generationProvider.generateBlocksBatch
        ? await generationProvider.generateBlocksBatch(generationRequests)
        : await mapWithConcurrency(
            generationRequests,
            this.config.maximumConcurrentGenerationTasks,
            async (generationRequest) => await generationProvider.generateBlock(generationRequest),
          );
    } catch (error) {
      throw new NarrativeEngineError("GENERATION_FAILED", "Batch block generation failed.", error);
    }

    if (drafts.length !== requests.length) {
      throw new NarrativeEngineError(
        "BATCH_RESULT_MISMATCH",
        `Generation provider returned ${drafts.length} drafts for ${requests.length} requests.`,
      );
    }

    const completed: Array<GenerateBlockResult<TBlock, TLore>> = [];
    for (let index = 0; index < requests.length; index += 1) {
      const request = requests[index];
      const context = contexts[index];
      const draft = drafts[index];
      if (request === undefined || context === undefined || draft === undefined) continue;
      try {
        const block = await this.dataProvider.insertBlock(request.channelId, draft);
        this.warmBlocks(request.channelId, [block]);
        completed.push({ block, context });
      } catch (error) {
        throw new BatchGenerationError(
          `Batch persistence failed at request ${index}.`,
          completed,
          error,
        );
      }
    }
    return completed;
  }

  /** Remove one block from the in-memory retrieval cache. */
  invalidateBlock(channelId: string, index: number): void {
    this.blockCache.invalidate(channelId, index);
  }

  /** Remove every cached block for one channel. */
  invalidateChannel(channelId: string): void {
    this.blockCache.invalidateChannel(channelId);
  }

  /** Remove every block from this engine's in-memory retrieval cache. */
  clearCache(): void {
    this.blockCache.clear();
  }

  private requireGenerationProvider(): GenerationProvider<TBlockInput, TBlock, TLore, TParameters> {
    if (!this.generationProvider) {
      throw new NarrativeEngineError(
        "GENERATION_PROVIDER_REQUIRED",
        "generateBlock requires a generationProvider; use buildContext for retrieval-only workflows.",
      );
    }
    return this.generationProvider;
  }

  private toGenerationRequest(
    context: NarrativeContext<TBlock, TLore>,
    request: GenerateBlockRequest<TParameters>,
  ): GenerationProviderRequest<TBlock, TLore, TParameters> {
    return request.parameters === undefined
      ? { context }
      : { context, parameters: request.parameters };
  }

  private scoreCandidates(
    candidates: readonly {
      block: TBlock;
      scoreVectorDense: number;
      scoreKeywordSparse: number;
    }[],
  ): Array<ScoredHybridCandidate<TBlock>> {
    const weightSparse = 1 - this.config.vectorSearchWeight;
    return candidates.map((candidate) => {
      const scoreRawFused =
        candidate.scoreVectorDense * this.config.vectorSearchWeight +
        candidate.scoreKeywordSparse * weightSparse;
      return {
        ...candidate,
        scoreRawFused,
        scoreFinalFused: candidate.block.isNotable
          ? scoreRawFused * this.config.notableSearchResultBonus
          : scoreRawFused,
      };
    });
  }

  private mergeBlocksNewestFirst(blocks: readonly TBlock[]): TBlock[] {
    const unique = new Map<string, TBlock>();
    for (const block of blocks) unique.set(String(block.id), block);
    return [...unique.values()].sort(
      (left, right) =>
        right.happenedAt - left.happenedAt || right.index - left.index ||
        String(right.id).localeCompare(String(left.id)),
    );
  }

  private selectRepresentations(
    representations: readonly NarrativeRepresentation[],
  ): NarrativeRepresentation[] {
    if (this.config.maximumEntityRepresentations === 0) return [];
    const groups = new Map<string, NarrativeRepresentation[]>();
    for (const representation of representations) {
      if (representation.uri.trim().length === 0) continue;
      const group = groups.get(representation.entityName) ?? [];
      group.push(representation);
      groups.set(representation.entityName, group);
    }

    const selected: NarrativeRepresentation[] = [];
    for (const group of groups.values()) {
      let representation: NarrativeRepresentation | undefined;
      if (this.config.preferredEntityRepresentationProperties.length === 0) {
        representation = group[0];
      } else {
        for (const property of this.config.preferredEntityRepresentationProperties) {
          representation = group.find((candidate) => candidate.property === property);
          if (representation) break;
        }
      }
      if (representation) selected.push(representation);
      if (selected.length >= this.config.maximumEntityRepresentations) break;
    }
    return selected;
  }

  private composePrompt(context: {
    inputQuery: string;
    chronologicalBlocks: readonly TBlock[];
    loreAtoms: readonly TLore[];
    entities: readonly unknown[];
    representations: readonly NarrativeRepresentation[];
    references: readonly NarrativeReference[];
    relationships: readonly NarrativeRelationship[];
    eventHistory: readonly NarrativeEvent[];
    metadata: { totalBlockCount: number };
  }): string {
    const sections: string[] = [];
    if (context.loreAtoms.length > 0) {
      sections.push(`${this.config.contextProse.loreHeading ?? "Essential facts of the story:"} ${context.loreAtoms.map((atom) => atom.content).join(" ")}`);
    }
    if (context.chronologicalBlocks.length > 0) {
      const oldestFirst = [...context.chronologicalBlocks].reverse();
      const blockLines = oldestFirst.map((block) => {
        if (!this.config.includeTemporalContextLabels) {
          return `${this.config.contextProse.entryLabel ?? "Entry"} ${String(block.id)}: ${block.content}`;
        }
        const offset = Math.max(1, context.metadata.totalBlockCount - block.index + 1);
        const unit = offset === 1
          ? (this.config.contextProse.singularRelativeTimeUnit ?? "beat")
          : (this.config.contextProse.pluralRelativeTimeUnit ?? "beats");
        return `${offset} ${unit} ago: ${block.content}`;
      });
      sections.push(`${this.config.contextProse.historyHeading ?? "Historical context:"}\n${blockLines.join("\n")}`);
    }

    sections.push(context.inputQuery);
    if (context.entities.length > 0) {
      sections.push(`${this.config.contextProse.entitiesHeading ?? "entities:"}\n${JSON.stringify(context.entities)}`);
    }
    return sections.join("\n\n");
  }

  private warmBlocks(channelId: string, blocks: readonly TBlock[]): void {
    for (const block of blocks) {
      if (Number.isInteger(block.index) && block.index > 0) {
        this.blockCache.set(channelId, block.index, block);
      }
    }
  }

  private async loadBlocks(channelId: string, indices: readonly number[]): Promise<TBlock[]> {
    const promises: Array<Promise<TBlock | undefined>> = uniqueIndices(indices).map((index) => {
      const cached = this.blockCache.get(channelId, index);
      if (cached) {
        return new Promise<TBlock>((resolve) => {
          resolve(cached);
        });
      }
      return this.getOrQueueBlock(channelId, index);
    });
    const resolved = await Promise.all(promises);
    const blocks: TBlock[] = [];
    for (const block of resolved) {
      if (block !== undefined) blocks.push(block as TBlock);
    }
    return blocks;
  }

  private getOrQueueBlock(channelId: string, index: number): Promise<TBlock | undefined> {
    const key = JSON.stringify([channelId, index]);
    const existing = this.inFlightBlocks.get(key);
    if (existing) return existing.promise;

    let resolvePromise: (block: TBlock | undefined) => void = () => undefined;
    let rejectPromise: (error: unknown) => void = () => undefined;
    const promise = new Promise<TBlock | undefined>((resolve, reject) => {
      resolvePromise = resolve;
      rejectPromise = reject;
    });
    this.inFlightBlocks.set(key, {
      promise,
      resolve: resolvePromise,
      reject: rejectPromise,
    });

    const pending = this.pendingByChannel.get(channelId) ?? {
      indices: new Set<number>(),
      scheduled: false,
    };
    pending.indices.add(index);
    this.pendingByChannel.set(channelId, pending);
    if (!pending.scheduled) {
      pending.scheduled = true;
      queueMicrotask(() => {
        void this.flushChannelBatch(channelId);
      });
    }
    return promise;
  }

  private async flushChannelBatch(channelId: string): Promise<void> {
    const pending = this.pendingByChannel.get(channelId);
    if (!pending) return;
    this.pendingByChannel.delete(channelId);
    const indices = [...pending.indices].sort((left, right) => left - right);

    try {
      const blocks = await this.dataProvider.getBlocksByIndices(channelId, indices);
      this.warmBlocks(channelId, blocks);
      const byIndex = new Map(blocks.map((block) => [block.index, block]));
      for (const index of indices) {
        const key = JSON.stringify([channelId, index]);
        this.inFlightBlocks.get(key)?.resolve(byIndex.get(index));
        this.inFlightBlocks.delete(key);
      }
    } catch (error) {
      for (const index of indices) {
        const key = JSON.stringify([channelId, index]);
        this.inFlightBlocks.get(key)?.reject(error);
        this.inFlightBlocks.delete(key);
      }
    }
  }
}

export type LabConfig = NarrativeEngineConfig;

export function configureLabEngine(_engine: NarrativeEngine): void {
  void _engine;
  // Compatibility shim. Configuration is supplied to the engine constructor or setLabConfig().
}
