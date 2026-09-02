import { describe, expect, it } from "vitest";
import {
  calculateHarmonicConstant,
  generateHistoricalIndices,
  generateReciprocalSequence,
  sequenceToBlockIndices,
} from "../src/index.js";

describe("reciprocal historical sampling", () => {
  it("produces deterministic, increasingly dense positions ending at the latest block", () => {
    expect(calculateHarmonicConstant(3)).toBeCloseTo(1.8333333333);
    expect(generateReciprocalSequence(20, 5)).toEqual([1, 9.32, 13.48, 16.25, 18.33, 19.99]);
    expect(generateHistoricalIndices(20, 5)).toEqual([1, 9, 13, 16, 18, 20]);
  });

  it("deduplicates rounded positions and handles degenerate inputs", () => {
    expect(sequenceToBlockIndices([1, 1.1, 1.4, 2.2])).toEqual([1, 2]);
    expect(generateReciprocalSequence(0, 5)).toEqual([1]);
    expect(calculateHarmonicConstant(0)).toBe(0);
  });
});
