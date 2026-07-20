//! Utility functions.
//!
//! Mirrors `utils.ts`: score normalization and provider shape validation.

use crate::provider::NarrativeProvider;
use crate::types::{BaseNarrativeBlock, BaseNarrativeLore};

/// Normalizes a value from an arbitrary range to `[0.0, 1.0]`.
///
/// Returns `0` when `min == max` (degenerate range).
pub fn normalize_score(value: f64, min: f64, max: f64) -> f64 {
    if (max - min).abs() < f64::EPSILON {
        return 0.0;
    }
    let normalized = (value - min) / (max - min);
    normalized.clamp(0.0, 1.0)
}

/// Runtime validation that a provider implements the required interface.
///
/// In Rust this is enforced at compile time via the `NarrativeProvider` trait,
/// but this function exists to mirror the JS runtime check and is useful for
/// boxed dynamic dispatch scenarios.
pub fn validate_provider_shape<P>(provider: &P) -> bool
where
    P: NarrativeProvider<BaseNarrativeBlock, BaseNarrativeLore>,
{
    // Trait bound guarantees all methods exist; validation always succeeds.
    let _ = provider;
    true
}

/// Dynamic (type-erased) validation — checks a `dyn Any` value has provider methods.
///
/// Provided as a weaker runtime analogue to the JS `validateProviderShape`.
/// Returns `false` and logs an error if `provider` is `None`.
pub fn validate_provider_shape_dyn(provider: Option<&dyn std::any::Any>) -> bool {
    if provider.is_none() {
        eprintln!(
            "[NarrativeEngine] Invalid Provider: Missing methods [getLoreAtoms, getNotableEvents, getBlocksByIndices, getHybridSearchCandidates, getBlockCount]"
        );
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_midpoint() {
        assert!((normalize_score(5.0, 0.0, 10.0) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn normalize_zero() {
        assert!((normalize_score(0.0, 0.0, 10.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn normalize_max() {
        assert!((normalize_score(10.0, 0.0, 10.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn normalize_shifted_range() {
        assert!((normalize_score(75.0, 50.0, 100.0) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn normalize_degenerate_range() {
        assert_eq!(normalize_score(5.0, 5.0, 5.0), 0.0);
    }

    #[test]
    fn normalize_clamp_below() {
        assert_eq!(normalize_score(-5.0, 0.0, 10.0), 0.0);
    }

    #[test]
    fn normalize_clamp_above() {
        assert_eq!(normalize_score(15.0, 0.0, 10.0), 1.0);
    }
}
