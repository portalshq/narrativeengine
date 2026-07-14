import { defineConfig } from "tsup";

export default defineConfig({
  entry: {
    server: "lab/server.ts",
    "configure-lab-engine": "lab/configure-lab-engine.ts",
  },
  format: ["esm"],
  outDir: "dist/lab",
  platform: "node",
  external: ["express", "cors", "path", "fs", "crypto", "url", "narrative-engine"],
});