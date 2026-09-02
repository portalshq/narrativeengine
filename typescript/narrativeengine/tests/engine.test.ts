import { describe, expect, it, vi } from "vitest";
import {
  BatchGenerationError,
  InMemoryBlockCache,
  NarrativeEngine,
  NarrativeEngineError,
  generateHistoricalIndices,
  type GenerationProvider,
  type NarrativeBlock,
  type NarrativeBlockInput,
  type NarrativeDataProvider,
  type NarrativeLore,
  type PxProvider,
} from "../src/index.js";

function makeBlock(index: number, content = `Block ${index}`, notable = false): NarrativeBlock {
  return { id: index, index, content, happenedAt: index * 100, isNotable: notable };
}

function makeProvider(overrides: Partial<NarrativeDataProvider> = {}): NarrativeDataProvider {
  return {
    getProviderType: () => "test",
    getBlockCount: vi.fn().mockResolvedValue(0),
    getLoreAtoms: vi.fn().mockResolvedValue([]),
    getHybridSearchCandidates: vi.fn().mockResolvedValue([]),
    getBlocksByIndices: vi.fn().mockResolvedValue([]),
    getNotableEvents: vi.fn().mockResolvedValue([]),
    insertBlock: vi.fn(async (_channelId: string, draft: NarrativeBlockInput) => ({
      ...draft,
      id: draft.id ?? 1,
      index: draft.index ?? 1,
      happenedAt: draft.happenedAt ?? 1,
    })) as NarrativeDataProvider["insertBlock"],
    ...overrides,
  };
}

describe("NarrativeEngine retrieval", () => {
  it("applies lore caps, fusion, notable boosts, saliency, deduplication, and newest-first ordering", async () => {
    const notableCandidate = makeBlock(2, "Notable relevant", true);
    const rejectedCandidate = makeBlock(3, "Ordinary weak");
    const notableEvent = makeBlock(4, "Explicit event", true);
    const lore: NarrativeLore[] = [
      { id: "old", content: "Old lore", happenedAt: 1 },
      { id: "new", content: "New lore", happenedAt: 2 },
      { id: "inactive", content: "Inactive", happenedAt: 3, isActive: false },
    ];
    const provider = makeProvider({
      getLoreAtoms: vi.fn().mockResolvedValue(lore),
      getHybridSearchCandidates: vi.fn().mockResolvedValue([
        { block: rejectedCandidate, scoreVectorDense: 0.5, scoreKeywordSparse: 0.5 },
        { block: notableCandidate, scoreVectorDense: 0.5, scoreKeywordSparse: 0.5 },
      ]),
      getNotableEvents: vi.fn().mockResolvedValue([notableEvent, notableCandidate]),
    });
    const engine = new NarrativeEngine({ dataProvider: provider, config: { maxLoreAtoms: 1 } });

    const context = await engine.buildContext({ channelId: "alpha", inputQuery: "Continue" });

    expect(context.loreAtoms).toEqual([lore[1]]);
    expect(context.chronologicalBlocks.map((block) => block.id)).toEqual([4, 2]);
    expect(context.metadata.hybridSurvivorCount).toBe(1);
    expect(context.metadata.notableEventCount).toBe(2);
    expect(context.prompt.endsWith("Continue")).toBe(true);
  });

  it("returns blocks newest-first while composing the prompt oldest-first", async () => {
    const blocks = [makeBlock(1), makeBlock(2), makeBlock(3)];
    const provider = makeProvider({
      getBlockCount: vi.fn().mockResolvedValue(3),
      getBlocksByIndices: vi.fn(async (_channelId: string, indices: readonly number[]) =>
        blocks.filter((block) => indices.includes(block.index)),
      ),
    });
    const engine = new NarrativeEngine({ dataProvider: provider });
    const context = await engine.buildContext({ channelId: "alpha", inputQuery: "Next" });

    expect(context.chronologicalBlocks.map((block) => block.index)).toEqual([3, 2, 1]);
    expect(context.prompt.indexOf("Block 1")).toBeLessThan(context.prompt.indexOf("Block 2"));
    expect(context.prompt.indexOf("Block 2")).toBeLessThan(context.prompt.indexOf("Block 3"));
  });
});

