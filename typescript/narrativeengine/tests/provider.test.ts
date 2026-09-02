import { describe, expect, it } from "vitest";
import {
  InMemoryNarrativeProvider,
  MemoryProvider,
  type NarrativeBlock,
  type NarrativeLore,
} from "../src/index.js";

const blocks: NarrativeBlock[] = [
  { id: 1, index: 1, content: "The council opens the vault.", happenedAt: 100, isNotable: true },
  { id: 2, index: 2, content: "The crew leaves quietly.", happenedAt: 200 },
];

const lore: NarrativeLore[] = [
  { id: "active", content: "The vault predates the city.", happenedAt: 300, isActive: true },
  { id: "inactive", content: "This fact was retired.", happenedAt: 400, isActive: false },
];

describe("MemoryProvider", () => {
  it("implements every provider method with channel isolation and authoritative insertion", async () => {
    const provider = new MemoryProvider({ channelId: "alpha", blocks, lore, now: () => 500 });
    provider.seedChannel("beta", [{ id: 8, index: 1, content: "Beta only.", happenedAt: 50 }]);

    expect(provider.getProviderType()).toBe("memory");
    await expect(provider.getBlockCount("alpha")).resolves.toBe(2);
    await expect(provider.getBlockCount("beta")).resolves.toBe(1);
    await expect(provider.getLoreAtoms("alpha")).resolves.toEqual([lore[0]]);
    await expect(provider.getBlocksByIndices("alpha", [2])).resolves.toEqual([blocks[1]]);
    await expect(provider.getNotableEvents("alpha")).resolves.toEqual([blocks[0]]);
    await expect(provider.getHybridSearchCandidates("alpha", "council", 1)).resolves.toEqual([
      { block: blocks[0], scoreVectorDense: 0.8, scoreKeywordSparse: 0.8 },
    ]);
    await expect(provider.getHybridSearchCandidates("alpha", "", 10)).resolves.toEqual([]);

    const inserted = await provider.insertBlock("alpha", { content: "A new memory." });
    expect(inserted).toMatchObject({ id: 3, index: 3, content: "A new memory.", happenedAt: 500 });
    await expect(provider.getBlockCount("alpha")).resolves.toBe(3);
    await expect(provider.getBlockCount("beta")).resolves.toBe(1);
  });

  it("retains the compatibility provider alias", () => {
    expect(InMemoryNarrativeProvider).toBe(MemoryProvider);
  });

  it("accepts the legacy blocks-and-lore constructor", async () => {
    const provider = new InMemoryNarrativeProvider(blocks, lore);
    await expect(provider.getBlockCount("default")).resolves.toBe(2);
    await expect(provider.getLoreAtoms("default")).resolves.toEqual([lore[0]]);
  });
});
