import { describe, expect, it } from "vitest";
import { InMemoryBlockCache, type NarrativeBlock } from "../src/index.js";

function block(index: number): NarrativeBlock {
  return { id: index, index, content: `Block ${index}`, happenedAt: index };
}

describe("InMemoryBlockCache", () => {
  it("persists entries across calls until fixed TTL expiration", () => {
    let now = 0;
    const cache = new InMemoryBlockCache({ ttlMs: 100, now: () => now });
    cache.set("alpha", 5, block(5));
    now = 50;
    expect(cache.get("alpha", 5)).toEqual(block(5));
    now = 100;
    expect(cache.get("alpha", 5)).toBeUndefined();
  });

  it("isolates channels and evicts the least recently used entry", () => {
    let now = 0;
    const cache = new InMemoryBlockCache({ ttlMs: 1_000, maxEntries: 2, now: () => now });
    cache.set("alpha", 1, block(1));
    now += 1;
    cache.set("alpha", 2, block(2));
    now += 1;
    expect(cache.get("alpha", 1)).toEqual(block(1));
    cache.set("alpha", 3, block(3));
    expect(cache.get("alpha", 2)).toBeUndefined();
    expect(cache.get("alpha", 1)).toEqual(block(1));
    expect(cache.get("beta", 1)).toBeUndefined();
  });

  it("supports targeted, channel, and global invalidation", () => {
    const cache = new InMemoryBlockCache<NarrativeBlock>();
    cache.set("alpha", 1, block(1));
    cache.set("alpha", 2, block(2));
    cache.set("beta", 1, block(1));
    cache.invalidate("alpha", 1);
    expect(cache.get("alpha", 1)).toBeUndefined();
    cache.invalidateChannel("alpha");
    expect(cache.get("alpha", 2)).toBeUndefined();
    expect(cache.get("beta", 1)).toBeDefined();
    cache.clear();
    expect(cache.get("beta", 1)).toBeUndefined();
  });

  it("rejects invalid cache limits", () => {
    expect(() => new InMemoryBlockCache({ ttlMs: 0 })).toThrow(RangeError);
    expect(() => new InMemoryBlockCache({ maxEntries: 0 })).toThrow(RangeError);
  });
});
