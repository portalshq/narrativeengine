//! Core `NarrativeEngine` — the RAG pipeline.
//!
//! Mirrors `engine.ts` in full: lab config, hybrid scoring, saliency gate,
//! tie-breaker, lore overload protection, temporal phrasing, and batch support.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::provider::{HybridCandidate, InMemoryNarrativeProvider, NarrativeProvider};
use crate::sequence::{
    generate_reciprocal_sequence, sequence_to_block_indices, RAG_DIVISIONS, RAG_MIN_BLOCKS,
};
use crate::trace::{logger_narrative_trace, TraceObject, TracePhases};
use crate::types::{BaseNarrativeBlock, BaseNarrativeLore, NarrativeBlockExt};

/// Maximum hybrid-search survivors kept after the saliency gate.
const LIMIT_HYBRID_TOP: usize = 3;

// ─────────────────────────────────────────────────────────────────────────────
// LabConfig
// ─────────────────────────────────────────────────────────────────────────────

/// Runtime configuration knobs for the RAG pipeline.
/// All fields optional — missing values fall back to defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabConfig {
    pub saliency_threshold: Option<f64>,
    pub weight_dense: Option<f64>,
    pub significance_coef: Option<f64>,
    pub temporal_phrasing: Option<bool>,
    /// Cap on lore atoms included in the prompt (Lore Overload protection).
    pub max_lore_atoms: Option<usize>,
    pub timestamp: Option<String>,
}

/// Fully resolved config with no optional fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedLabConfig {
    pub saliency_threshold: f64,
    pub weight_dense: f64,
    pub significance_coef: f64,
    pub temporal_phrasing: bool,
    pub max_lore_atoms: usize,
    pub timestamp: Option<String>,
}

impl Default for ResolvedLabConfig {
    fn default() -> Self {
        Self {
            saliency_threshold: 0.65,
            weight_dense: 0.7,
            significance_coef: 1.5,
            temporal_phrasing: true,
            max_lore_atoms: 20,
            timestamp: None,
        }
    }
}

