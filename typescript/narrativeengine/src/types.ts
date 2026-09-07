export type NarrativeId = string | number;

export interface NarrativeBlock {
  id: NarrativeId;
  index: number;
  content: string;
  happenedAt: number;
  isNotable?: boolean;
  [key: string]: unknown;
}

export interface NarrativeBlockInput {
  content: string;
  id?: NarrativeId;
  index?: number;
  happenedAt?: number;
  isNotable?: boolean;
  [key: string]: unknown;
}

export interface NarrativeLore {
  id: NarrativeId;
  content: string;
  happenedAt: number;
  isActive?: boolean;
  [key: string]: unknown;
}

export interface HybridCandidate<TBlock extends NarrativeBlock = NarrativeBlock> {
  block: TBlock;
  scoreVectorDense: number;
  scoreKeywordSparse: number;
}

export interface ScoredHybridCandidate<TBlock extends NarrativeBlock = NarrativeBlock>
  extends HybridCandidate<TBlock> {
  scoreRawFused: number;
  scoreFinalFused: number;
}

export interface NarrativeEntity {
  id: string;
  name: string;
  type: string;
  description?: string;
  properties?: Readonly<Record<string, unknown>>;
}

export interface NarrativeRepresentation {
  format: string;
  uri: string;
  name: string;
  entityName: string;
  id: string;
  description?: string;
  property?: string;
}

export interface NarrativeReference {
  sourceId: string;
  targetId: string;
  name?: string;
  uri?: string;
  description?: string;
}

export interface NarrativeRelationship {
  sourceId: string;
  targetId: string;
  type: string;
  description?: string;
}

export interface NarrativeEvent {
  id: string;
  name: string;
  happenedAt?: number;
  description?: string;
  entityIds?: readonly string[];
}

export interface PxEnrichment {
  entities?: readonly NarrativeEntity[];
  representations?: readonly NarrativeRepresentation[];
  references?: readonly NarrativeReference[];
  relationships?: readonly NarrativeRelationship[];
  eventHistory?: readonly NarrativeEvent[];
}

export interface RetrievalMetadata {
  providerType: string;
  totalBlockCount: number;
  historicalIndices: readonly number[];
  hybridCandidateCount: number;
  hybridSurvivorCount: number;
  notableEventCount: number;
  retrievedBlockCount: number;
}

export interface NarrativeContext<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
> {
  channelId: string;
  inputQuery: string;
  chronologicalBlocks: readonly TBlock[];
  loreAtoms: readonly TLore[];
  entities: readonly NarrativeEntity[];
  representations: readonly NarrativeRepresentation[];
  references: readonly NarrativeReference[];
  relationships: readonly NarrativeRelationship[];
  eventHistory: readonly NarrativeEvent[];
  metadata: RetrievalMetadata;
  warnings: readonly string[];
  prompt: string;
}

export interface BuildContextRequest {
  channelId: string;
  inputQuery: string;
}

export interface GenerateBlockRequest<TParameters = unknown> extends BuildContextRequest {
  parameters?: TParameters;
}

export interface GenerationProviderRequest<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
  TParameters = unknown,
> {
  context: NarrativeContext<TBlock, TLore>;
  parameters?: TParameters;
}

export interface GenerateBlockResult<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
> {
  block: TBlock;
  context: NarrativeContext<TBlock, TLore>;
}

export type PxErrorPolicy = "continue" | "fail";

/** One easy-to-read instruction in the block-retrieval recipe. */
export type BlockRetrievalStep =
  | { takeNewestBlocks: number }
  | { addNotableBlocksUntilThereAre: number }
  | { fillRemainingSpaceWithSearchResults: true };

/**
 * An ordered recipe for choosing story blocks. The engine deduplicates every
 * step, never exceeds `maximumBlocks`, and returns its result newest first.
 */
export interface BlockRetrievalConfig {
  /** The hard maximum number of blocks included in the final context. */
  maximumBlocks: number;
  /** Execute these retrieval instructions from top to bottom. */
  steps: readonly BlockRetrievalStep[];
  /** Final result order. Currently `newestFirst` is the supported order. */
  returnBlocks?: "newestFirst";
}

/** Controls the words NarrativeEngine uses when it renders its context. */
export interface ContextProseConfig {
  loreHeading?: string;
  historyHeading?: string;
  entitiesHeading?: string;
  entryLabel?: string;
  singularRelativeTimeUnit?: string;
  pluralRelativeTimeUnit?: string;
}

