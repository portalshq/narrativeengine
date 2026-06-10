use crate::error::Result;
use crate::models::{HybridCandidate, LabConfig, NarrativeBlock, NarrativeLore};
use crate::schema::schema_bundle;
use crate::validation::{validate_block, validate_config, validate_lore};
use serde::Serialize;

pub fn create_block(id: impl Into<String>, content: impl Into<String>) -> Result<NarrativeBlock> {
    let block = NarrativeBlock {
        id: id.into(),
        content: content.into(),
    };
    validate_block(&block)?;
    Ok(block)
}

pub fn generate_candidate(lore: &NarrativeLore, config: &LabConfig) -> Result<HybridCandidate> {
    validate_lore(lore)?;
    validate_config(config)?;

    let content = lore
        .blocks
        .iter()
        .map(|block| block.content.trim())
        .collect::<Vec<_>>()
        .join(" ");

    let block = NarrativeBlock {
        id: format!("{}:hybrid", lore.id),
        content,
    };

    let candidate = HybridCandidate {
        id: format!("candidate-{}-{}", stable_identifier(&lore.id), config.seed),
        block,
        score: deterministic_score(lore, config),
        rationale: format!(
            "Hybridized {} block(s) from '{}' with seed {}.",
            lore.blocks.len(),
            lore.title,
            config.seed
        ),
    };

    Ok(candidate)
}

pub fn render_lore_summary(lore: &NarrativeLore) -> Result<String> {
    validate_lore(lore)?;
    let words = lore
        .blocks
        .iter()
        .map(|block| block.content.split_whitespace().count())
        .sum::<usize>();
    Ok(format!(
        "{} contains {} block(s) and {} word(s).",
        lore.title,
        lore.blocks.len(),
        words
    ))
}

pub fn create_block_json(id: impl Into<String>, content: impl Into<String>) -> Result<String> {
    to_canonical_json(&create_block(id, content)?)
}

pub fn generate_candidate_json(lore_json: &str, config_json: &str) -> Result<String> {
    let lore: NarrativeLore = serde_json::from_str(lore_json)?;
    let config: LabConfig = serde_json::from_str(config_json)?;
    to_canonical_json(&generate_candidate(&lore, &config)?)
}

pub fn render_lore_summary_json(lore_json: &str) -> Result<String> {
    let lore: NarrativeLore = serde_json::from_str(lore_json)?;
    render_lore_summary(&lore)
}

pub fn schema_bundle_json() -> Result<String> {
    to_canonical_json(&schema_bundle())
}

fn to_canonical_json<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

fn stable_identifier(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn deterministic_score(lore: &NarrativeLore, config: &LabConfig) -> f64 {
    let mut hash = config.seed ^ 0xcbf2_9ce4_8422_2325;

    for byte in lore.id.bytes().chain(lore.title.bytes()).chain(
        lore.blocks
            .iter()
            .flat_map(|block| block.id.bytes().chain(block.content.bytes())),
    ) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }

    let base = (hash % 10_000) as f64 / 10_000.0;
    let temperature_factor = 1.0 + (config.temperature / 10.0);
    ((base * temperature_factor).min(1.0) * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LabConfig, NarrativeLore};
    use proptest::prelude::*;
    use rstest::rstest;

    #[rstest]
    fn create_block_rejects_blank_id() {
        let err = create_block(" ", "content").expect_err("blank id must be invalid");
        assert!(err.to_string().contains("NarrativeBlock.id"));
    }

    #[rstest]
    fn generate_candidate_is_deterministic() {
        let block = create_block("b1", "The engine remembers.").unwrap();
        let lore = NarrativeLore {
            id: "lore".to_string(),
            title: "Memory".to_string(),
            blocks: vec![block],
        };
        let config = LabConfig::default();

        assert_eq!(
            generate_candidate(&lore, &config).unwrap(),
            generate_candidate(&lore, &config).unwrap()
        );
    }

    proptest! {
        #[test]
        fn valid_blocks_round_trip_json(id in "[a-zA-Z0-9][a-zA-Z0-9_-]{0,24}", content in "\\PC{1,120}") {
            let json = create_block_json(id, content).unwrap();
            let block: NarrativeBlock = serde_json::from_str(&json).unwrap();
            validate_block(&block).unwrap();
        }
    }
}
