use crate::models::{HybridCandidate, LabConfig, NarrativeBlock, NarrativeLore};
use schemars::schema_for;
use serde_json::{Value, json};

pub const MODEL_NAMES: [&str; 4] = [
    "NarrativeBlock",
    "NarrativeLore",
    "LabConfig",
    "HybridCandidate",
];

pub fn schema_bundle() -> Value {
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "package": "narrativeengine",
        "version": env!("CARGO_PKG_VERSION"),
        "model_order": MODEL_NAMES,
        "field_order": {
            "NarrativeBlock": ["id", "content"],
            "NarrativeLore": ["id", "title", "blocks"],
            "LabConfig": ["temperature", "max_candidates", "seed"],
            "HybridCandidate": ["id", "block", "score", "rationale"],
        },
        "models": {
            "NarrativeBlock": schema_for!(NarrativeBlock),
            "NarrativeLore": schema_for!(NarrativeLore),
            "LabConfig": schema_for!(LabConfig),
            "HybridCandidate": schema_for!(HybridCandidate),
        }
    })
}
