#!/usr/bin/env node

/**
 * Generates type definitions for all SDK languages from Rust schemas.
 * Now delegates to `just generate-protos`.
 *
 * Usage: node scripts/generate-types.mjs
 *
 * Also exported for testing:
 *   import { targets, rootDir, manifest, run } from "./generate-types.mjs";
 */

import { existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const __filename = fileURLToPath(import.meta.url);
export const rootDir = resolve(__dirname, "..");
export const manifest = resolve(rootDir, "Cargo.toml");

/** Language-to-output-path mapping for all SDK codegen targets. */
export const targets = [
  ["python", resolve(rootDir, "python/narrativeengine/narrativeengine/models.py")],
  ["typescript", resolve(rootDir, "typescript/narrativeengine/src/models.ts")],
  ["go", resolve(rootDir, "generated/go/models.go")],
  ["java", resolve(rootDir, "generated/java/NarrativeModels.java")],
  ["csharp", resolve(rootDir, "generated/csharp/NarrativeModels.cs")],
  ["swift", resolve(rootDir, "generated/swift/NarrativeModels.swift")],
  ["kotlin", resolve(rootDir, "generated/kotlin/NarrativeModels.kt")],
];

/**
 * Run codegen via just generate-protos.
 * @param {object} [opts] - Options for testing.
 */
export function run({ exists = existsSync, log = console } = {}) {
  if (!exists(manifest)) {
    const msg =
      `FATAL: Cargo manifest not found at ${manifest}\n` +
      "The script at scripts/generate-types.mjs could not locate Cargo.toml.\n" +
      "Ensure the script has not been moved relative to the repository root.";
    log.error(msg);
    process.exit(1);
  }

  // generate-protos is a no-op (proto codegen is handled by tonic-build in build.rs)
  log.log("→ Proto codegen: handled by tonic-build in build.rs. Nothing to generate.");
}

// Auto-execute when run as CLI, but not when imported for testing.
if (process.argv[1] === __filename) {
  run();
}