describe("NarrativeEngine PX and generation", () => {
  it("enriches before generation, applies ordered representation fallback, persists, and returns the envelope", async () => {
    const order: string[] = [];
    const provider = makeProvider({
      insertBlock: vi.fn(async (_channelId, draft) => {
        order.push("insert");
        return { ...draft, id: 10, index: 10, happenedAt: 1_000 } as NarrativeBlock;
      }),
    });
    const pxProvider: PxProvider = {
      enrichContext: vi.fn(async () => {
        order.push("px");
        return {
          entities: [
            { id: "a", name: "Aria", type: "character", description: "A navigator" },
            { id: "b", name: "Bex", type: "character" },
          ],
          representations: [
            { id: "a-sheet", entityName: "Aria", name: "sheet", format: "png", uri: "https://signed/a-sheet", property: "sheet" },
            { id: "a-avatar", entityName: "Aria", name: "avatar", format: "png", uri: "https://signed/a-avatar", property: "avatar" },
            { id: "b-sheet", entityName: "Bex", name: "sheet", format: "png", uri: "https://signed/b-sheet", property: "sheet" },
          ],
          references: [{ sourceId: "a", targetId: "b" }],
          relationships: [{ sourceId: "a", targetId: "b", type: "ally" }],
          eventHistory: [{ id: "event", name: "First meeting" }],
        };
      }),
    };
    const generationProvider: GenerationProvider = {
      generateBlock: vi.fn(async ({ context }) => {
        order.push("generate");
        expect(context.representations.map((representation) => representation.id)).toEqual([
          "a-avatar",
          "b-sheet",
        ]);
        expect(context.relationships).toHaveLength(1);
        expect(context.eventHistory).toHaveLength(1);
        return { content: "Generated" };
      }),
    };
    const engine = new NarrativeEngine({
      dataProvider: provider,
      pxProvider,
      generationProvider,
      config: { representationProperties: ["avatar", "sheet"], maxUniqueEntityRepresentations: 2 },
    });

    const result = await engine.generateBlock({ channelId: "alpha", inputQuery: "Continue" });

    expect(order).toEqual(["px", "generate", "insert"]);
    expect(result.block).toMatchObject({ id: 10, content: "Generated" });
    expect(result.context.prompt).toContain("Structured entity context");
  });

  it("continues with a warning by default and can fail hard on PX errors", async () => {
    const pxProvider: PxProvider = { enrichContext: vi.fn().mockRejectedValue(new Error("offline")) };
    const provider = makeProvider();
    const continuing = new NarrativeEngine({ dataProvider: provider, pxProvider });
    const context = await continuing.buildContext({ channelId: "alpha", inputQuery: "Next" });
    expect(context.warnings).toEqual(["PX context enrichment failed: offline"]);

    const failing = new NarrativeEngine({
      dataProvider: provider,
      pxProvider,
      config: { pxErrorPolicy: "fail" },
    });
    await expect(failing.buildContext({ channelId: "alpha", inputQuery: "Next" })).rejects.toMatchObject({
      code: "PX_FAILED",
    });
  });

  it("requires a generation provider only for generation workflows", async () => {
    const engine = new NarrativeEngine(makeProvider());
    await expect(engine.generateContext("alpha", "Immediate")).resolves.toBe("Immediate");
    await expect(engine.generateBlock({ channelId: "alpha", inputQuery: "Immediate" })).rejects.toBeInstanceOf(
      NarrativeEngineError,
    );
  });
});

