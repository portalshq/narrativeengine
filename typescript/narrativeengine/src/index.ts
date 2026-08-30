import native from "./native.js";
export type { HybridCandidate, LabConfig, NarrativeBlock, NarrativeLore } from "./models.js";
import type { HybridCandidate, LabConfig, NarrativeBlock, NarrativeLore } from "./models.js";

export function createBlock(id: string, content: string): NarrativeBlock {
  return JSON.parse(native.createBlockJson(id, content)) as NarrativeBlock;
}

export function generateCandidate(lore: NarrativeLore, config: LabConfig): HybridCandidate {
  return JSON.parse(
    native.generateCandidateJson(JSON.stringify(lore), JSON.stringify(config)),
  ) as HybridCandidate;
}

export function renderLoreSummary(lore: NarrativeLore): string {
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

export class NarrativeEngine {
  private engine: any;

  constructor(provider?: any) {
    // Support both old API (with provider) and new API (no args)
    this.engine = new native.JsNarrativeEngine();
  }

  generateContext(channelId: string, query: string): string {
    return this.engine.generateContext(channelId, query);
  }
}

// ─────────────────────────────────────────────────────────────────────────────
// Backward compatibility function
// ─────────────────────────────────────────────────────────────────────────────

export function configureLabEngine(engine: NarrativeEngine): void {
  // Placeholder for backward compatibility - no-op in new architecture
  // Lab configuration is now handled through internal engine settings
}

