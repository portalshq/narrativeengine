export function calculateHarmonicConstant(terms: number): number {
  if (terms <= 0) return 0;
  let sum = 0;
  for (let index = 1; index <= terms; index += 1) sum += 1 / index;
  return sum;
}

export function generateReciprocalSequence(target: number, divisions: number): number[] {
  if (target <= 1 || divisions <= 0) return [1];

  const scale = (target - 1) / calculateHarmonicConstant(divisions);
  const sequence = [1];
  for (let index = 1; index <= divisions; index += 1) {
    const previous = sequence[index - 1];
    if (previous === undefined) break;
    sequence.push(Number((previous + scale / index).toFixed(2)));
  }
  return sequence;
}

export function sequenceToBlockIndices(sequence: readonly number[]): number[] {
  return [...new Set(sequence.map((value) => Math.max(1, Math.round(value))))].sort(
    (left, right) => left - right,
  );
}

export function generateHistoricalIndices(totalBlocks: number, divisions: number): number[] {
  return sequenceToBlockIndices(generateReciprocalSequence(totalBlocks, divisions));
}
