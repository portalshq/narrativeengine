//! RAG sequence utilities.
//!
//! Mirrors `sequence.ts`: harmonic constant, reciprocal sequence generation,
//! block-index conversion, and cluster-window expansion.

/// Number of historical blocks to retrieve for RAG context.
pub const RAG_DIVISIONS: usize = 5;

/// Minimum block count before RAG skeleton kicks in.
pub const RAG_MIN_BLOCKS: usize = 3;

/// Window size for clustered block fetching (blocks before/after each milestone).
pub const RAG_CLUSTER_WINDOW: usize = 1;

/// Calculates the harmonic constant H(n) = Σ 1/i for i = 1..=n.
///
/// Used as the normalizing denominator for the reciprocal sequence.
pub fn calculate_harmonic_constant(n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    (1..=n).map(|i| 1.0 / i as f64).sum()
}

/// Generates a sequence from 1 to `target_n` with reciprocal spacing.
///
/// Jumps between consecutive values decrease as 1/i, producing indices that
/// are sparse at the start (distant past) and dense at the end (recent).
///
/// Returns `(divisions + 1)` values from 1 to `target_n`.
pub fn generate_reciprocal_sequence(target_n: usize, divisions: usize) -> Vec<f64> {
    if target_n <= 1 || divisions == 0 {
        return vec![1.0];
    }
    let k = calculate_harmonic_constant(divisions);
    let scale = (target_n as f64 - 1.0) / k;
    let mut sequence = vec![1.0_f64];
    for i in 1..=divisions {
        let last = sequence[i - 1];
        let jump = scale / i as f64;
        // Round to 2 decimal places to mirror TypeScript behaviour
        let next = ((last + jump) * 100.0).round() / 100.0;
        sequence.push(next);
    }
    sequence
}

/// Converts a raw reciprocal sequence into unique, rounded, 1-indexed block positions.
///
/// Deduplicates and sorts ascending.
pub fn sequence_to_block_indices(sequence: &[f64]) -> Vec<usize> {
    let mut rounded: Vec<usize> = sequence
        .iter()
        .map(|&v| (v.round() as usize).max(1))
        .collect();
    rounded.sort_unstable();
    rounded.dedup();
    rounded
}

/// Expands block indices to include surrounding blocks for micro-context.
///
/// Provides narrative connective tissue around each selected milestone.
pub fn expand_to_clustered_indices(
    indices: &[usize],
    total_blocks: usize,
    window_size: usize,
) -> Vec<usize> {
    let mut windowed: std::collections::BTreeSet<usize> = std::collections::BTreeSet::new();
    for &idx in indices {
        let start = idx.saturating_sub(window_size);
        let end = (idx + window_size).min(total_blocks);
        for target in start..=end {
            if target >= 1 {
                windowed.insert(target);
            }
        }
    }
    windowed.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harmonic_constant_n1() {
        assert_eq!(calculate_harmonic_constant(1), 1.0);
    }

    #[test]
    fn harmonic_constant_n2() {
        // 1 + 1/2 = 1.5
        assert!((calculate_harmonic_constant(2) - 1.5).abs() < 1e-10);
    }

    #[test]
    fn harmonic_constant_zero() {
        assert_eq!(calculate_harmonic_constant(0), 0.0);
    }

    #[test]
    fn reciprocal_sequence_logarithmic_spread() {
        let target = 1000;
        let divisions = 5;
        let seq = generate_reciprocal_sequence(target, divisions);

        assert_eq!(seq[0], 1.0);
        assert!((seq[seq.len() - 1] - target as f64).abs() < 1.0);

        // Gaps should be larger at the start than the end
        let gap_start = seq[1] - seq[0];
        let gap_end = seq[5] - seq[4];
        assert!(gap_start > gap_end, "gap_start={gap_start}, gap_end={gap_end}");
    }

    #[test]
    fn sequence_to_block_indices_dedup_and_round() {
        // 1.2 rounds to 1 (deduped with existing 1), 5.8 rounds to 6, 10 stays
        let raw = vec![1.0, 1.2, 5.8, 10.0];
        let indices = sequence_to_block_indices(&raw);
        assert_eq!(indices, vec![1, 6, 10]);
    }

    #[test]
    fn sequence_to_block_indices_sorted() {
        let raw = vec![10.0, 3.0, 7.0, 3.4];
        let indices = sequence_to_block_indices(&raw);
        // 10->10, 3->3, 7->7, 3.4->3 (deduped)
        assert_eq!(indices, vec![3, 7, 10]);
    }

    #[test]
    fn expand_clustered_indices_bounded() {
        let indices = vec![1, 5, 10];
        let expanded = expand_to_clustered_indices(&indices, 10, 1);
        // 1±1=[1,2], 5±1=[4,5,6], 10±1=[9,10]
        assert!(expanded.contains(&1));
        assert!(expanded.contains(&2));
        assert!(expanded.contains(&4));
        assert!(expanded.contains(&6));
        assert!(expanded.contains(&9));
        // Never exceeds total_blocks
        assert!(!expanded.contains(&11));
    }

    #[test]
    fn generate_reciprocal_sequence_single_block() {
        let seq = generate_reciprocal_sequence(1, 5);
        assert_eq!(seq, vec![1.0]);
    }

    #[test]
    fn generate_reciprocal_sequence_zero_divisions() {
        let seq = generate_reciprocal_sequence(100, 0);
        assert_eq!(seq, vec![1.0]);
    }
}