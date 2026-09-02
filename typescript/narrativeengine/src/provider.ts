import type {
  GenerationProviderRequest,
  HybridCandidate,
  NarrativeBlock,
  NarrativeBlockInput,
  NarrativeLore,
  PxEnrichment,
  PxProviderRequest,
} from "./types.js";

export interface NarrativeDataProvider<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
  TBlockInput extends NarrativeBlockInput = NarrativeBlockInput,
> {
  getBlockCount(channelId: string): Promise<number>;
  getLoreAtoms(channelId: string): Promise<readonly TLore[]>;
  getHybridSearchCandidates(
    channelId: string,
    query: string,
    limit: number,
  ): Promise<readonly HybridCandidate<TBlock>[]>;
  getBlocksByIndices(channelId: string, indices: readonly number[]): Promise<readonly TBlock[]>;
  getNotableEvents(channelId: string): Promise<readonly TBlock[]>;
  insertBlock(channelId: string, block: TBlockInput): Promise<TBlock>;
  getProviderType(): string;
}

export type NarrativeProvider<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
  TBlockInput extends NarrativeBlockInput = NarrativeBlockInput,
> = NarrativeDataProvider<TBlock, TLore, TBlockInput>;

export interface GenerationProvider<
  TBlockInput extends NarrativeBlockInput = NarrativeBlockInput,
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
  TParameters = unknown,
> {
  generateBlock(
    request: GenerationProviderRequest<TBlock, TLore, TParameters>,
  ): Promise<TBlockInput>;
  generateBlocksBatch?(
    requests: readonly GenerationProviderRequest<TBlock, TLore, TParameters>[],
  ): Promise<readonly TBlockInput[]>;
}

export interface PxProvider<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
> {
  enrichContext(request: PxProviderRequest<TBlock, TLore>): Promise<PxEnrichment>;
}

export interface MemoryProviderOptions<
  TBlock extends NarrativeBlock,
  TLore extends NarrativeLore,
  TBlockInput extends NarrativeBlockInput,
> {
  channelId?: string;
  blocks?: readonly TBlock[];
  lore?: readonly TLore[];
  materializeBlock?: (
    channelId: string,
    block: TBlockInput,
    nextIndex: number,
  ) => TBlock;
  now?: () => number;
}

interface ChannelMemory<TBlock extends NarrativeBlock, TLore extends NarrativeLore> {
  blocks: TBlock[];
  lore: TLore[];
}

export class MemoryProvider<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
  TBlockInput extends NarrativeBlockInput = NarrativeBlockInput,
> implements NarrativeDataProvider<TBlock, TLore, TBlockInput>
{
  private readonly channels = new Map<string, ChannelMemory<TBlock, TLore>>();
  private readonly materializeBlock?: MemoryProviderOptions<
    TBlock,
    TLore,
    TBlockInput
  >["materializeBlock"];
  private readonly now: () => number;

  constructor(
    optionsOrBlocks: MemoryProviderOptions<TBlock, TLore, TBlockInput> | readonly TBlock[] = {},
    legacyLore: readonly TLore[] = [],
  ) {
    const options: MemoryProviderOptions<TBlock, TLore, TBlockInput> = Array.isArray(optionsOrBlocks)
      ? { blocks: optionsOrBlocks, lore: legacyLore }
      : (optionsOrBlocks as MemoryProviderOptions<TBlock, TLore, TBlockInput>);
    this.materializeBlock = options.materializeBlock;
    this.now = options.now ?? Date.now;
    const channelId = options.channelId ?? "default";
    this.channels.set(channelId, {
      blocks: [...(options.blocks ?? [])],
      lore: [...(options.lore ?? [])],
    });
  }

  getProviderType(): string {
    return "memory";
  }

  async getBlockCount(channelId: string): Promise<number> {
    return this.getChannel(channelId).blocks.length;
  }

  async getLoreAtoms(channelId: string): Promise<readonly TLore[]> {
    return this.getChannel(channelId).lore.filter((atom) => atom.isActive !== false);
  }

  async getHybridSearchCandidates(
    channelId: string,
    query: string,
    limit: number,
  ): Promise<readonly HybridCandidate<TBlock>[]> {
    const normalizedQuery = query.trim().toLowerCase();
    if (normalizedQuery.length === 0 || limit <= 0) return [];

    return this.getChannel(channelId)
      .blocks.filter((block) => block.content.toLowerCase().includes(normalizedQuery))
      .slice(0, limit)
      .map((block) => ({
        block,
        scoreVectorDense: 0.8,
        scoreKeywordSparse: 0.8,
      }));
  }

  async getBlocksByIndices(
    channelId: string,
    indices: readonly number[],
  ): Promise<readonly TBlock[]> {
    const requested = new Set(indices);
    return this.getChannel(channelId).blocks.filter((block) => requested.has(block.index));
  }

  async getNotableEvents(channelId: string): Promise<readonly TBlock[]> {
    return this.getChannel(channelId).blocks.filter((block) => block.isNotable === true);
  }

  async insertBlock(channelId: string, input: TBlockInput): Promise<TBlock> {
    const channel = this.getChannel(channelId);
    const nextIndex = Math.max(0, ...channel.blocks.map((block) => block.index)) + 1;
    const block = this.materializeBlock
      ? this.materializeBlock(channelId, input, nextIndex)
      : ({
          ...input,
          id: input.id ?? nextIndex,
          index: input.index ?? nextIndex,
          happenedAt: input.happenedAt ?? this.now(),
        } as unknown as TBlock);
    channel.blocks.push(block);
    return block;
  }

  seedChannel(channelId: string, blocks: readonly TBlock[], lore: readonly TLore[] = []): void {
    this.channels.set(channelId, { blocks: [...blocks], lore: [...lore] });
  }

  private getChannel(channelId: string): ChannelMemory<TBlock, TLore> {
    const existing = this.channels.get(channelId);
    if (existing) return existing;
    const created: ChannelMemory<TBlock, TLore> = { blocks: [], lore: [] };
    this.channels.set(channelId, created);
    return created;
  }
}

export { MemoryProvider as InMemoryNarrativeProvider };
