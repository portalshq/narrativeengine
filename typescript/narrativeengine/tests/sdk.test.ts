import { describe, expect, it } from "vitest";
import { createBlock, generateCandidate, renderLoreSummary } from "../src/index.js";

describe("NarrativeEngine TypeScript SDK", () => {
  it("creates blocks and candidates through native Rust", () => {
    const block = createBlock("intro", "A signal appears in the archive.");
    const lore = { id: "lore-1", title: "Archive Signal", blocks: [block] };
    const config = { temperature: 0.7, max_candidates: 4, seed: 7 };
    const candidate = generateCandidate(lore, config);

    expect(candidate.id).toBe("candidate-lore-1-7");
    expect(candidate.block.id).toBe("lore-1:hybrid");
    expect(renderLoreSummary(lore)).toBe("Archive Signal contains 1 block(s) and 6 word(s).");
  });
});

