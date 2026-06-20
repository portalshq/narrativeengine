//! NAP Structured Merge Engine v2.
//!
//! Schema-driven three-way merge for narrative manifests.
//!
//! # Architecture
//!
//! ```text
//! SDL → Normalize → Path Union → Strategy Dispatch → Merge → Validate → Persist
//! ```
//!
//! # Protocol Invariants (hardcoded, not in SDL)
//!
//! 1. Normalize before merge
//! 2. Missing ≠ null
//! 3. Identity immutable (mutation → conflict)
//! 4. Merge over path union
//! 5. Validate after merge (caller responsibility)
//! 6. Deterministic execution
//! 7. Atomic persistence (caller responsibility)
//!
//! # SDL defines (configuration, not invariants)
//!
//! - Property types
//! - Merge strategies per property
//! - Identity rules for arrays

pub mod atomic_write;
pub mod conflict;
pub mod diff;
pub mod merge_engine;
pub mod normalization;
pub mod path;
pub mod sdl;
pub mod strategies;

// Re-exports for top-level usage
pub use conflict::{Conflict, ConflictType, MergeResult};
pub use diff::{Change, ChangeOp, DiffResult, diff, diff_normalized};
pub use merge_engine::MergeEngine;
pub use normalization::normalize;
pub use path::{CanonicalPath, build_path_map, path_union, resolve_path};
pub use sdl::{
    IdentityRule, MergeStrategyDef, MergeStrategyType, PropertyDef, PropertyType, SdlDocument,
    SdlError,
};
