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
  reciprocalDivisions: 5,
  minimumBlocks: 3,
  hybridCandidateLimit: 20,
  hybridTopK: 3,
  saliencyThreshold: 0.65,
  weightDense: 0.7,
  significanceCoefficient: 1.5,
  maxLoreAtoms: 20,
  maxNotableEvents: 20,
  maxUniqueEntityRepresentations: 5,
  representationProperties: Object.freeze([]),
  maxConcurrency: 4,
  pxErrorPolicy: "continue",
  temporalPhrasing: true,
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
    reciprocalDivisions: config.reciprocalDivisions ?? DEFAULT_CONFIG.reciprocalDivisions,
    minimumBlocks: config.minimumBlocks ?? DEFAULT_CONFIG.minimumBlocks,
    hybridCandidateLimit: config.hybridCandidateLimit ?? DEFAULT_CONFIG.hybridCandidateLimit,
    hybridTopK: config.hybridTopK ?? DEFAULT_CONFIG.hybridTopK,
    saliencyThreshold: config.saliencyThreshold ?? DEFAULT_CONFIG.saliencyThreshold,
    weightDense: config.weightDense ?? DEFAULT_CONFIG.weightDense,
    significanceCoefficient:
      config.significanceCoefficient ?? DEFAULT_CONFIG.significanceCoefficient,
    maxLoreAtoms: config.maxLoreAtoms ?? DEFAULT_CONFIG.maxLoreAtoms,
    maxNotableEvents: config.maxNotableEvents ?? DEFAULT_CONFIG.maxNotableEvents,
    maxUniqueEntityRepresentations:
      config.maxUniqueEntityRepresentations ?? DEFAULT_CONFIG.maxUniqueEntityRepresentations,
    representationProperties: Object.freeze([...(config.representationProperties ?? [])]),
    maxConcurrency: config.maxConcurrency ?? DEFAULT_CONFIG.maxConcurrency,
    pxErrorPolicy: config.pxErrorPolicy ?? DEFAULT_CONFIG.pxErrorPolicy,
    temporalPhrasing: config.temporalPhrasing ?? DEFAULT_CONFIG.temporalPhrasing,
  };

  const positiveIntegers: Array<[string, number]> = [
    ["reciprocalDivisions", resolved.reciprocalDivisions],
    ["minimumBlocks", resolved.minimumBlocks],
    ["hybridCandidateLimit", resolved.hybridCandidateLimit],
    ["hybridTopK", resolved.hybridTopK],
    ["maxLoreAtoms", resolved.maxLoreAtoms],
    ["maxNotableEvents", resolved.maxNotableEvents],
    ["maxConcurrency", resolved.maxConcurrency],
  ];
  for (const [name, value] of positiveIntegers) {
    if (!Number.isInteger(value) || value <= 0) {
      throw new NarrativeEngineError("INVALID_CONFIG", `${name} must be a positive integer.`);
    }
  }
  if (
    !Number.isInteger(resolved.maxUniqueEntityRepresentations) ||
    resolved.maxUniqueEntityRepresentations < 0
  ) {
    throw new NarrativeEngineError(
      "INVALID_CONFIG",
      "maxUniqueEntityRepresentations must be a non-negative integer.",
    );
  }
  if (resolved.weightDense < 0 || resolved.weightDense > 1) {
    throw new NarrativeEngineError("INVALID_CONFIG", "weightDense must be between zero and one.");
  }
  if (!Number.isFinite(resolved.saliencyThreshold)) {
    throw new NarrativeEngineError("INVALID_CONFIG", "saliencyThreshold must be finite.");
  }
  if (!Number.isFinite(resolved.significanceCoefficient) || resolved.significanceCoefficient < 0) {
    throw new NarrativeEngineError(
      "INVALID_CONFIG",
      "significanceCoefficient must be a non-negative finite number.",
    );
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

  getLabConfig(): ResolvedNarrativeEngineConfig {
    return { ...this.config, representationProperties: [...this.config.representationProperties] };
  }

  setLabConfig(config: NarrativeEngineConfig): void {
    this.config = resolveConfig({ ...this.config, ...config });
  }

  async buildContext(request: BuildContextRequest): Promise<NarrativeContext<TBlock, TLore>> {
    const { channelId, inputQuery } = request;
    const [totalBlockCount, rawLore, rawCandidates, rawNotableEvents] = await Promise.all([
      this.dataProvider.getBlockCount(channelId),
      this.dataProvider.getLoreAtoms(channelId),
      this.dataProvider.getHybridSearchCandidates(
        channelId,
        inputQuery,
        this.config.hybridCandidateLimit,
      ),
      this.dataProvider.getNotableEvents(channelId),
    ]);

    const loreAtoms = [...rawLore]
      .filter((atom) => atom.isActive !== false)
      .sort((left, right) => right.happenedAt - left.happenedAt)
      .slice(0, this.config.maxLoreAtoms);
    const notableEvents = [...rawNotableEvents]
      .sort((left, right) => right.happenedAt - left.happenedAt || right.index - left.index)
      .slice(0, this.config.maxNotableEvents);
    const scoredCandidates = this.scoreCandidates(rawCandidates);
    const survivors = scoredCandidates
      .filter((candidate) => candidate.scoreFinalFused >= this.config.saliencyThreshold)
      .sort(
        (left, right) =>
          right.scoreFinalFused - left.scoreFinalFused ||
          right.block.happenedAt - left.block.happenedAt ||
          right.block.index - left.block.index,
      )
      .slice(0, this.config.hybridTopK);

    this.warmBlocks(channelId, rawCandidates.map((candidate) => candidate.block));
    this.warmBlocks(channelId, notableEvents);

    const historicalIndices =
      totalBlockCount >= this.config.minimumBlocks
        ? generateHistoricalIndices(totalBlockCount, this.config.reciprocalDivisions)
        : [];
    const historicalBlocks = await this.loadBlocks(channelId, historicalIndices);
    const chronologicalBlocks = this.mergeBlocksNewestFirst([
      ...historicalBlocks,
      ...notableEvents,
      ...survivors.map((candidate) => candidate.block),
    ]);

    const warnings: string[] = [];
    let enrichment: PxEnrichment = {};
    if (this.pxProvider) {
      try {
        enrichment = await this.pxProvider.enrichContext({
          channelId,
          inputQuery,
          chronologicalBlocks,
          loreAtoms,
          representationProperties: this.config.representationProperties,
          maxUniqueEntityRepresentations: this.config.maxUniqueEntityRepresentations,
        });
      } catch (error) {
        if (this.config.pxErrorPolicy === "fail") {
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
      prompt: this.composePrompt(contextWithoutPrompt),
    };
  }

  async generateContext(channelId: string, inputQuery: string): Promise<string> {
    return (await this.buildContext({ channelId, inputQuery })).prompt;
  }

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

  async generateBlocksBatch(
    requests: readonly GenerateBlockRequest<TParameters>[],
  ): Promise<readonly GenerateBlockResult<TBlock, TLore>[]> {
    if (requests.length === 0) return [];
    const generationProvider = this.requireGenerationProvider();
    const contexts = await mapWithConcurrency(
      requests,
      this.config.maxConcurrency,
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
            this.config.maxConcurrency,
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

  invalidateBlock(channelId: string, index: number): void {
    this.blockCache.invalidate(channelId, index);
  }

  invalidateChannel(channelId: string): void {
    this.blockCache.invalidateChannel(channelId);
  }

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
    const weightSparse = 1 - this.config.weightDense;
    return candidates.map((candidate) => {
      const scoreRawFused =
        candidate.scoreVectorDense * this.config.weightDense +
        candidate.scoreKeywordSparse * weightSparse;
      return {
        ...candidate,
        scoreRawFused,
        scoreFinalFused: candidate.block.isNotable
          ? scoreRawFused * this.config.significanceCoefficient
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
    if (this.config.maxUniqueEntityRepresentations === 0) return [];
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
      if (this.config.representationProperties.length === 0) {
        representation = group[0];
      } else {
        for (const property of this.config.representationProperties) {
          representation = group.find((candidate) => candidate.property === property);
          if (representation) break;
        }
      }
      if (representation) selected.push(representation);
      if (selected.length >= this.config.maxUniqueEntityRepresentations) break;
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
      sections.push(`Essential facts of the story: ${context.loreAtoms.map((atom) => atom.content).join(" ")}`);
    }
    if (context.chronologicalBlocks.length > 0) {
      const oldestFirst = [...context.chronologicalBlocks].reverse();
      const blockLines = oldestFirst.map((block) => {
        if (!this.config.temporalPhrasing) return `Entry ${String(block.id)}: ${block.content}`;
        const offset = Math.max(1, context.metadata.totalBlockCount - block.index + 1);
        return `${offset} ${offset === 1 ? "storyblock" : "storyblocks"} ago: ${block.content}`;
      });
      sections.push(`Historical context:\n${blockLines.join("\n")}`);
    }

    const pxPayload = {
      entities: context.entities,
      representations: context.representations,
      references: context.references,
      relationships: context.relationships,
      eventHistory: context.eventHistory,
    };
    if (Object.values(pxPayload).some((values) => values.length > 0)) {
      sections.push(`Structured entity context:\n${JSON.stringify(pxPayload)}`);
    }
    sections.push(context.inputQuery);
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
