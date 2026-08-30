use napi::Error;
use napi_derive::napi;
use narrativeengine::{
    engine::NarrativeEngine,
    narrative::v1::{BaseNarrativeBlock, BaseNarrativeLore},
    provider::InMemoryNarrativeProvider,
};

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

        let context = rt.block_on(async {
            self.engine.generate_context(&channel_id, &query).await
        });

        Ok(context)
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
            self.engine.generate_block(&channel_id, &input_query, parameters).await
        });

        match result {
            Ok(envelope) => {
                let json = serde_json::to_string(&envelope)
                    .map_err(|e| Error::from_reason(format!("Failed to serialize result: {}", e)))?;
                Ok(json)
            }
            Err(error) => Err(Error::from_reason(format!("Generation failed: {}", error.message))),
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
                let json = serde_json::to_string(&result)
                    .map_err(|e| Error::from_reason(format!("Failed to serialize result: {}", e)))?;
                Ok(json)
            }
            Err(error) => Err(Error::from_reason(format!("Batch generation failed: {}", error.message))),
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
                let json = serde_json::to_string(&result)
                    .map_err(|e| Error::from_reason(format!("Failed to serialize result: {}", e)))?;
                Ok(json)
            }
            Err(error) => Err(Error::from_reason(format!("Parallel generation failed: {}", error.message))),
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