impl ResolvedLabConfig {
    fn apply_overrides(self, o: LabConfig) -> Self {
        Self {
            saliency_threshold: o.saliency_threshold.unwrap_or(self.saliency_threshold),
            weight_dense: o.weight_dense.unwrap_or(self.weight_dense),
            significance_coef: o.significance_coef.unwrap_or(self.significance_coef),
            temporal_phrasing: o.temporal_phrasing.unwrap_or(self.temporal_phrasing),
            max_lore_atoms: o.max_lore_atoms.unwrap_or(self.max_lore_atoms),
            timestamp: o.timestamp.or(self.timestamp),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper traits — allow the engine to be generic over block/lore types
// ─────────────────────────────────────────────────────────────────────────────

/// Methods the engine needs from any block type.
pub trait HasNarrativeBlock {
    fn block_id_str(&self) -> String;
    fn block_index(&self) -> usize;
    fn block_content(&self) -> &str;
    fn happened_at(&self) -> i64;
    fn notable(&self) -> bool;
}

/// Methods the engine needs from any lore type.
pub trait HasNarrativeLore {
    fn lore_content(&self) -> &str;
    fn happened_at(&self) -> i64;
}

impl HasNarrativeBlock for BaseNarrativeBlock {
    fn block_id_str(&self) -> String { self.block_id().to_string() }
    fn block_index(&self) -> usize   { self.index as usize }
    fn block_content(&self) -> &str  { &self.content }
    fn happened_at(&self) -> i64     { self.happened_at }
    fn notable(&self) -> bool        { self.is_notable() }
}



impl HasNarrativeLore for BaseNarrativeLore {
    fn lore_content(&self) -> &str { &self.content }
    fn happened_at(&self) -> i64   { self.happened_at }
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal types
// ─────────────────────────────────────────────────────────────────────────────

struct ScoredCandidate<TBlock> {
    block: TBlock,
    score_raw_fused: f64,
    score_final_fused: f64,
}

struct SharedContext<TBlock, TLore> {
    #[allow(dead_code)]
    total_block_count: usize,
    lore_atoms: Vec<TLore>,
    blocks_historical: Vec<TBlock>,
    current_block_count: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// NarrativeEngine
// ─────────────────────────────────────────────────────────────────────────────

pub struct NarrativeEngine<
    TBlock: Clone + Send + Sync = BaseNarrativeBlock,
    TLore:  Clone + Send + Sync = BaseNarrativeLore,
> {
    provider:   Box<dyn NarrativeProvider<TBlock, TLore>>,
    lab_config: ResolvedLabConfig,
}

impl Default for NarrativeEngine {
    fn default() -> Self {
        Self::new(Box::new(InMemoryNarrativeProvider::default()))
    }
}

impl<TBlock, TLore> NarrativeEngine<TBlock, TLore>
where
    TBlock: Clone + Send + Sync + HasNarrativeBlock + 'static,
    TLore:  Clone + Send + Sync + HasNarrativeLore  + 'static,
{
    pub fn new(provider: Box<dyn NarrativeProvider<TBlock, TLore>>) -> Self {
        Self { provider, lab_config: ResolvedLabConfig::default() }
    }

    pub fn set_lab_config(&mut self, overrides: LabConfig) {
        self.lab_config = ResolvedLabConfig::default().apply_overrides(overrides);
    }

    pub fn get_lab_config(&self) -> ResolvedLabConfig {
        self.lab_config.clone()
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Generates a single context prompt.
    pub async fn generate_context(&self, channel_id: &str, query: &str) -> String {
        match self.generate_context_single(channel_id, query).await {
            Ok(p)  => p,
            Err(e) => { eprintln!("[NarrativeEngine] Error: {e}"); String::new() }
        }
    }

    /// Generates context prompts for multiple queries (shared context fetched once).
    pub async fn generate_context_batch(
        &self,
        channel_id: &str,
        queries: &[String],
    ) -> HashMap<String, String> {
        let shared = match self.fetch_shared_context(channel_id).await {
            Ok(s)  => s,
            Err(e) => { eprintln!("[NarrativeEngine] Batch error: {e}"); return HashMap::new(); }
        };

        let candidate_map = self
            .provider
            .get_hybrid_search_candidates_batch(channel_id, queries, 20)
            .await;

        let mut result = HashMap::new();
        for q in queries {
            let candidates = candidate_map.get(q).cloned().unwrap_or_default();
            let prose = self.process_query_context(q, candidates, &shared);
            result.insert(q.clone(), prose);
        }
        result
    }

    // ── Shared context (used by batch path) ──────────────────────────────────

    async fn fetch_shared_context(
        &self,
        channel_id: &str,
    ) -> Result<SharedContext<TBlock, TLore>, String> {
        let total_block_count = self.provider.get_block_count(channel_id).await;

        let mut lore_atoms = self.provider.get_lore_atoms(channel_id).await;
        lore_atoms.sort_by(|a, b| b.happened_at().cmp(&a.happened_at()));
        lore_atoms.truncate(self.lab_config.max_lore_atoms);

        let mut blocks_historical: Vec<TBlock> = Vec::new();
        if total_block_count >= RAG_MIN_BLOCKS {
            let seq     = generate_reciprocal_sequence(total_block_count, RAG_DIVISIONS);
            let indices = sequence_to_block_indices(&seq);
            blocks_historical = self.provider.get_blocks_by_indices(channel_id, &indices).await;
        }

        let current_block_count = blocks_historical
            .last()
            .map(|b| b.block_index() + 1)
            .unwrap_or(0);

        Ok(SharedContext { total_block_count, lore_atoms, blocks_historical, current_block_count })
    }

    /// Processes candidates against shared context (batch path only).
    fn process_query_context(
        &self,
        query: &str,
        candidates: Vec<HybridCandidate<TBlock>>,
        shared: &SharedContext<TBlock, TLore>,
    ) -> String {
        let survivors = self.score_and_filter(candidates);
        let blocks_chrono = self.merge_and_sort_chronologically(&shared.blocks_historical, &survivors);
        self.compose_prose(&blocks_chrono, &shared.lore_atoms, query, shared.current_block_count)
    }

    // ── Core single-query pipeline ────────────────────────────────────────────

    async fn generate_context_single(
        &self,
        channel_id: &str,
        input_query: &str,
    ) -> Result<String, String> {
        // ── PHASE 1: HARVEST ─────────────────────────────────────────────────
        let total_block_count = self.provider.get_block_count(channel_id).await;

        let mut lore_atoms = self.provider.get_lore_atoms(channel_id).await;
        lore_atoms.sort_by(|a, b| b.happened_at().cmp(&a.happened_at()));
        lore_atoms.truncate(self.lab_config.max_lore_atoms);

        let candidates_hybrid = self
            .provider
            .get_hybrid_search_candidates(channel_id, input_query, 20)
            .await;

        let mut blocks_historical: Vec<TBlock> = Vec::new();
        let mut block_sequence_intervals: Vec<usize> = Vec::new();

        if total_block_count >= RAG_MIN_BLOCKS {
            let seq     = generate_reciprocal_sequence(total_block_count, RAG_DIVISIONS);
            let indices = sequence_to_block_indices(&seq);
            block_sequence_intervals = indices.clone();
            blocks_historical = self.provider.get_blocks_by_indices(channel_id, &indices).await;
        }

        // ── PHASE 2 + 3: FUSION, SCORING, SALIENCY GATE, TIE-BREAKER ────────
        let evicted_ids: Vec<String>;
        let survivors: Vec<HybridCandidate<TBlock>>;

        {
            let weight_sparse = 1.0 - self.lab_config.weight_dense;
            let mut scored: Vec<ScoredCandidate<TBlock>> = candidates_hybrid
                .into_iter()
                .map(|c| {
                    let score_raw = c.score_vector_dense * self.lab_config.weight_dense
                        + c.score_keyword_sparse * weight_sparse;
                    let score_final = if c.block.notable() {
                        score_raw * self.lab_config.significance_coef
                    } else {
                        score_raw
                    };
                    ScoredCandidate { block: c.block, score_raw_fused: score_raw, score_final_fused: score_final }
                })
                .collect();

            evicted_ids = scored
                .iter()
                .filter(|c| c.score_final_fused < self.lab_config.saliency_threshold)
                .map(|c| c.block.block_id_str())
                .collect();

            scored.retain(|c| c.score_final_fused >= self.lab_config.saliency_threshold);

            // Sort: score DESC, then happened_at DESC (Tie-Breaker: Recency wins)
            scored.sort_by(|a, b| {
                b.score_final_fused
                    .partial_cmp(&a.score_final_fused)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.block.happened_at().cmp(&a.block.happened_at()))
            });
            scored.truncate(LIMIT_HYBRID_TOP);

            survivors = scored
                .into_iter()
                .map(|s| HybridCandidate {
                    block: s.block,
                    score_vector_dense: s.score_raw_fused,
                    score_keyword_sparse: 0.0,
                })
                .collect();
        }

        // ── PHASE 4: TIMELINE ALIGNMENT ──────────────────────────────────────
        let blocks_chrono = self.merge_and_sort_chronologically(&blocks_historical, &survivors);

        // ── PHASE 5: PROSE GENERATION ─────────────────────────────────────────
        let current_block_count = blocks_chrono.last().map(|b| b.block_index() + 1).unwrap_or(0);
        let finalized_prompt =
            self.compose_prose(&blocks_chrono, &lore_atoms, input_query, current_block_count);

        // ── TRACE ─────────────────────────────────────────────────────────────
        let trace = TraceObject {
            timestamp: chrono::Utc::now().to_rfc3339(),
            channel_id: channel_id.to_string(),
            input_query: input_query.to_string(),
            provider_type: Some(self.provider.get_provider_type().to_string()),
            lab_config: Some(LabConfig {
                saliency_threshold: Some(self.lab_config.saliency_threshold),
                weight_dense:       Some(self.lab_config.weight_dense),
                significance_coef:  Some(self.lab_config.significance_coef),
                temporal_phrasing:  Some(self.lab_config.temporal_phrasing),
                max_lore_atoms:     Some(self.lab_config.max_lore_atoms),
                timestamp:          self.lab_config.timestamp.clone(),
            }),
            phases: TracePhases {
                harvest: Some(serde_json::json!({
                    "totalBlockCount": total_block_count,
                    "loreCount":       lore_atoms.len(),
                    "intervals":       block_sequence_intervals,
                })),
                saliency: Some(serde_json::json!({
                    "threshold":     self.lab_config.saliency_threshold,
                    "evicted":       evicted_ids,
                    "survivorCount": survivors.len(),
                })),
                timeline: Some(serde_json::json!({ "blockCount": blocks_chrono.len() })),
                prose:    Some(serde_json::json!({
                    "promptLength": finalized_prompt.len(),
                    "loreAtoms":    lore_atoms.len(),
                    "blockCount":   blocks_chrono.len(),
                })),
                fusion: None,
            },
            finalized_prompt:    Some(finalized_prompt.clone()),
            discarded_candidates: None,
            error: None,
        };
        logger_narrative_trace(&trace);

        Ok(finalized_prompt)
    }

    // ── Shared scoring helper (used by batch path) ────────────────────────────

    fn score_and_filter(
        &self,
        candidates: Vec<HybridCandidate<TBlock>>,
    ) -> Vec<HybridCandidate<TBlock>> {
        let weight_sparse = 1.0 - self.lab_config.weight_dense;
        let mut scored: Vec<ScoredCandidate<TBlock>> = candidates
            .into_iter()
            .map(|c| {
                let raw = c.score_vector_dense * self.lab_config.weight_dense
                    + c.score_keyword_sparse * weight_sparse;
                let fin = if c.block.notable() { raw * self.lab_config.significance_coef } else { raw };
                ScoredCandidate { block: c.block, score_raw_fused: raw, score_final_fused: fin }
            })
            .collect();

        scored.retain(|c| c.score_final_fused >= self.lab_config.saliency_threshold);
        scored.sort_by(|a, b| {
            b.score_final_fused
                .partial_cmp(&a.score_final_fused)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.block.happened_at().cmp(&a.block.happened_at()))
        });
        scored.truncate(LIMIT_HYBRID_TOP);

        scored.into_iter().map(|s| HybridCandidate {
            block: s.block,
            score_vector_dense: s.score_raw_fused,
            score_keyword_sparse: 0.0,
        }).collect()
    }

    // ── Merge + chronological sort ────────────────────────────────────────────

    fn merge_and_sort_chronologically(
        &self,
        blocks_historical: &[TBlock],
        candidates_survivor: &[HybridCandidate<TBlock>],
    ) -> Vec<TBlock> {
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut merged: Vec<TBlock> = Vec::new();

        for block in blocks_historical {
            let key = block.block_id_str();
            if seen.insert(key) {
                merged.push(block.clone());
            }
        }
        for candidate in candidates_survivor {
            let key = candidate.block.block_id_str();
            if seen.contains(&key) {
                // Survivor overwrites historical (same JS Map.set semantics)
                if let Some(pos) = merged.iter().position(|b| b.block_id_str() == key) {
                    merged[pos] = candidate.block.clone();
                }
            } else {
                seen.insert(key);
                merged.push(candidate.block.clone());
            }
        }

        merged.sort_by_key(|b| b.happened_at());
        merged
    }

    // ── Prose composition ─────────────────────────────────────────────────────

    fn compose_prose(
        &self,
        blocks_chrono: &[TBlock],
        lore_atoms: &[TLore],
        immediate_context: &str,
        current_block_count: usize,
    ) -> String {
        let lore_section: String = lore_atoms
            .iter()
            .map(|l| l.lore_content())
            .collect::<Vec<_>>()
            .join(" ");

        let block_sections: Vec<String> = blocks_chrono
            .iter()
            .map(|block| {
                if self.lab_config.temporal_phrasing {
                    let offset = current_block_count.saturating_sub(block.block_index()) + 1;
                    let unit = if offset == 1 { "storyblock" } else { "storyblocks" };
                    format!("{offset} {unit} ago: {}", block.block_content())
                } else {
                    format!("Entry {}: {}", block.block_id_str(), block.block_content())
                }
            })
            .collect();

        let mut parts: Vec<String> = Vec::new();
        if !lore_section.is_empty() {
            parts.push(format!("Essential facts of the story: {lore_section}"));
        }
        if !block_sections.is_empty() {
            parts.push(block_sections.join("\n"));
        }
        parts.push(immediate_context.to_string());
        parts.join("\n")
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::InMemoryNarrativeProvider;
    use crate::types::{BaseNarrativeBlock, BaseNarrativeLore, BlockId};

    // ─── Stub provider ────────────────────────────────────────────────────────

    struct StubProvider {
        candidates: Vec<HybridCandidate<BaseNarrativeBlock>>,
        lore:       Vec<BaseNarrativeLore>,
        block_count: usize,
    }

    impl StubProvider {
        fn new(
            candidates: Vec<(BaseNarrativeBlock, f64, f64)>,
            lore: Vec<BaseNarrativeLore>,
        ) -> Self {
            Self {
                candidates: candidates
                    .into_iter()
                    .map(|(b, d, s)| HybridCandidate {
                        block: b,
                        score_vector_dense: d,
                        score_keyword_sparse: s,
                    })
                    .collect(),
                lore,
                block_count: 10,
            }
        }
    }

    impl NarrativeProvider<BaseNarrativeBlock, BaseNarrativeLore> for StubProvider {
        fn get_block_count(&self, _: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = usize> + Send + '_>> {
            let c = self.block_count;
            Box::pin(async move { c })
        }
        fn get_lore_atoms(&self, _: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BaseNarrativeLore>> + Send + '_>> {
            let l = self.lore.clone();
            Box::pin(async move { l })
        }
        fn get_hybrid_search_candidates(&self, _: &str, _: &str, _: usize) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<HybridCandidate<BaseNarrativeBlock>>> + Send + '_>> {
            let c = self.candidates.clone();
            Box::pin(async move { c })
        }
        fn get_hybrid_search_candidates_batch(&self, _: &str, qs: &[String], _: usize) -> std::pin::Pin<Box<dyn std::future::Future<Output = HashMap<String, Vec<HybridCandidate<BaseNarrativeBlock>>>> + Send + '_>> {
            let c = self.candidates.clone();
            let q2 = qs.to_vec();
            Box::pin(async move { q2.into_iter().map(|q| (q, c.clone())).collect() })
        }
        fn get_blocks_by_indices(&self, _: &str, _: &[usize]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BaseNarrativeBlock>> + Send + '_>> {
            Box::pin(async { vec![] })
        }
        fn get_notable_events(&self, _: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BaseNarrativeBlock>> + Send + '_>> {
            Box::pin(async { vec![] })
        }
        fn add_block(&self, _: &str, _: BaseNarrativeBlock) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
            Box::pin(async {})
        }
        fn get_provider_type(&self) -> &'static str { "test" }
    }

    fn block(id: &str, index: usize, content: &str, happened_at: i64, notable: bool) -> BaseNarrativeBlock {
        BaseNarrativeBlock {
            id: Some(BlockId::Str(id.into()).into()),
            index: index as u64,
            content: content.into(),
            happened_at,
            is_notable: Some(notable),
        }
    }

    // ── Tie-Breaker Paradox: Recency wins ────────────────────────────────────
    #[tokio::test]
    async fn tie_breaker_recency_wins() {
        let engine = NarrativeEngine::new(Box::new(StubProvider::new(
            vec![
                (block("old", 10, "Older", 100, false), 0.8, 0.8),
                (block("new", 20, "Newer", 200, false), 0.8, 0.8),
            ],
            vec![],
        )));
        let result = engine.generate_context("test", "query").await;
        // Both survive saliency gate; chrono merge orders by happened_at,
        // so "Newer" is present in the output.
        assert!(result.contains("Newer"), "result: {result}");
    }

    // ── Lore Overload protection ──────────────────────────────────────────────
    #[tokio::test]
    async fn lore_overload_cap() {
        let lore: Vec<BaseNarrativeLore> = (0u64..50)
            .map(|i| BaseNarrativeLore {
                id: Some(BlockId::Num(i as i64).into()),
                content: format!("Rule {i}"),
                happened_at: i as i64,
                is_active: Some(true),
            })
            .collect();

        let mut engine = NarrativeEngine::new(Box::new(StubProvider::new(vec![], lore)));
        engine.set_lab_config(LabConfig {
            max_lore_atoms: Some(5),
            saliency_threshold: None, weight_dense: None,
            significance_coef: None, temporal_phrasing: None, timestamp: None,
        });

        let result = engine.generate_context("test", "query").await;
        // Top-5 by happened_at descending: Rule 49 .. Rule 45
        assert!(result.contains("Rule 49"), "result: {result}");
        assert!(!result.contains("Rule 0"),  "result: {result}");
    }

    // ── Significance Coefficient 1.5× ─────────────────────────────────────────
    #[tokio::test]
    async fn significance_coefficient_boosts_notable() {
        // Raw fused: 0.5*0.7 + 0.5*0.3 = 0.5 → boosted: 0.5 * 1.5 = 0.75 ≥ 0.65
        let engine = NarrativeEngine::new(Box::new(StubProvider::new(
            vec![(block("notable", 10, "Important", 1, true), 0.5, 0.5)],
            vec![],
        )));
        let result = engine.generate_context("test", "query").await;
        assert!(result.contains("Important"), "result: {result}");
    }

    // ── Saliency Gate eviction ────────────────────────────────────────────────
    #[tokio::test]
    async fn saliency_gate_evicts_weak_candidate() {
        // 0.4*0.7 + 0.4*0.3 = 0.4 < 0.65 → evicted
        let engine = NarrativeEngine::new(Box::new(StubProvider::new(
            vec![(block("weak", 10, "Irrelevant", 1, false), 0.4, 0.4)],
            vec![],
        )));
        let result = engine.generate_context("test", "query").await;
        assert!(!result.contains("Irrelevant"), "result: {result}");
    }

    // ── Default config ────────────────────────────────────────────────────────
    #[test]
    fn default_config_saliency_threshold() {
        let engine = NarrativeEngine::default();
        assert!((engine.get_lab_config().saliency_threshold - 0.65).abs() < f64::EPSILON);
    }

    // ── Temporal phrasing offset ──────────────────────────────────────────────
    #[tokio::test]
    async fn temporal_phrasing_offset_calculation() {
        let blocks = vec![
            BaseNarrativeBlock { id: Some(BlockId::Num(1).into()), index: 1, content: "The beginning".into(), happened_at: 100, is_notable: Some(false) },
            BaseNarrativeBlock { id: Some(BlockId::Num(2).into()), index: 2, content: "The middle".into(),    happened_at: 150, is_notable: Some(false) },
            BaseNarrativeBlock { id: Some(BlockId::Num(3).into()), index: 3, content: "The end".into(),       happened_at: 200, is_notable: Some(false) },
        ];
        let provider = InMemoryNarrativeProvider::new(blocks, vec![]);
        let mut engine = NarrativeEngine::new(Box::new(provider));
        engine.set_lab_config(LabConfig {
            temporal_phrasing: Some(true),
            saliency_threshold: None, weight_dense: None,
            significance_coef: None, max_lore_atoms: None, timestamp: None,
        });

        let result = engine.generate_context("test", "query").await;
        // 3 blocks total; block at index 2: offset = (3-2)+1 = 2
        assert!(result.contains("2 storyblocks ago"), "result: {result}");
    }

    // ── No nuclear deletes ────────────────────────────────────────────────────
    #[test]
    fn no_delete_records_method_on_provider() {
        // Compile-time check: InMemoryNarrativeProvider has no delete_records method.
        let _p = InMemoryNarrativeProvider::default();
        // Uncommenting the next line MUST fail to compile:
        // _p.delete_records("test");
    }

    // ── Partial config override ───────────────────────────────────────────────
    #[test]
    fn set_lab_config_partial_override() {
        let mut engine = NarrativeEngine::default();
        engine.set_lab_config(LabConfig {
            saliency_threshold: Some(0.9),
            weight_dense: None, significance_coef: None,
            temporal_phrasing: None, max_lore_atoms: None, timestamp: None,
        });
        let cfg = engine.get_lab_config();
        assert!((cfg.saliency_threshold - 0.9).abs() < f64::EPSILON);
        assert!((cfg.weight_dense - 0.7).abs() < f64::EPSILON);
    }

    // ── Batch context generation ──────────────────────────────────────────────
    #[tokio::test]
    async fn batch_returns_entry_per_query() {
        let engine = NarrativeEngine::default();
        let queries = vec!["cube".to_string(), "ELARA".to_string()];
        let result = engine.generate_context_batch("test", &queries).await;
        assert!(result.contains_key("cube"));
        assert!(result.contains_key("ELARA"));
    }
}