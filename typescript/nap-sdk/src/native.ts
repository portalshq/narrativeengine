import { createRequire } from "node:module";

interface NativeBindings {
  parseUri(uri: string): string;
  parseManifest(yamlStr: string): string;
  resolve(uri: string, repoPath: string): string;
  version(): string;
}

const require = createRequire(import.meta.url);
const native = require("../index.cjs") as NativeBindings;

export default native;
