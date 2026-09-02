import type { NarrativeBlock } from "./types.js";

export interface BlockCache<TBlock extends NarrativeBlock = NarrativeBlock> {
  get(channelId: string, index: number): TBlock | undefined;
  set(channelId: string, index: number, block: TBlock): void;
  invalidate(channelId: string, index: number): void;
  invalidateChannel(channelId: string): void;
  clear(): void;
}

export interface InMemoryBlockCacheOptions {
  ttlMs?: number;
  maxEntries?: number;
  now?: () => number;
}

interface CacheEntry<TBlock extends NarrativeBlock> {
  channelId: string;
  index: number;
  block: TBlock;
  expiresAt: number;
  lastAccessedAt: number;
}

const DEFAULT_TTL_MS = 5 * 60 * 1_000;
const DEFAULT_MAX_ENTRIES = 10_000;

function cacheKey(channelId: string, index: number): string {
  return JSON.stringify([channelId, index]);
}

export class InMemoryBlockCache<TBlock extends NarrativeBlock = NarrativeBlock>
  implements BlockCache<TBlock>
{
  private readonly entries = new Map<string, CacheEntry<TBlock>>();
  private readonly ttlMs: number;
  private readonly maxEntries: number;
  private readonly now: () => number;

  constructor(options: InMemoryBlockCacheOptions = {}) {
    this.ttlMs = options.ttlMs ?? DEFAULT_TTL_MS;
    this.maxEntries = options.maxEntries ?? DEFAULT_MAX_ENTRIES;
    this.now = options.now ?? Date.now;

    if (!Number.isFinite(this.ttlMs) || this.ttlMs <= 0) {
      throw new RangeError("Block cache ttlMs must be greater than zero.");
    }
    if (!Number.isInteger(this.maxEntries) || this.maxEntries <= 0) {
      throw new RangeError("Block cache maxEntries must be a positive integer.");
    }
  }

  get(channelId: string, index: number): TBlock | undefined {
    const key = cacheKey(channelId, index);
    const entry = this.entries.get(key);
    if (!entry) return undefined;

    const now = this.now();
    if (entry.expiresAt <= now) {
      this.entries.delete(key);
      return undefined;
    }

    entry.lastAccessedAt = now;
    this.entries.delete(key);
    this.entries.set(key, entry);
    return entry.block;
  }

  set(channelId: string, index: number, block: TBlock): void {
    const key = cacheKey(channelId, index);
    const now = this.now();
    const entry: CacheEntry<TBlock> = {
      channelId,
      index,
      block,
      expiresAt: now + this.ttlMs,
      lastAccessedAt: now,
    };

    this.entries.delete(key);
    this.entries.set(key, entry);
    this.evictOverflow();
  }

  invalidate(channelId: string, index: number): void {
    this.entries.delete(cacheKey(channelId, index));
  }

  invalidateChannel(channelId: string): void {
    for (const [key, entry] of this.entries) {
      if (entry.channelId === channelId) this.entries.delete(key);
    }
  }

  clear(): void {
    this.entries.clear();
  }

  private evictOverflow(): void {
    while (this.entries.size > this.maxEntries) {
      const oldestKey = this.entries.keys().next().value as string | undefined;
      if (oldestKey === undefined) return;
      this.entries.delete(oldestKey);
    }
  }
}
