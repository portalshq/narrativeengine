interface BaseNarrativeBlock {
    id: string | number;
    index: number;
    content: string;
    happenedAt: number;
    isNotable?: boolean;
}
interface BaseNarrativeLore {
    id: string | number;
    content: string;
    happenedAt: number;
    isActive?: boolean;
}

interface HybridCandidate<TBlock extends BaseNarrativeBlock> {
    block: TBlock;
    scoreVectorDense: number;
    scoreKeywordSparse: number;
}
interface NarrativeProvider<TBlock extends BaseNarrativeBlock = BaseNarrativeBlock, TLore extends BaseNarrativeLore = BaseNarrativeLore> {
    getBlockCount(channelId: string): Promise<number>;
    getLoreAtoms(channelId: string): Promise<TLore[]>;
    getHybridSearchCandidates(channelId: string, query: string, limit: number): Promise<HybridCandidate<TBlock>[]>;
    getBlocksByIndices(channelId: string, indices: number[]): Promise<TBlock[]>;
    getNotableEvents(channelId: string): Promise<TBlock[]>;
    addBlock?(channelId: string, block: TBlock): Promise<void>;
    getProviderType?(): string;
}
/**
 * A Zero-Dependency In-Memory Provider for testing and local development.
 * Supports persisting blocks to browser localStorage.
 */
declare class InMemoryNarrativeProvider<TBlock extends BaseNarrativeBlock, TLore extends BaseNarrativeLore> implements NarrativeProvider<TBlock, TLore> {
    private blocks;
    private lore;
    private storageKeyBlocks;
    private storageKeyLore;
    private useBrowserStorage;
    constructor(initialBlocks?: TBlock[], initialLore?: TLore[], options?: {
        useBrowserStorage?: boolean;
        channelId?: string;
    });
    getProviderType(): string;
    getLoreAtoms(_channelId: string): Promise<TLore[]>;
    getNotableEvents(_channelId: string): Promise<TBlock[]>;
    getBlocksByIndices(_channelId: string, indices: number[]): Promise<TBlock[]>;
    getBlockCount(_channelId: string): Promise<number>;
    getHybridSearchCandidates(_channelId: string, query: string, limit: number): Promise<HybridCandidate<TBlock>[]>;
    addBlock(_channelId: string, block: TBlock): Promise<void>;
}

interface LabConfig {
    saliencyThreshold?: number;
    weightDense?: number;
    significanceCoef?: number;
    temporalPhrasing?: boolean;
    maxLoreAtoms?: number;
    timestamp?: string | null;
}
declare class NarrativeEngine<TBlock extends BaseNarrativeBlock = BaseNarrativeBlock, TLore extends BaseNarrativeLore = BaseNarrativeLore> {
    private provider;
    private labConfig;
    constructor(provider?: NarrativeProvider<TBlock, TLore>);
    setLabConfig(config: LabConfig): void;
    getLabConfig(): Required<LabConfig>;
    generateContext(channelId: string, inputQuery: string): Promise<string>;
    private mergeAndSortChronologically;
    private composeProse;
}

/**
 * Normalizes a value from a specific range to the engine-required 0.0 - 1.0 range.
 */
declare function normalizeScore(value: number, min: number, max: number): number;
/**
 * Runtime validation for the NarrativeProvider.
 * Ensures the passed object has the required functional 'shape'.
 */
declare function validateProviderShape(provider: any): boolean;

declare const GLOBAL_KEY: unique symbol;
declare const LAB_TOKEN: unique symbol;
declare const SESSION_SECRET: any;
declare function configureLabEngine(engine: NarrativeEngine): void;
declare function getActiveEngine(): NarrativeEngine | undefined;

export { type BaseNarrativeBlock, type BaseNarrativeLore, GLOBAL_KEY, type HybridCandidate, InMemoryNarrativeProvider, LAB_TOKEN, type LabConfig, NarrativeEngine, type NarrativeProvider, SESSION_SECRET, configureLabEngine, getActiveEngine, normalizeScore, validateProviderShape };
