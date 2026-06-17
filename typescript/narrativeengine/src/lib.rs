use napi::Error;
use napi_derive::napi;

fn map_error(error: narrativeengine::NarrativeError) -> Error {
    Error::from_reason(error.to_string())
}

#[napi(js_name = "createBlockJson")]
pub fn create_block_json(id: String, content: String) -> napi::Result<String> {
    narrativeengine::create_block_json(id, content).map_err(map_error)
}

#[napi(js_name = "generateCandidateJson")]
pub fn generate_candidate_json(lore_json: String, config_json: String) -> napi::Result<String> {
    narrativeengine::generate_candidate_json(&lore_json, &config_json).map_err(map_error)
}

#[napi(js_name = "renderLoreSummaryJson")]
pub fn render_lore_summary_json(lore_json: String) -> napi::Result<String> {
    narrativeengine::render_lore_summary_json(&lore_json).map_err(map_error)
}

#[napi(js_name = "schemaBundleJson")]
pub fn schema_bundle_json() -> napi::Result<String> {
    narrativeengine::schema_bundle_json().map_err(map_error)
}

#[napi]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
