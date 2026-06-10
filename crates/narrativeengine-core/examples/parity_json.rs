use narrativeengine_core::{
    LabConfig, NarrativeBlock, NarrativeLore, create_block_json, generate_candidate_json,
    render_lore_summary_json,
};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let block_json = create_block_json("intro", "A signal appears in the archive.")?;
    let block: NarrativeBlock = serde_json::from_str(&block_json)?;
    let lore = NarrativeLore {
        id: "lore-1".to_string(),
        title: "Archive Signal".to_string(),
        blocks: vec![block],
    };
    let config = LabConfig::default();
    let lore_json = serde_json::to_string(&lore)?;
    let config_json = serde_json::to_string(&config)?;
    let candidate_json = generate_candidate_json(&lore_json, &config_json)?;
    let summary = render_lore_summary_json(&lore_json)?;

    let output = json!({
        "block": serde_json::from_str::<serde_json::Value>(&block_json)?,
        "candidate": serde_json::from_str::<serde_json::Value>(&candidate_json)?,
        "summary": summary,
    });
    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}
