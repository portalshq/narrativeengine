use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct NarrativeBlock {
    pub id: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct NarrativeLore {
    pub id: String,
    pub title: String,
    pub blocks: Vec<NarrativeBlock>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct LabConfig {
    pub temperature: f64,
    pub max_candidates: u32,
    pub seed: u64,
}

impl Default for LabConfig {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_candidates: 4,
            seed: 7,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct HybridCandidate {
    pub id: String,
    pub block: NarrativeBlock,
    pub score: f64,
    pub rationale: String,
}
