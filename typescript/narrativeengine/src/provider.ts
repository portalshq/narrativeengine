export interface NarrativeBlock {
  id: string | number;
  index: number;
  content: string;
  happenedAt: number;
  isNotable?: boolean;
}

export interface NarrativeLore {
  id: string | number;
  content: string;
  happenedAt: number;
  isActive?: boolean;
}

export interface HybridCandidate<TBlock extends NarrativeBlock = NarrativeBlock> {
  block: TBlock;
  scoreVectorDense: number;
  scoreKeywordSparse: number;
}

/* eslint-disable no-unused-vars */
export interface NarrativeProvider<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
> {
  getBlockCount(channelId: string): Promise<number>;
  getLoreAtoms(channelId: string): Promise<TLore[]>;
  getHybridSearchCandidates(
    channelId: string,
    query: string,
    limit: number,
  ): Promise<HybridCandidate<TBlock>[]>;
  getBlocksByIndices(channelId: string, indices: number[]): Promise<TBlock[]>;
  getProviderType?(): string;
  getNotableEvents?(channelId: string): Promise<TBlock[]>;
  addBlock?(channelId: string, block: TBlock): Promise<void>;
}
/* eslint-enable no-unused-vars */

export class InMemoryNarrativeProvider<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
> implements NarrativeProvider<TBlock, TLore> {
  private readonly blocks: TBlock[];
  private readonly lore: TLore[];

  constructor(initialBlocks: TBlock[] = [], initialLore: TLore[] = []) {
    this.blocks = [...initialBlocks];
    this.lore = [...initialLore];
  }

  getProviderType(): string {
    return "in-memory";
  }

  async getBlockCount(_channelId: string): Promise<number> {
    void _channelId;
    return this.blocks.length;
  }

  async getLoreAtoms(_channelId: string): Promise<TLore[]> {
    void _channelId;
    return this.lore.filter((atom) => atom.isActive !== false);
  }

  async getHybridSearchCandidates(
    _channelId: string,
    query: string,
    limit: number,
  ): Promise<HybridCandidate<TBlock>[]> {
    void _channelId;
    const normalizedQuery = query.toLowerCase();
    return this.blocks
      .filter((block) => block.content.toLowerCase().includes(normalizedQuery))
      .slice(0, limit)
      .map((block) => ({
        block,
        scoreVectorDense: 0.8,
        scoreKeywordSparse: 0.8,
      }));
  }

  async getBlocksByIndices(_channelId: string, indices: number[]): Promise<TBlock[]> {
    void _channelId;
    return this.blocks.filter((block) => indices.includes(block.index));
  }

  async getNotableEvents(_channelId: string): Promise<TBlock[]> {
    void _channelId;
    return this.blocks.filter((block) => block.isNotable === true);
  }

  async addBlock(_channelId: string, block: TBlock): Promise<void> {
    void _channelId;
    this.blocks.push(block);
  }
}
