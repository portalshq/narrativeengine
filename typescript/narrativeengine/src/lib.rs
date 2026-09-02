use napi::Error;
use napi_derive::napi;
use narrativeengine::{
    BlockId,
    engine::{NarrativeEngine, PreparedContext},
    narrative::v1::{BaseNarrativeBlock, BaseNarrativeLore},
    provider::{HybridCandidate, InMemoryNarrativeProvider},
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
enum JsNarrativeId {
    Number(i64),
    String(String),
}

impl From<JsNarrativeId> for narrativeengine::narrative::v1::BlockId {
    fn from(value: JsNarrativeId) -> Self {
        match value {
            JsNarrativeId::Number(value) => BlockId::Num(value).into(),
            JsNarrativeId::String(value) => BlockId::Str(value).into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsNarrativeBlockInput {
    id: JsNarrativeId,
    index: u64,
    content: String,
    happened_at: i64,
    is_notable: Option<bool>,
}

impl From<JsNarrativeBlockInput> for BaseNarrativeBlock {
    fn from(value: JsNarrativeBlockInput) -> Self {
        Self {
            id: Some(value.id.into()),
            index: value.index,
            content: value.content,
            happened_at: value.happened_at,
            is_notable: value.is_notable,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsNarrativeLoreInput {
    id: JsNarrativeId,
    content: String,
    happened_at: i64,
    is_active: Option<bool>,
}

impl From<JsNarrativeLoreInput> for BaseNarrativeLore {
    fn from(value: JsNarrativeLoreInput) -> Self {
        Self {
            id: Some(value.id.into()),
            content: value.content,
            happened_at: value.happened_at,
            is_active: value.is_active,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsHybridCandidateInput {
    block: JsNarrativeBlockInput,
    score_vector_dense: f64,
    score_keyword_sparse: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JsPreparedContextInput {
    channel_id: String,
    input_query: String,
    total_block_count: usize,
    lore_atoms: Vec<JsNarrativeLoreInput>,
    candidates_hybrid: Vec<JsHybridCandidateInput>,
    blocks_historical: Vec<JsNarrativeBlockInput>,
    provider_type: String,
    block_sequence_intervals: Vec<usize>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Basic utility functions (legacy compatibility - simplified versions)
// ─────────────────────────────────────────────────────────────────────────────

#[napi(js_name = "createBlockJson")]
pub fn create_block_json(id: String, content: String) -> napi::Result<String> {
    // Return a simple JSON representation for compatibility
    let block_json = format!(
        r#"{{"id":"{}","index":0,"content":"{}","happened_at":0,"is_notable":false}}"#,
        id, content
    );
    Ok(block_json)
}

#[napi(js_name = "generateCandidateJson")]
pub fn generate_candidate_json(_lore_json: String, _config_json: String) -> napi::Result<String> {
    // Return a simple placeholder for compatibility
    Ok(r#"{"score":0.5,"block":{"content":"placeholder"},"lore_atoms":[]}"#.to_string())
}

#[napi(js_name = "renderLoreSummaryJson")]
pub fn render_lore_summary_json(_lore_json: String) -> napi::Result<String> {
    // Return a simple placeholder for compatibility
    Ok("Lore summary placeholder".to_string())
}

#[napi(js_name = "schemaBundleJson")]
pub fn schema_bundle_json() -> napi::Result<String> {
    Ok(r#"{"version":"0.6.0","features":["basic_generation","context_generation"]}"#.to_string())
}

#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ─────────────────────────────────────────────────────────────────────────────
// NarrativeEngine class bindings
// ─────────────────────────────────────────────────────────────────────────────

#[napi]
pub struct JsNarrativeEngine {
    engine: NarrativeEngine<BaseNarrativeBlock, BaseNarrativeLore>,
}

#[napi]
impl JsNarrativeEngine {
    #[napi(constructor)]
    pub fn new() -> napi::Result<Self> {
        let provider = InMemoryNarrativeProvider::new(vec![], vec![]);
        let engine = NarrativeEngine::new(std::sync::Arc::new(provider));

        Ok(JsNarrativeEngine { engine })
    }

    #[napi]
    pub fn generate_context(&self, channel_id: String, query: String) -> napi::Result<String> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::from_reason(format!("Failed to create runtime: {}", e)))?;

        let context =
            rt.block_on(async { self.engine.generate_context(&channel_id, &query).await });

        Ok(context)
    }

    #[napi]
    pub fn plan_context(&self, total_block_count: u32) -> napi::Result<String> {
        serde_json::to_string(&self.engine.plan_context(total_block_count as usize)).map_err(
            |error| Error::from_reason(format!("Failed to serialize context plan: {error}")),
        )
    }

    #[napi]
    pub fn generate_context_from_data(&self, input_json: String) -> napi::Result<String> {
        let input: JsPreparedContextInput = serde_json::from_str(&input_json).map_err(|error| {
            Error::from_reason(format!("Failed to parse prepared context: {error}"))
        })?;
        let prepared = PreparedContext {
            channel_id: input.channel_id,
            input_query: input.input_query,
            total_block_count: input.total_block_count,
            lore_atoms: input.lore_atoms.into_iter().map(Into::into).collect(),
            candidates_hybrid: input
                .candidates_hybrid
                .into_iter()
                .map(|candidate| HybridCandidate {
                    block: candidate.block.into(),
                    score_vector_dense: candidate.score_vector_dense,
                    score_keyword_sparse: candidate.score_keyword_sparse,
                })
                .collect(),
            blocks_historical: input
                .blocks_historical
                .into_iter()
                .map(Into::into)
                .collect(),
            provider_type: input.provider_type,
            block_sequence_intervals: input.block_sequence_intervals,
        };

        Ok(self.engine.generate_context_from_data(prepared))
    }

    #[napi]
    pub fn generate_block(
        &self,
        channel_id: String,
        input_query: String,
        parameters_json: String,
    ) -> napi::Result<String> {
        use narrativeengine::narrative::v1::GenerationParameters;

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::from_reason(format!("Failed to create runtime: {}", e)))?;

        let parameters: GenerationParameters = serde_json::from_str(&parameters_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse parameters: {}", e)))?;

        let result = rt.block_on(async {
            self.engine
                .generate_block(&channel_id, &input_query, parameters)
                .await
        });

        match result {
            Ok(envelope) => {
                let json = serde_json::to_string(&envelope).map_err(|e| {
                    Error::from_reason(format!("Failed to serialize result: {}", e))
                })?;
                Ok(json)
            }
            Err(error) => Err(Error::from_reason(format!(
                "Generation failed: {}",
                error.message
            ))),
        }
    }

    #[napi]
    pub fn generate_blocks_sequential(
        &self,
        channel_id: String,
        previous_context: String,
        options_json: String,
    ) -> napi::Result<String> {
        use narrativeengine::narrative::v1::BatchGenerationOptions;

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::from_reason(format!("Failed to create runtime: {}", e)))?;

        let options: BatchGenerationOptions = serde_json::from_str(&options_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse options: {}", e)))?;

        let result = rt.block_on(async {
            self.engine
                .generate_blocks_sequential(&channel_id, &previous_context, options)
                .await
        });

        match result {
            Ok(result) => {
                let json = serde_json::to_string(&result).map_err(|e| {
                    Error::from_reason(format!("Failed to serialize result: {}", e))
                })?;
                Ok(json)
            }
            Err(error) => Err(Error::from_reason(format!(
                "Batch generation failed: {}",
                error.message
            ))),
        }
    }

    #[napi]
    pub fn generate_blocks_parallel(
        &self,
        channel_id: String,
        branch_contexts: Vec<String>,
        options_json: String,
    ) -> napi::Result<String> {
        use narrativeengine::narrative::v1::BatchGenerationOptions;

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::from_reason(format!("Failed to create runtime: {}", e)))?;

        let options: BatchGenerationOptions = serde_json::from_str(&options_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse options: {}", e)))?;

        let result = rt.block_on(async {
            self.engine
                .generate_blocks_parallel(&channel_id, &branch_contexts, options)
                .await
        });

        match result {
            Ok(result) => {
                let json = serde_json::to_string(&result).map_err(|e| {
                    Error::from_reason(format!("Failed to serialize result: {}", e))
                })?;
                Ok(json)
            }
            Err(error) => Err(Error::from_reason(format!(
                "Parallel generation failed: {}",
                error.message
            ))),
        }
    }

    #[napi]
    pub fn set_lab_config(&mut self, config_json: String) -> napi::Result<()> {
        use narrativeengine::engine::LabConfig;

        let config: LabConfig = serde_json::from_str(&config_json)
            .map_err(|e| Error::from_reason(format!("Failed to parse config: {}", e)))?;

        self.engine.set_lab_config(config);
        Ok(())
    }

    #[napi]
    pub fn get_lab_config(&self) -> napi::Result<String> {
        let config = self.engine.get_lab_config();
        let json = serde_json::to_string(&config)
            .map_err(|e| Error::from_reason(format!("Failed to serialize config: {}", e)))?;
        Ok(json)
    }

    #[napi]
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
