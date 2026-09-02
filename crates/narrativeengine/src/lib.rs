//! `narrative_engine` — Rust port of the TypeScript narrative RAG engine.
//!
//! # Modules
//! - [`types`]    — Core data structures (`BaseNarrativeBlock`, `BaseNarrativeLore`)
//! - [`sequence`] — Reciprocal-sequence RAG utilities
//! - [`provider`] — Provider trait + `InMemoryNarrativeProvider`
//! - [`engine`]   — `NarrativeEngine` (the RAG pipeline)
//! - [`trace`]    — Observability / trace logging
//! - [`mocks`]    — 100 story blocks + 20 lore entries for testing
//! - [`utils`]    — Score normalisation and provider validation
//!
//! # Enhanced API
//!
//! The crate provides an enhanced API for structured block generation with optional entity enrichment:
//!
//! - `generate_block` — Generate a single block with optional entity extraction and representations
//! - `generate_blocks_sequential` — Generate blocks sequentially for narrative continuity
//! - `generate_blocks_parallel` — Generate blocks in parallel for independent branches
//!
//! These methods are available on `NarrativeEngine<BaseNarrativeBlock, BaseNarrativeLore>` and return structured envelopes containing:
//! - Generated block content
//! - Historical context blocks
//! - Optional entity context (when enabled)
//! - Optional representations (when entities are found)
//! - Generation metadata
//!
//! # Entity Enrichment Integration
//!
//! Entity extraction and representation retrieval are provided as extension points for application integration:
//! - Applications can implement nap-sdk integration using the provided callbacks
//! - Entity extraction is optional and configurable via `GenerationParameters`
//! - Graceful degradation: missing or failed enrichment returns empty collections
//!
//! # Batch Generation
//!
//! Sequential batch generation mirrors the application's `batch-generate.ts` pattern:
//! - Each block's output becomes the next block's input for narrative continuity
//! - Supports dry-run mode and explicit persistence controls
//! - Continues after recoverable failures, aborts on systemic errors
//!
//! Parallel batch generation supports independent story branches:
//! - Branches execute concurrently for performance
//! - One branch failure does not discard successful branches
//! - Supports bounded concurrency and cancellation

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
pub use engine::{ContextPlan, LabConfig, NarrativeEngine, PreparedContext, ResolvedLabConfig};
pub use narrative::v1::{
    BaseNarrativeBlock, BaseNarrativeLore, BatchGenerationOptions, BatchGenerationResult,
    ContextData, Entity, EnvelopeMetadata, GenerationParameters, Representation, ReturnEnvelope,
};
pub use provider::{HybridCandidate, InMemoryNarrativeProvider, NarrativeProvider};
pub use types::BlockId;
pub use utils::{normalize_score, validate_provider_shape};
