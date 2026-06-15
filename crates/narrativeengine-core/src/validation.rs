use crate::error::{NarrativeError, Result};
use crate::models::{LabConfig, NarrativeBlock, NarrativeLore};

pub fn validate_block(block: &NarrativeBlock) -> Result<()> {
    require_non_empty("NarrativeBlock.id", &block.id)?;
    require_non_empty("NarrativeBlock.content", &block.content)?;
    Ok(())
}

pub fn validate_lore(lore: &NarrativeLore) -> Result<()> {
    require_non_empty("NarrativeLore.id", &lore.id)?;
    require_non_empty("NarrativeLore.title", &lore.title)?;

    if lore.blocks.is_empty() {
        return Err(NarrativeError::Validation(
            "NarrativeLore.blocks must contain at least one block".to_string(),
        ));
    }

    for block in &lore.blocks {
        validate_block(block)?;
    }

    Ok(())
}

pub fn validate_config(config: &LabConfig) -> Result<()> {
    if !config.temperature.is_finite() {
        return Err(NarrativeError::Validation(
            "LabConfig.temperature must be finite".to_string(),
        ));
    }

    if !(0.0..=2.0).contains(&config.temperature) {
        return Err(NarrativeError::Validation(
            "LabConfig.temperature must be between 0.0 and 2.0".to_string(),
        ));
    }

    if config.max_candidates == 0 || config.max_candidates > 1000 {
        return Err(NarrativeError::Validation(
            "LabConfig.max_candidates must be between 1 and 1000".to_string(),
        ));
    }

    Ok(())
}

fn require_non_empty(field: &str, value: &str) -> Result<()> {
    // Use ASCII whitespace check (not trim()) so behavior is stable across Rust editions.
    // In edition 2024, trim() also strips non-ASCII whitespace like U+00A0 (NBSP).
    if value.chars().all(|c| c.is_ascii_whitespace()) {
        return Err(NarrativeError::Validation(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}
