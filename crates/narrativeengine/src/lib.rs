//! `narrative_engine` — Rust port of the TypeScript narrative RAG engine.
//!
//! # Modules
//! - [`types`]    — Core data structures (`BaseNarrativeBlock`, `BaseNarrativeLore`)
//! - [`sequence`] — Reciprocal-sequence RAG utilities
//! - [`provider`] — Provider trait + `InMemoryNarrativeProvider`
//! - [`engine`]   — `NarrativeEngine` (the RAG pipeline)
//! - [`lab`]      — Global engine registry
//! - [`trace`]    — Observability / trace logging
//! - [`mocks`]    — 100 story blocks + 20 lore entries for testing
//! - [`utils`]    — Score normalisation and provider validation

pub mod engine;
pub mod mocks;
pub mod provider;
pub mod sequence;
pub mod trace;
pub mod types;
pub mod utils;

// Generated proto code
pub mod narrative {
    pub mod v1 {
        tonic::include_proto!("narrative.v1");
    }
}

// Convenience re-exports
pub use engine::{LabConfig, NarrativeEngine, ResolvedLabConfig};
pub use narrative::v1::{BaseNarrativeBlock, BaseNarrativeLore};
pub use provider::{HybridCandidate, InMemoryNarrativeProvider, NarrativeProvider};
pub use types::BlockId;
pub use utils::{normalize_score, validate_provider_shape};
