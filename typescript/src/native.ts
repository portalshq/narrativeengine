import { createRequire } from "node:module";

interface NativeBindings {
  createBlockJson(id: string, content: string): string;
  generateCandidateJson(loreJson: string, configJson: string): string;
  renderLoreSummaryJson(loreJson: string): string;
  schemaBundleJson(): string;
  version(): string;
}

const require = createRequire(import.meta.url);
const native = require("../index.cjs") as NativeBindings;

export default native;

