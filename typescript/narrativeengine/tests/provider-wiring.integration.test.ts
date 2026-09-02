import { describe, expect, it, vi } from "vitest";
import {
  NarrativeEngine,
  type NarrativeBlock,
  type NarrativeLore,
  type NarrativeProvider,
} from "../src/index.js";

describe("NarrativeEngine provider wiring", () => {
  it("retrieves through the JavaScript provider and finalizes in Rust", async () => {
    const historical: NarrativeBlock = {
      id: 1,
      index: 1,
      content: "The council formed beneath the observatory.",
      happenedAt: 100,
      isNotable: false,
    };
    const candidate: NarrativeBlock = {
      id: 4,
      index: 4,
      content: "A hidden vote changed the succession.",
      happenedAt: 400,
      isNotable: true,
    };
    const lore: NarrativeLore = {
      id: "lore-1",
      content: "The observatory is forbidden after dusk.",
      happenedAt: 500,
      isActive: true,
    };
    const provider: NarrativeProvider = {
      getBlockCount: vi.fn().mockResolvedValue(4),
      getLoreAtoms: vi.fn().mockResolvedValue([lore]),
      getHybridSearchCandidates: vi.fn().mockResolvedValue([
        { block: candidate, scoreVectorDense: 0.5, scoreKeywordSparse: 0.5 },
      ]),
      getBlocksByIndices: vi.fn().mockResolvedValue([historical]),
      getProviderType: () => "sqlite-test",
    };
    const engine = new NarrativeEngine(provider);

    const result = await engine.generateContext("alpha", "The council returns.");

    expect(provider.getBlockCount).toHaveBeenCalledWith("alpha");
    expect(provider.getLoreAtoms).toHaveBeenCalledWith("alpha");
    expect(provider.getHybridSearchCandidates).toHaveBeenCalledWith("alpha", "The council returns.", 20);
    expect(provider.getBlocksByIndices).toHaveBeenCalledWith("alpha", expect.arrayContaining([1, 4]));
    expect(result).toContain(lore.content);
    expect(result).toContain(historical.content);
    expect(result).toContain(candidate.content);
    expect(result.endsWith("The council returns.")).toBe(true);
  });

  it("preserves the empty native-provider fallback when no provider is supplied", async () => {
    const engine = new NarrativeEngine();

    await expect(engine.generateContext("alpha", "Immediate context")).resolves.toBe(
      "Immediate context",
    );
  });
});
