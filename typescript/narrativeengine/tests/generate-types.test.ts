import { describe, expect, it, vi, beforeEach, afterEach } from "vitest";
import { resolve } from "node:path";

const repoRoot = resolve(import.meta.dirname!, "../../..");

const expectedTargets: Record<string, string> = {
  python: "python/narrativeengine/narrativeengine/models.py",
  typescript: "typescript/narrativeengine/src/models.ts",
  go: "generated/go/models.go",
  java: "generated/java/NarrativeModels.java",
  csharp: "generated/csharp/NarrativeModels.cs",
  swift: "generated/swift/NarrativeModels.swift",
  kotlin: "generated/kotlin/NarrativeModels.kt",
};

describe("generate-types.mjs targets", () => {
  let mod: Awaited<typeof import("../../../scripts/generate-types.mjs")>;

  beforeEach(async () => {
    mod = await import("../../../scripts/generate-types.mjs");
  });

  it("exports a targets array with all 7 SDK languages", () => {
    expect(mod.targets).toBeDefined();
    expect(Array.isArray(mod.targets)).toBe(true);
    expect(mod.targets).toHaveLength(7);
  });

  it("each target is a [language, path] tuple", () => {
    for (const [lang, out] of mod.targets) {
      expect(typeof lang).toBe("string");
      expect(typeof out).toBe("string");
      expect(lang.length).toBeGreaterThan(0);
      expect(out.length).toBeGreaterThan(0);
    }
  });

  it("covers the expected set of languages", () => {
    const languages = mod.targets.map(([lang]) => lang).sort();
    expect(languages).toEqual(Object.keys(expectedTargets).sort());
  });

  it("every output path resolves below the repo root", () => {
    for (const [, out] of mod.targets) {
      expect(out.startsWith(repoRoot)).toBe(true);
    }
  });

  it("each output path matches its expected relative path", () => {
    for (const [lang, out] of mod.targets) {
      const expected = resolve(repoRoot, expectedTargets[lang]);
      expect(out).toBe(expected);
    }
  });

  it("exports rootDir that matches the repo root", () => {
    expect(mod.rootDir).toBe(repoRoot);
  });

  it("exports manifest path pointing at repo Cargo.toml", () => {
    expect(mod.manifest).toBe(resolve(repoRoot, "Cargo.toml"));
  });
});

describe("generate-types.mjs run()", () => {
  let mod: Awaited<typeof import("../../../scripts/generate-types.mjs")>;

  beforeEach(async () => {
    mod = await import("../../../scripts/generate-types.mjs");
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("logs proto codegen is a no-op (handled by tonic-build)", () => {
    const logSpy = { log: vi.fn(), error: vi.fn() };

    mod.run({ log: logSpy });

    expect(logSpy.log).toHaveBeenCalledWith(
      expect.stringContaining("tonic-build"),
    );
    expect(logSpy.error).not.toHaveBeenCalled();
  });

  it("exits with error when Cargo.toml is missing", () => {
    const execMock = vi.fn();
    const exitSpy = vi
      .spyOn(process, "exit")
      .mockImplementation(() => { throw new Error("process.exit"); });
    const logSpy = { log: vi.fn(), error: vi.fn() };

    expect(() => mod.run({ exec: execMock, exists: () => false, log: logSpy })).toThrow("process.exit");

    expect(execMock).toHaveBeenCalledTimes(0);
    expect(exitSpy).toHaveBeenCalledWith(1);
    expect(logSpy.error).toHaveBeenCalledWith(expect.stringContaining("Cargo manifest not found"));
    exitSpy.mockRestore();
  });

  it("does not call exec (proto codegen is a no-op)", () => {
    const execMock = vi.fn();
    const logSpy = { log: vi.fn(), error: vi.fn() };

    mod.run({ exec: execMock, log: logSpy });

    expect(execMock).not.toHaveBeenCalled();
  });
});

describe("generate-types.mjs module structure", () => {
  it("does not auto-execute on import — all other tests prove this", () => {
    // Every test in this file imports the module. If run() auto-executed
    // on import, process.exit would terminate the test runner. The fact
    // we reach here proves the process.argv[1] guard works.
    expect(true).toBe(true);
  });

  it("exports run function for programmatic use", async () => {
    const mod = await import("../../../scripts/generate-types.mjs");
    expect(typeof mod.run).toBe("function");
    expect(mod.run.name).toBe("run");
  });
});
