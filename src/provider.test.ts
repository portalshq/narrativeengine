import { describe, it, expect } from 'vitest';
import { InMemoryNarrativeProvider } from './provider';
import { MOCK_BLOCKS, MOCK_LORE } from './mocks';

describe('InMemoryNarrativeProvider', () => {
  it('should return block count', async () => {
    const provider = new InMemoryNarrativeProvider(MOCK_BLOCKS, MOCK_LORE);
    const count = await provider.getBlockCount('test');
    expect(count).toBe(MOCK_BLOCKS.length);
  });

  it('should return lore atoms that are active', async () => {
    const customLore = [
      ...MOCK_LORE,
      { id: 'lore-inactive', content: 'Inactive lore', happenedAt: 1000, isActive: false }
    ];
    const provider = new InMemoryNarrativeProvider(MOCK_BLOCKS, customLore);
    const atoms = await provider.getLoreAtoms('test');
    // Only active lore atoms should be returned (MOCK_LORE has 17 active, 3 inactive)
    expect(atoms.length).toBe(17);
    expect(atoms.find(l => l.id === 'lore-inactive')).toBeUndefined();
  });

  it('should return blocks by indices matching block IDs', async () => {
    const provider = new InMemoryNarrativeProvider(MOCK_BLOCKS, MOCK_LORE);
    const indices = [1, 48];
    const blocks = await provider.getBlocksByIndices('test', indices);
    expect(blocks.length).toBe(2);
    expect(blocks.map(b => Number(b.id))).toEqual(indices);
  });

  it('should return hybrid search candidates by content match', async () => {
    const provider = new InMemoryNarrativeProvider(MOCK_BLOCKS, MOCK_LORE);
    const limit = 2;
    const candidates = await provider.getHybridSearchCandidates('test', 'cube', limit);
    expect(candidates.length).toBeGreaterThan(0);
    expect(candidates.length).toBeLessThanOrEqual(limit);
    
    candidates.forEach(c => {
      expect(c.block.content.toLowerCase()).toContain('cube');
      expect(c.scoreVectorDense).toBe(0.8);
      expect(c.scoreKeywordSparse).toBe(0.8);
    });
  });

  it('should return notable events', async () => {
    const provider = new InMemoryNarrativeProvider(MOCK_BLOCKS, MOCK_LORE);
    const notableEvents = await provider.getNotableEvents('test');
    const expectedCount = MOCK_BLOCKS.filter(b => b.isNotable).length;
    
    expect(notableEvents.length).toBe(expectedCount);
    notableEvents.forEach(b => {
      expect(b.isNotable).toBe(true);
    });
  });

  it('should append blocks and identify itself', async () => {
    const provider = new InMemoryNarrativeProvider([], []);
    const block = {
      id: 99,
      index: 99,
      content: 'A newly remembered event',
      happenedAt: 1_700_000_000_000,
      isNotable: false,
    };

    await provider.addBlock('test', block);

    expect(provider.getProviderType()).toBe('in-memory');
    await expect(provider.getBlockCount('test')).resolves.toBe(1);
    await expect(provider.getBlocksByIndices('test', [99])).resolves.toEqual([block]);
  });
});
