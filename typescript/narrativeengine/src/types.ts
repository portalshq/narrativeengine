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

export interface NarrativeEngineConfig {
  reciprocalDivisions?: number;
  minimumBlocks?: number;
  hybridCandidateLimit?: number;
  hybridTopK?: number;
  saliencyThreshold?: number;
  weightDense?: number;
  significanceCoefficient?: number;
  maxLoreAtoms?: number;
  maxNotableEvents?: number;
  maxUniqueEntityRepresentations?: number;
  representationProperties?: readonly string[];
  maxConcurrency?: number;
  pxErrorPolicy?: PxErrorPolicy;
  temporalPhrasing?: boolean;
}

export interface ResolvedNarrativeEngineConfig {
  reciprocalDivisions: number;
  minimumBlocks: number;
  hybridCandidateLimit: number;
  hybridTopK: number;
  saliencyThreshold: number;
  weightDense: number;
  significanceCoefficient: number;
  maxLoreAtoms: number;
  maxNotableEvents: number;
  maxUniqueEntityRepresentations: number;
  representationProperties: readonly string[];
  maxConcurrency: number;
  pxErrorPolicy: PxErrorPolicy;
  temporalPhrasing: boolean;
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
