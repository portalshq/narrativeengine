//! Per-strategy merge logic.
//!
//! Each strategy is a pure function that takes the three values
//! (base, current, proposed) and returns a merge result for that
//! specific path.

pub mod atomic;
pub mod deep_merge;
pub mod edge_list;
pub mod ordered_unique;
pub mod replace;
pub mod set_union;