describe("NarrativeEngine block cache", () => {
  it("serves later requests from cache and queries only newly requested indices", async () => {
    let totalBlocks = 9;
    const allBlocks = Array.from({ length: 12 }, (_, index) => makeBlock(index + 1));
    const getBlocksByIndices = vi.fn(async (_channelId: string, indices: readonly number[]) =>
      allBlocks.filter((block) => indices.includes(block.index)),
    );
    const provider = makeProvider({
      getBlockCount: vi.fn(async () => totalBlocks),
      getBlocksByIndices,
    });
    const engine = new NarrativeEngine({ dataProvider: provider });

    await engine.buildContext({ channelId: "alpha", inputQuery: "one" });
    const firstIndices = generateHistoricalIndices(9, 5);
    expect(getBlocksByIndices).toHaveBeenLastCalledWith("alpha", firstIndices);
    await engine.buildContext({ channelId: "alpha", inputQuery: "two" });
    expect(getBlocksByIndices).toHaveBeenCalledTimes(1);

    totalBlocks = 12;
    await engine.buildContext({ channelId: "alpha", inputQuery: "three" });
    const nextIndices = generateHistoricalIndices(12, 5);
    expect(getBlocksByIndices).toHaveBeenCalledTimes(2);
    expect(getBlocksByIndices).toHaveBeenLastCalledWith(
      "alpha",
      nextIndices.filter((index) => !firstIndices.includes(index)),
    );
  });

  it("refetches after TTL, isolates channels, and supports explicit invalidation", async () => {
    let now = 0;
    const getBlocksByIndices = vi.fn(async (_channelId: string, indices: readonly number[]) =>
      indices.map((index) => makeBlock(index)),
    );
    const provider = makeProvider({
      getBlockCount: vi.fn().mockResolvedValue(3),
      getBlocksByIndices,
    });
    const cache = new InMemoryBlockCache<NarrativeBlock>({ ttlMs: 100, now: () => now });
    const engine = new NarrativeEngine({ dataProvider: provider, blockCache: cache });

    await engine.buildContext({ channelId: "alpha", inputQuery: "one" });
    await engine.buildContext({ channelId: "beta", inputQuery: "one" });
    expect(getBlocksByIndices).toHaveBeenCalledTimes(2);
    now = 100;
    await engine.buildContext({ channelId: "alpha", inputQuery: "two" });
    expect(getBlocksByIndices).toHaveBeenCalledTimes(3);
    engine.invalidateChannel("alpha");
    await engine.buildContext({ channelId: "alpha", inputQuery: "three" });
    expect(getBlocksByIndices).toHaveBeenCalledTimes(4);
  });

  it("coalesces concurrent indexed-block misses into one provider batch", async () => {
    const getBlocksByIndices = vi.fn(async (_channelId: string, indices: readonly number[]) => {
      await Promise.resolve();
      return indices.map((index) => makeBlock(index));
    });
    const provider = makeProvider({
      getBlockCount: vi.fn().mockResolvedValue(9),
      getBlocksByIndices,
    });
    const engine = new NarrativeEngine({ dataProvider: provider });

    await Promise.all([
      engine.buildContext({ channelId: "alpha", inputQuery: "one" }),
      engine.buildContext({ channelId: "alpha", inputQuery: "two" }),
    ]);
    expect(getBlocksByIndices).toHaveBeenCalledTimes(1);
  });
});

describe("NarrativeEngine batch generation", () => {
  it("uses optimized batch generation and persists results in request order", async () => {
    const persisted: string[] = [];
    const provider = makeProvider({
      insertBlock: vi.fn(async (_channelId, draft) => {
        persisted.push(draft.content);
        const index = persisted.length;
        return { ...draft, id: index, index, happenedAt: index } as NarrativeBlock;
      }),
    });
    const generationProvider: GenerationProvider = {
      generateBlock: vi.fn().mockRejectedValue(new Error("single path should not run")),
      generateBlocksBatch: vi.fn(async (requests) =>
        requests.map((request) => ({ content: `Generated ${request.context.channelId}` })),
      ),
    };
    const engine = new NarrativeEngine({ dataProvider: provider, generationProvider });
    const results = await engine.generateBlocksBatch([
      { channelId: "a", inputQuery: "A" },
      { channelId: "b", inputQuery: "B" },
    ]);

    expect(generationProvider.generateBlocksBatch).toHaveBeenCalledTimes(1);
    expect(persisted).toEqual(["Generated a", "Generated b"]);
    expect(results.map((result) => result.block.content)).toEqual(["Generated a", "Generated b"]);
  });

  it("reports completed results when persistence fails partway through", async () => {
    let calls = 0;
    const provider = makeProvider({
      insertBlock: vi.fn(async (_channelId, draft) => {
        calls += 1;
        if (calls === 2) throw new Error("database unavailable");
        return { ...draft, id: calls, index: calls, happenedAt: calls } as NarrativeBlock;
      }),
    });
    const generationProvider: GenerationProvider = {
      generateBlock: vi.fn(async ({ context }) => ({ content: context.inputQuery })),
    };
    const engine = new NarrativeEngine({ dataProvider: provider, generationProvider });

    try {
      await engine.generateBlocksBatch([
        { channelId: "a", inputQuery: "A" },
        { channelId: "b", inputQuery: "B" },
      ]);
      throw new Error("Expected batch generation to fail.");
    } catch (error) {
      expect(error).toBeInstanceOf(BatchGenerationError);
      expect((error as BatchGenerationError).completed).toHaveLength(1);
    }
  });
});
