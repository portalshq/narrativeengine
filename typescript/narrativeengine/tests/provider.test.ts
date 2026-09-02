import { describe, expect, it } from "vitest";
import { InMemoryNarrativeProvider, type NarrativeBlock, type NarrativeLore } from "../src/index.js";

const blocks: NarrativeBlock[] = [
  { id: 1, index: 1, content: "The council opens the vault.", happenedAt: 100, isNotable: true },
  { id: 2, index: 2, content: "The crew leaves quietly.", happenedAt: 200, isNotable: false },
];

const lore: NarrativeLore[] = [
  { id: "active", content: "The vault predates the city.", happenedAt: 300, isActive: true },
  { id: "inactive", content: "This fact was retired.", happenedAt: 400, isActive: false },
];

describe("InMemoryNarrativeProvider", () => {
  it("implements every provider method", async () => {
    const provider = new InMemoryNarrativeProvider(blocks, lore);

    expect(provider.getProviderType()).toBe("in-memory");
    await expect(provider.getBlockCount("alpha")).resolves.toBe(2);
    await expect(provider.getLoreAtoms("alpha")).resolves.toEqual([lore[0]]);
    await expect(provider.getBlocksByIndices("alpha", [2])).resolves.toEqual([blocks[1]]);
    await expect(provider.getNotableEvents("alpha")).resolves.toEqual([blocks[0]]);

    const candidates = await provider.getHybridSearchCandidates("alpha", "council", 1);
    expect(candidates).toEqual([
      { block: blocks[0], scoreVectorDense: 0.8, scoreKeywordSparse: 0.8 },
    ]);

    const added = { id: 3, index: 3, content: "A new memory.", happenedAt: 500 };
    await provider.addBlock("alpha", added);
    await expect(provider.getBlockCount("alpha")).resolves.toBe(3);
  });
});
