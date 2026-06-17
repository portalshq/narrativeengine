import { createRequire } from "node:module";

/* eslint-disable no-unused-vars */
interface NativeBindings {
  createBlockJson(id: string, content: string): string;
  generateCandidateJson(loreJson: string, configJson: string): string;
  renderLoreSummaryJson(loreJson: string): string;
  schemaBundleJson(): string;
  version(): string;
}
/* eslint-enable no-unused-vars */

const require = createRequire(import.meta.url);
const native = require("../index.cjs") as NativeBindings;

export default native;

