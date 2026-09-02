import { createRequire } from "node:module";

/* eslint-disable no-unused-vars */
interface NativeBindings {
  createBlockJson(id: string, content: string): string;
  generateCandidateJson(loreJson: string, configJson: string): string;
  renderLoreSummaryJson(loreJson: string): string;
  schemaBundleJson(): string;
  version(): string;
}

interface JsNarrativeEngineConstructor {
  new (): JsNarrativeEngine;
}

interface JsNarrativeEngine {
  generateContext(channelId: string, query: string): string;
  planContext(totalBlockCount: number): string;
  generateContextFromData(inputJson: string): string;
  generateBlock(channelId: string, inputQuery: string, parametersJson: string): string;
  generateBlocksSequential(channelId: string, previousContext: string, optionsJson: string): string;
  generateBlocksParallel(channelId: string, branchContexts: string[], optionsJson: string): string;
  setLabConfig(configJson: string): void;
  getLabConfig(): string;
  version(): string;
}

interface NativeModule extends NativeBindings {
  JsNarrativeEngine: JsNarrativeEngineConstructor;
}
/* eslint-enable no-unused-vars */

const require = createRequire(import.meta.url);
const native = require("../index.cjs") as NativeModule;

export default native;
