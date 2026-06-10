pub mod engine;
pub mod error;
pub mod models;
pub mod schema;
pub mod validation;

pub use engine::{
    create_block, create_block_json, generate_candidate, generate_candidate_json,
    render_lore_summary, render_lore_summary_json, schema_bundle_json,
};
pub use error::{NarrativeError, Result};
pub use models::{HybridCandidate, LabConfig, NarrativeBlock, NarrativeLore};
pub use schema::{schema_bundle, MODEL_NAMES};
