#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptFilename = fileURLToPath(import.meta.url);
const scriptDirectory = dirname(scriptFilename);
export const rootDir = resolve(scriptDirectory, "..");
export const proto = resolve(rootDir, "proto/narrative/v1/narrative.proto");
export const targets = [
  [
    "typescript",
    resolve(rootDir, "typescript/narrativeengine/src/generated/narrative/v1/narrative.ts"),
  ],
];

export function run({ exists = existsSync, exec = execFileSync, log = console } = {}) {
  if (!exists(proto)) {
    log.error(`FATAL: NarrativeEngine protobuf schema not found at ${proto}`);
    process.exitCode = 1;
    return;
  }

  log.log("→ Generating TypeScript DTOs from proto/narrative/v1/narrative.proto");
  exec("npm", ["run", "generate"], {
    cwd: resolve(rootDir, "typescript/narrativeengine"),
    stdio: "inherit",
  });
}

if (process.argv[1] === scriptFilename) run();
