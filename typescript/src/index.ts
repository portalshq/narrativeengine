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