export type ContextRendererInput<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
> = Omit<NarrativeContext<TBlock, TLore>, "prompt">;

export interface NarrativeEngineConfig {
  /** Number of reciprocal-spaced historical samples to retrieve. */
  historicalSampleDivisions?: number;
  /** Block count required before reciprocal historical sampling begins. */
  minimumBlocksForHistoricalSampling?: number;
  /** Maximum hybrid-search candidates requested before relevance scoring. */
  hybridSearchCandidateLimit?: number;
  /** Maximum relevant hybrid-search results included in context. */
  maximumSearchResults?: number;
  /** Minimum score a hybrid-search candidate needs to be included. */
  minimumSearchRelevanceScore?: number;
  /** Portion of relevance score contributed by vector similarity (0–1). */
  vectorSearchWeight?: number;
  /** Score multiplier for a notable hybrid-search result. */
  notableSearchResultBonus?: number;
  /** Maximum active lore items included in context. */
  maximumLoreItems?: number;
  /** Maximum explicit notable blocks included by the legacy retrieval flow. */
  maximumNotableBlocks?: number;
  /** Maximum entity representations included in context. */
  maximumEntityRepresentations?: number;
  /** Ordered representation properties to prefer for each entity. */
  preferredEntityRepresentationProperties?: readonly string[];
  /** Maximum concurrent generation tasks in batch generation. */
  maximumConcurrentGenerationTasks?: number;
  /** Whether enrichment errors warn and continue or fail the request. */
  enrichmentFailureBehavior?: PxErrorPolicy;
  /** Whether history labels say how many beats ago a block occurred. */
  includeTemporalContextLabels?: boolean;
  /** An ordered, readable recipe for selecting blocks. */
  blockRetrieval?: BlockRetrievalConfig | undefined;
  /** Replace individual headings and labels in the rendered context. */
  contextProse?: ContextProseConfig;
  /** Take complete control of context rendering. */
  renderContext?: ((context: ContextRendererInput) => string) | undefined;

  /** @deprecated Use `historicalSampleDivisions`. */ reciprocalDivisions?: number;
  /** @deprecated Use `minimumBlocksForHistoricalSampling`. */ minimumBlocks?: number;
  /** @deprecated Use `hybridSearchCandidateLimit`. */ hybridCandidateLimit?: number;
  /** @deprecated Use `maximumSearchResults`. */ hybridTopK?: number;
  /** @deprecated Use `minimumSearchRelevanceScore`. */ saliencyThreshold?: number;
  /** @deprecated Use `vectorSearchWeight`. */ weightDense?: number;
  /** @deprecated Use `notableSearchResultBonus`. */ significanceCoefficient?: number;
  /** @deprecated Use `maximumLoreItems`. */ maxLoreAtoms?: number;
  /** @deprecated Use `maximumNotableBlocks`. */ maxNotableEvents?: number;
  /** @deprecated Use `maximumEntityRepresentations`. */ maxUniqueEntityRepresentations?: number;
  /** @deprecated Use `preferredEntityRepresentationProperties`. */ representationProperties?: readonly string[];
  /** @deprecated Use `maximumConcurrentGenerationTasks`. */ maxConcurrency?: number;
  /** @deprecated Use `enrichmentFailureBehavior`. */ pxErrorPolicy?: PxErrorPolicy;
  /** @deprecated Use `includeTemporalContextLabels`. */ temporalPhrasing?: boolean;
}

export interface ResolvedNarrativeEngineConfig {
  historicalSampleDivisions: number;
  minimumBlocksForHistoricalSampling: number;
  hybridSearchCandidateLimit: number;
  maximumSearchResults: number;
  minimumSearchRelevanceScore: number;
  vectorSearchWeight: number;
  notableSearchResultBonus: number;
  maximumLoreItems: number;
  maximumNotableBlocks: number;
  maximumEntityRepresentations: number;
  preferredEntityRepresentationProperties: readonly string[];
  maximumConcurrentGenerationTasks: number;
  enrichmentFailureBehavior: PxErrorPolicy;
  includeTemporalContextLabels: boolean;
  blockRetrieval?: BlockRetrievalConfig | undefined;
  contextProse: ContextProseConfig;
  renderContext?: ((context: ContextRendererInput) => string) | undefined;
}

export interface PxProviderRequest<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
> {
  channelId: string;
  inputQuery: string;
  chronologicalBlocks: readonly TBlock[];
  loreAtoms: readonly TLore[];
  representationProperties: readonly string[];
  maxUniqueEntityRepresentations: number;
}
