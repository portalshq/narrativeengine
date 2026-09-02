import native from "./native.js";
export type { LabConfig } from "./models.js";
export type {
  HybridCandidate,
  NarrativeBlock,
  NarrativeLore,
  NarrativeProvider,
} from "./provider.js";
export { InMemoryNarrativeProvider } from "./provider.js";
import type {
  HybridCandidate as GeneratedHybridCandidate,
  LabConfig,
  NarrativeBlock as GeneratedNarrativeBlock,
  NarrativeLore as GeneratedNarrativeLore,
} from "./models.js";
import type {
  NarrativeBlock,
  NarrativeLore,
  NarrativeProvider,
} from "./provider.js";

export function createBlock(id: string, content: string): GeneratedNarrativeBlock {
  return JSON.parse(native.createBlockJson(id, content)) as GeneratedNarrativeBlock;
}

export function generateCandidate(
  lore: GeneratedNarrativeLore,
  config: LabConfig,
): GeneratedHybridCandidate {
  return JSON.parse(
    native.generateCandidateJson(JSON.stringify(lore), JSON.stringify(config)),
  ) as GeneratedHybridCandidate;
}

export function renderLoreSummary(lore: GeneratedNarrativeLore): string {
  return native.renderLoreSummaryJson(JSON.stringify(lore));
}

export function schemaBundle(): unknown {
  return JSON.parse(native.schemaBundleJson());
}

export function version(): string {
  return native.version();
}

// ─────────────────────────────────────────────────────────────────────────────
// NarrativeEngine class
// ─────────────────────────────────────────────────────────────────────────────

interface ContextPlan {
  historicalIndices: number[];
  candidateLimit: number;
}

export class NarrativeEngine<
  TBlock extends NarrativeBlock = NarrativeBlock,
  TLore extends NarrativeLore = NarrativeLore,
> {
  private readonly engine: InstanceType<typeof native.JsNarrativeEngine>;
  private readonly provider?: NarrativeProvider<TBlock, TLore>;

  constructor(provider?: NarrativeProvider<TBlock, TLore>) {
    this.engine = new native.JsNarrativeEngine();
    this.provider = provider;
  }

  async generateContext(channelId: string, query: string): Promise<string> {
    if (!this.provider) {
      return this.engine.generateContext(channelId, query);
    }

    const totalBlockCount = await this.provider.getBlockCount(channelId);
    const plan = JSON.parse(this.engine.planContext(totalBlockCount)) as ContextPlan;
    const [loreAtoms, candidatesHybrid, blocksHistorical] = await Promise.all([
      this.provider.getLoreAtoms(channelId),
      this.provider.getHybridSearchCandidates(channelId, query, plan.candidateLimit),
      plan.historicalIndices.length > 0
        ? this.provider.getBlocksByIndices(channelId, plan.historicalIndices)
        : Promise.resolve([] as TBlock[]),
    ]);

    return this.engine.generateContextFromData(JSON.stringify({
      channelId,
      inputQuery: query,
      totalBlockCount,
      loreAtoms,
      candidatesHybrid,
      blocksHistorical,
      providerType: this.provider.getProviderType?.() ?? "custom",
      blockSequenceIntervals: plan.historicalIndices,
    }));
  }

  generateBlock(channelId: string, inputQuery: string, parameters: any): any {
    return JSON.parse(
      this.engine.generateBlock(channelId, inputQuery, JSON.stringify(parameters))
    );
  }

  generateBlocksSequential(channelId: string, previousContext: string, options: any): any {
    return JSON.parse(
      this.engine.generateBlocksSequential(channelId, previousContext, JSON.stringify(options))
    );
  }

  generateBlocksParallel(channelId: string, branchContexts: string[], options: any): any {
    return JSON.parse(
      this.engine.generateBlocksParallel(channelId, branchContexts, JSON.stringify(options))
    );
  }

  setLabConfig(config: any): void {
    this.engine.setLabConfig(JSON.stringify(config));
  }

  getLabConfig(): any {
    return JSON.parse(this.engine.getLabConfig());
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Backward compatibility function
// ─────────────────────────────────────────────────────────────────────────────

export function configureLabEngine(engine: NarrativeEngine): void {
  void engine;
  // Placeholder for backward compatibility - no-op in new architecture
  // Lab configuration is now handled through internal engine settings
}
