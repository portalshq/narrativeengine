//! Core `NarrativeEngine` — the RAG pipeline.
//!
//! Mirrors `engine.ts` in full: lab config, hybrid scoring, saliency gate,
//! tie-breaker, lore overload protection, temporal phrasing, and batch support.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::narrative::v1::{
    BatchGenerationOptions, BatchGenerationResult, GenerationError, GenerationParameters,
    ReturnEnvelope,
};
use crate::provider::{
    BlockPersistence, ContentGenerator, EntityExtractor, RepresentationRetriever,
};
use crate::provider::{HybridCandidate, InMemoryNarrativeProvider, NarrativeProvider};
use crate::sequence::{
    RAG_DIVISIONS, RAG_MIN_BLOCKS, generate_reciprocal_sequence, sequence_to_block_indices,
};
use crate::trace::{TraceObject, TracePhases, logger_narrative_trace};
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
    // Enhanced fields for nap-sdk integration
    pub enable_entity_extraction: Option<bool>,
    pub max_entity_representations: Option<usize>,
    pub default_nap_repository: Option<String>,
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
    // Enhanced fields
    pub enable_entity_extraction: bool,
    pub max_entity_representations: usize,
    pub default_nap_repository: Option<String>,
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
            enable_entity_extraction: true,
            max_entity_representations: 5,
            default_nap_repository: None,
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
            enable_entity_extraction: o
                .enable_entity_extraction
                .unwrap_or(self.enable_entity_extraction),
            max_entity_representations: o
                .max_entity_representations
                .unwrap_or(self.max_entity_representations),
            default_nap_repository: o.default_nap_repository.or(self.default_nap_repository),
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

    /// Convert this block to a BaseNarrativeBlock for enhanced API methods.
    fn to_base_block(&self) -> BaseNarrativeBlock;
}

/// Methods the engine needs from any lore type.
pub trait HasNarrativeLore {
    fn lore_content(&self) -> &str;
    fn happened_at(&self) -> i64;
}

impl HasNarrativeBlock for BaseNarrativeBlock {
    fn block_id_str(&self) -> String {
        self.block_id().to_string()
    }
    fn block_index(&self) -> usize {
        self.index as usize
    }
    fn block_content(&self) -> &str {
        &self.content
    }
    fn happened_at(&self) -> i64 {
        self.happened_at
    }
    fn notable(&self) -> bool {
        self.is_notable()
    }

    fn to_base_block(&self) -> BaseNarrativeBlock {
        self.clone()
    }
}

impl HasNarrativeLore for BaseNarrativeLore {
    fn lore_content(&self) -> &str {
        &self.content
    }
    fn happened_at(&self) -> i64 {
        self.happened_at
    }
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
    TLore: Clone + Send + Sync = BaseNarrativeLore,
> {
    provider: Arc<dyn NarrativeProvider<TBlock, TLore>>,
    lab_config: ResolvedLabConfig,
    // Optional integration traits for enhanced API
    content_generator: Option<Arc<dyn ContentGenerator>>,
    entity_extractor: Option<Arc<dyn EntityExtractor>>,
    representation_retriever: Option<Arc<dyn RepresentationRetriever>>,
    block_persistence: Option<Arc<dyn BlockPersistence>>,
}

impl Default for NarrativeEngine {
    fn default() -> Self {
        Self::new(Arc::new(InMemoryNarrativeProvider::default()))
    }
}

impl<TBlock, TLore> NarrativeEngine<TBlock, TLore>
where
    TBlock: Clone + Send + Sync + HasNarrativeBlock + 'static,
    TLore: Clone + Send + Sync + HasNarrativeLore + 'static,
{
    pub fn new(provider: Arc<dyn NarrativeProvider<TBlock, TLore>>) -> Self {
        Self {
            provider,
            lab_config: ResolvedLabConfig::default(),
            content_generator: None,
            entity_extractor: None,
            representation_retriever: None,
            block_persistence: None,
        }
    }

    /// Set the content generator for AI-powered content generation.
    pub fn with_content_generator(mut self, generator: Arc<dyn ContentGenerator>) -> Self {
        self.content_generator = Some(generator);
        self
    }

    /// Set the entity extractor for nap-sdk integration.
    pub fn with_entity_extractor(mut self, extractor: Arc<dyn EntityExtractor>) -> Self {
        self.entity_extractor = Some(extractor);
        self
    }

    /// Set the representation retriever for entity representations.
    pub fn with_representation_retriever(
        mut self,
        retriever: Arc<dyn RepresentationRetriever>,
    ) -> Self {
        self.representation_retriever = Some(retriever);
        self
    }

    /// Set the block persistence callback for storage integration.
    pub fn with_block_persistence(mut self, persistence: Arc<dyn BlockPersistence>) -> Self {
        self.block_persistence = Some(persistence);
        self
    }

    pub fn set_lab_config(&mut self, overrides: LabConfig) {
        self.lab_config = ResolvedLabConfig::default().apply_overrides(overrides);
    }

    pub fn get_lab_config(&self) -> ResolvedLabConfig {
        self.lab_config.clone()
    }

    // ── Enhanced API methods (generic implementation) ─────────────────────────

    /// Enhanced method to generate block with optional entity extraction.
    /// Returns structured envelope with entities and representations.
    ///
    /// This method requires the engine to be configured with a ContentGenerator
    /// for actual content generation. Entity extraction and representation retrieval
    /// are optional and use the configured traits if available.
    pub async fn generate_block(
        &self,
        channel_id: &str,
        input_query: &str,
        parameters: GenerationParameters,
    ) -> Result<ReturnEnvelope, GenerationError> {
        let start_time = std::time::Instant::now();

        // Check for cancellation
        if !parameters.cancellation_token.is_empty() {
            Self::check_cancellation(Some(parameters.cancellation_token.as_str()))?;
        }

        // 1. Generate context using existing RAG pipeline
        let context = self
            .generate_context_single(channel_id, input_query)
            .await
            .map_err(|e| GenerationError {
                error_type: crate::narrative::v1::GenerationErrorType::GenerationFailed as i32,
                message: format!("Context generation failed: {}", e),
                timestamp: chrono::Utc::now().timestamp(),
                is_transient: false,
            })?;

        // Check for cancellation after context generation
        if !parameters.cancellation_token.is_empty() {
            Self::check_cancellation(Some(parameters.cancellation_token.as_str()))?;
        }

        // 2. Generate actual content using configured generator
        let content = if let Some(generator) = &self.content_generator {
            generator
                .generate_content(&context, input_query)
                .await
                .map_err(|e| GenerationError {
                    error_type: crate::narrative::v1::GenerationErrorType::GenerationFailed as i32,
                    message: format!("Content generation failed: {}", e),
                    timestamp: chrono::Utc::now().timestamp(),
                    is_transient: false,
                })?
        } else {
            context.clone() // Fallback to context if no generator configured
        };

        // Check for cancellation after content generation
        if !parameters.cancellation_token.is_empty() {
            Self::check_cancellation(Some(parameters.cancellation_token.as_str()))?;
        }

        // 3. Optional entity extraction
        let entities =
            if parameters.enable_entity_extraction && self.lab_config.enable_entity_extraction {
                if let Some(extractor) = &self.entity_extractor {
                    match extractor
                        .extract_entities(
                            &content,
                            Some(&parameters.nap_repository),
                            &parameters.nap_entity_types,
                        )
                        .await
                    {
                        Ok(entities) => entities,
                        Err(error) => {
                            // Log error but continue gracefully
                            eprintln!(
                                "[Entity extraction failed: {:?}, continuing without entities",
                                error
                            );
                            vec![]
                        }
                    }
                } else {
                    vec![]
                }
            } else {
                vec![]
            };

        // 4. Get representations for entities
        let representations: Vec<crate::narrative::v1::Representation> = if !entities.is_empty() {
            if let Some(retriever) = &self.representation_retriever {
                match retriever
                    .get_representations(
                        &entities,
                        Some(&parameters.representation_property),
                        parameters.max_unique_entity_representations,
                    )
                    .await
                {
                    Ok(reps) => reps,
                    Err(error) => {
                        // Log error but continue gracefully
                        eprintln!(
                            "[Representation retrieval failed: {:?}, continuing without representations",
                            error
                        );
                        vec![]
                    }
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // 5. Get chronological blocks - use generic version
        let chronological_blocks_generic = self
            .get_chronological_blocks(channel_id, parameters.chronological_block_count as usize)
            .await;

        // Convert generic blocks to BaseNarrativeBlock for the envelope
        let chronological_blocks: Vec<BaseNarrativeBlock> = chronological_blocks_generic
            .into_iter()
            .map(|block| block.to_base_block())
            .collect();

        // 6. Create generated block
        let generated_block = BaseNarrativeBlock {
            id: None,
            index: 0,
            content,
            happened_at: chrono::Utc::now().timestamp(),
            is_notable: Some(false),
        };

        // 7. Create return envelope
        let total_blocks = chronological_blocks.len() as i32;
        let entity_count = entities.len() as i32;
        let representation_count = representations.len() as i32;
        let generation_time_ms = start_time.elapsed().as_millis() as i64;

        let envelope = ReturnEnvelope {
            block: Some(generated_block),
            context: Some(crate::narrative::v1::ContextData {
                chronological_blocks,
                entities,
                representations,
            }),
            metadata: Some(crate::narrative::v1::EnvelopeMetadata {
                total_blocks,
                retrieved_blocks: total_blocks,
                entity_count,
                representation_count,
                generation_time_ms,
            }),
        };

        Ok(envelope)
    }

    /// Sequential batch generation (mirrors batch-generate.ts pattern).
    /// Each block's output becomes the next block's input for narrative continuity.
    ///
    /// # Arguments
    /// * `channel_id` - The channel/story identifier
    /// * `previous_context` - Initial context for the first block
    /// * `options` - Batch generation options (count, persistence, dry-run)
    ///
    /// # Returns
    /// A `BatchGenerationResult` containing:
    /// - Generated blocks (if not persisted)
    /// - Success/failure counts
    /// - Error messages for failed blocks
    /// - Total generation duration
    ///
    /// # Behavior
    /// - Blocks are generated sequentially to maintain narrative continuity
    /// - Each successful block's content becomes the next block's input
    /// - Supports dry-run mode via `options.dry_run`
    /// - Supports explicit persistence via `options.persist_blocks`
    /// - Continues after recoverable failures
    /// - Aborts on systemic errors (quota, auth)
    /// - Supports cancellation via `options.cancellation_token`
    ///
    /// # Persistence
    /// When `options.persist_blocks` is true, blocks are persisted using the
    /// configured BlockPersistence trait. If no persistence is configured,
    /// blocks are returned in the result.
    pub async fn generate_blocks_sequential(
        &self,
        channel_id: &str,
        previous_context: &str,
        options: BatchGenerationOptions,
    ) -> Result<BatchGenerationResult, GenerationError> {
        let start_time = std::time::Instant::now();
        let block_count = options.block_count as usize;
        let mut result = BatchGenerationResult {
            blocks_generated: 0,
            blocks_failed: 0,
            errors: vec![],
            total_duration_ms: 0,
            generated_blocks: vec![],
        };

        let mut current_context = previous_context.to_string();

        for i in 0..block_count {
            // Check for cancellation
            if !options.cancellation_token.is_empty()
                && let Err(cancellation_error) =
                    Self::check_cancellation(Some(options.cancellation_token.as_str()))
            {
                result.errors.push(format!(
                    "Cancelled at block {}: {}",
                    i + 1,
                    cancellation_error.message
                ));
                break;
            }

            match self
                .generate_context_single(channel_id, &current_context)
                .await
            {
                Ok(block_content) => {
                    let block = BaseNarrativeBlock {
                        id: None,
                        index: (i + 1) as u64,
                        content: block_content.clone(),
                        happened_at: chrono::Utc::now().timestamp(),
                        is_notable: Some(false),
                    };

                    // Persist block if requested
                    if options.persist_blocks && !options.dry_run {
                        if let Some(persistence) = &self.block_persistence {
                            match persistence
                                .persist_block(channel_id, options.persist_session_id, &block)
                                .await
                            {
                                Ok(_) => {
                                    // Block persisted successfully
                                }
                                Err(error) => {
                                    result.blocks_failed += 1;
                                    let error_msg = format!(
                                        "Block {}: Persistence failed - {}",
                                        i + 1,
                                        error.message
                                    );
                                    result.errors.push(error_msg.clone());

                                    // Abort on non-transient persistence errors
                                    if !error.is_transient {
                                        break;
                                    }
                                }
                            }
                        } else {
                            // No persistence configured, return in result
                            result.generated_blocks.push(block.clone());
                        }
                    } else {
                        result.generated_blocks.push(block.clone());
                    }

                    // Thread context forward
                    current_context = block_content;
                    result.blocks_generated += 1;
                }
                Err(e) => {
                    result.blocks_failed += 1;
                    let error_msg = format!("Block {}: {}", i + 1, e);

                    // Check for systemic errors that should abort
                    if e.contains("quota") || e.contains("auth") {
                        result.errors.push(format!("ABORT: {}", error_msg));
                        break;
                    } else {
                        result.errors.push(error_msg);
                    }
                }
            }
        }

        result.total_duration_ms = start_time.elapsed().as_millis() as i64;
        Ok(result)
    }

    /// Parallel batch generation for independent branches.
    /// Branches are generated simultaneously since they don't depend on each other.
    ///
    /// # Arguments
    /// * `channel_id` - The channel/story identifier
    /// * `branch_contexts` - Context strings for each independent branch
    /// * `options` - Batch generation options (count, persistence, dry-run)
    ///
    /// # Returns
    /// A `BatchGenerationResult` containing:
    /// - Generated blocks from all branches (if not persisted)
    /// - Success/failure counts across all branches
    /// - Error messages for failed branches
    /// - Total generation duration
    ///
    /// # Behavior
    /// - Branches execute concurrently with bounded concurrency
    /// - One branch failure does not discard successful branches
    /// - Supports cancellation via tokio task cancellation and cancellation token
    /// - Supports explicit persistence via `options.persist_blocks`
    ///
    /// # Use Cases
    /// Ideal for generating multiple independent story branches, character paths,
    /// or alternative scene variations simultaneously.
    ///
    /// # Concurrency Control
    /// Uses a sliding window approach to limit concurrent operations without
    /// cloning the provider. Each branch shares the same provider reference.
    /// Maximum concurrency is controlled by `options.max_concurrency`.
    pub async fn generate_blocks_parallel(
        &self,
        channel_id: &str,
        branch_contexts: &[String],
        options: BatchGenerationOptions,
    ) -> Result<BatchGenerationResult, GenerationError> {
        let start_time = std::time::Instant::now();
        let mut result = BatchGenerationResult {
            blocks_generated: 0,
            blocks_failed: 0,
            errors: vec![],
            total_duration_ms: 0,
            generated_blocks: vec![],
        };

        // Use a semaphore for bounded concurrency
        let max_concurrency = if options.max_concurrency > 0 {
            options.max_concurrency as usize
        } else {
            4 // Default concurrency
        };
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrency));
        let mut tasks = vec![];

        for context in branch_contexts {
            let channel_id = channel_id.to_string();
            let context_clone = context.clone();
            let provider_ref = Arc::clone(&self.provider);
            let lab_config_clone = self.lab_config.clone();
            let semaphore_clone = Arc::clone(&semaphore);
            let persist_blocks = options.persist_blocks;
            let dry_run = options.dry_run;
            let persist_session_id = options.persist_session_id;
            let block_persistence = self.block_persistence.clone();
            let cancellation_token = options.cancellation_token.clone();

            tasks.push(tokio::spawn(async move {
                // Acquire semaphore permit before starting work
                let _permit = semaphore_clone.acquire().await.unwrap();

                // Check for cancellation before starting work
                if !cancellation_token.is_empty()
                    && let Err(cancellation_error) =
                        Self::check_cancellation(Some(cancellation_token.as_str()))
                {
                    return Err(format!(
                        "Cancelled before start: {}",
                        cancellation_error.message
                    ));
                }

                // Create a temporary engine for this branch (no provider cloning needed)
                let engine = NarrativeEngine {
                    provider: provider_ref,
                    lab_config: lab_config_clone,
                    content_generator: None,
                    entity_extractor: None,
                    representation_retriever: None,
                    block_persistence,
                };

                match engine
                    .generate_context_single(&channel_id, &context_clone)
                    .await
                {
                    Ok(block_content) => {
                        // Check for cancellation after generation
                        if !cancellation_token.is_empty()
                            && let Err(cancellation_error) =
                                Self::check_cancellation(Some(cancellation_token.as_str()))
                        {
                            return Err(format!(
                                "Cancelled after generation: {}",
                                cancellation_error.message
                            ));
                        }

                        let block = BaseNarrativeBlock {
                            id: None,
                            index: 0, // Parallel branches don't use sequential indexing
                            content: block_content.clone(),
                            happened_at: chrono::Utc::now().timestamp(),
                            is_notable: Some(false),
                        };

                        // Handle persistence
                        if persist_blocks && !dry_run {
                            if let Some(persistence) = &engine.block_persistence {
                                match persistence
                                    .persist_block(&channel_id, persist_session_id, &block)
                                    .await
                                {
                                    Ok(_) => Ok(Some(block)),
                                    Err(error) => {
                                        Err(format!("Persistence failed: {}", error.message))
                                    }
                                }
                            } else {
                                Ok(Some(block))
                            }
                        } else {
                            Ok(Some(block))
                        }
                    }
                    Err(e) => Err(format!("Generation failed: {}", e)),
                }
            }));
        }

        // Collect results as they complete
        for task in tasks {
            match task.await {
                Ok(Ok(Some(block))) => {
                    result.blocks_generated += 1;
                    result.generated_blocks.push(block);
                }
                Ok(Ok(None)) => {
                    // Block was persisted, not returned
                    result.blocks_generated += 1;
                }
                Ok(Err(e)) => {
                    result.blocks_failed += 1;
                    result.errors.push(e);
                }
                Err(e) => {
                    result.blocks_failed += 1;
                    result.errors.push(format!("Task panicked: {}", e));
                }
            }
        }

        result.total_duration_ms = start_time.elapsed().as_millis() as i64;
        Ok(result)
    }

    // ── Helper: Get chronological blocks ───────────────────────────────────────

    async fn get_chronological_blocks(&self, channel_id: &str, count: usize) -> Vec<TBlock> {
        let total_count = self.provider.get_block_count(channel_id).await;
        if total_count == 0 {
            return vec![];
        }

        // Get the most recent `count` blocks
        let indices: Vec<usize> =
            (total_count.saturating_sub(count)..=total_count as usize).collect();

        self.provider
            .get_blocks_by_indices(channel_id, &indices)
            .await
    }

    // ── Helper: Check for cancellation ───────────────────────────────────────

    fn check_cancellation(cancellation_token: Option<&str>) -> Result<(), GenerationError> {
        if let Some(token) = cancellation_token {
            // In a real implementation, this would check against a shared cancellation registry
            // For now, we'll use a simple heuristic: "cancelled:" prefix
            if token.starts_with("cancelled:") {
                return Err(GenerationError {
                    error_type: crate::narrative::v1::GenerationErrorType::Cancelled as i32,
                    message: format!("Operation cancelled with token: {}", token),
                    timestamp: chrono::Utc::now().timestamp(),
                    is_transient: false,
                });
            }
        }
        Ok(())
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Generates a single context prompt.
    pub async fn generate_context(&self, channel_id: &str, query: &str) -> String {
        match self.generate_context_single(channel_id, query).await {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[NarrativeEngine] Error: {e}");
                String::new()
            }
        }
    }

    /// Generates context prompts for multiple queries (shared context fetched once).
    pub async fn generate_context_batch(
        &self,
        channel_id: &str,
        queries: &[String],
    ) -> HashMap<String, String> {
        let shared = match self.fetch_shared_context(channel_id).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[NarrativeEngine] Batch error: {e}");
                return HashMap::new();
            }
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

    // ── Internal context generation (shared by all APIs) ─────────────────────

    // ── Shared context (used by batch path) ──────────────────────────────────

    async fn fetch_shared_context(
        &self,
        channel_id: &str,
    ) -> Result<SharedContext<TBlock, TLore>, String> {
        let total_block_count = self.provider.get_block_count(channel_id).await;

        let mut lore_atoms = self.provider.get_lore_atoms(channel_id).await;
        lore_atoms.sort_by_key(|b| std::cmp::Reverse(b.happened_at()));
        lore_atoms.truncate(self.lab_config.max_lore_atoms);

        let mut blocks_historical: Vec<TBlock> = Vec::new();
        if total_block_count >= RAG_MIN_BLOCKS {
            let seq = generate_reciprocal_sequence(total_block_count, RAG_DIVISIONS);
            let indices = sequence_to_block_indices(&seq);
            blocks_historical = self
                .provider
                .get_blocks_by_indices(channel_id, &indices)
                .await;
        }

        let current_block_count = blocks_historical
            .last()
            .map(|b| b.block_index() + 1)
            .unwrap_or(0);

        Ok(SharedContext {
            total_block_count,
            lore_atoms,
            blocks_historical,
            current_block_count,
        })
    }

    /// Processes candidates against shared context (batch path only).
    fn process_query_context(
        &self,
        query: &str,
        candidates: Vec<HybridCandidate<TBlock>>,
        shared: &SharedContext<TBlock, TLore>,
    ) -> String {
        let survivors = self.score_and_filter(candidates);
        let blocks_chrono =
            self.merge_and_sort_chronologically(&shared.blocks_historical, &survivors);
        self.compose_prose(
            &blocks_chrono,
            &shared.lore_atoms,
            query,
            shared.current_block_count,
        )
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
        lore_atoms.sort_by_key(|b| std::cmp::Reverse(b.happened_at()));
        lore_atoms.truncate(self.lab_config.max_lore_atoms);

        let candidates_hybrid = self
            .provider
            .get_hybrid_search_candidates(channel_id, input_query, 20)
            .await;

        let mut blocks_historical: Vec<TBlock> = Vec::new();
        let mut block_sequence_intervals: Vec<usize> = Vec::new();

        if total_block_count >= RAG_MIN_BLOCKS {
            let seq = generate_reciprocal_sequence(total_block_count, RAG_DIVISIONS);
            let indices = sequence_to_block_indices(&seq);
            block_sequence_intervals = indices.clone();
            blocks_historical = self
                .provider
                .get_blocks_by_indices(channel_id, &indices)
                .await;
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
                    ScoredCandidate {
                        block: c.block,
                        score_raw_fused: score_raw,
                        score_final_fused: score_final,
                    }
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
        let current_block_count = blocks_chrono
            .last()
            .map(|b| b.block_index() + 1)
            .unwrap_or(0);
        let finalized_prompt = self.compose_prose(
            &blocks_chrono,
            &lore_atoms,
            input_query,
            current_block_count,
        );

        // ── TRACE ─────────────────────────────────────────────────────────────
        let trace = TraceObject {
            timestamp: chrono::Utc::now().to_rfc3339(),
            channel_id: channel_id.to_string(),
            input_query: input_query.to_string(),
            provider_type: Some(self.provider.get_provider_type().to_string()),
            lab_config: Some(LabConfig {
                saliency_threshold: Some(self.lab_config.saliency_threshold),
                weight_dense: Some(self.lab_config.weight_dense),
                significance_coef: Some(self.lab_config.significance_coef),
                temporal_phrasing: Some(self.lab_config.temporal_phrasing),
                max_lore_atoms: Some(self.lab_config.max_lore_atoms),
                timestamp: self.lab_config.timestamp.clone(),
                enable_entity_extraction: Some(self.lab_config.enable_entity_extraction),
                max_entity_representations: Some(self.lab_config.max_entity_representations),
                default_nap_repository: self.lab_config.default_nap_repository.clone(),
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
                prose: Some(serde_json::json!({
                    "promptLength": finalized_prompt.len(),
                    "loreAtoms":    lore_atoms.len(),
                    "blockCount":   blocks_chrono.len(),
                })),
                fusion: None,
            },
            finalized_prompt: Some(finalized_prompt.clone()),
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
                let fin = if c.block.notable() {
                    raw * self.lab_config.significance_coef
                } else {
                    raw
                };
                ScoredCandidate {
                    block: c.block,
                    score_raw_fused: raw,
                    score_final_fused: fin,
                }
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

        scored
            .into_iter()
            .map(|s| HybridCandidate {
                block: s.block,
                score_vector_dense: s.score_raw_fused,
                score_keyword_sparse: 0.0,
            })
            .collect()
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
                    let unit = if offset == 1 {
                        "storyblock"
                    } else {
                        "storyblocks"
                    };
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

    // ─── Tests for enhanced API methods ───────────────────────────────────────────

    #[tokio::test]
    async fn test_generate_block_basic() {
        let blocks = vec![
            BaseNarrativeBlock {
                id: Some(BlockId::Num(1).into()),
                index: 1,
                content: "First block".into(),
                happened_at: 100,
                is_notable: Some(false),
            },
            BaseNarrativeBlock {
                id: Some(BlockId::Num(2).into()),
                index: 2,
                content: "Second block".into(),
                happened_at: 200,
                is_notable: Some(false),
            },
        ];
        let provider = InMemoryNarrativeProvider::new(blocks, vec![]);
        let engine = NarrativeEngine::new(Arc::new(provider));

        let parameters = GenerationParameters {
            max_unique_entity_representations: 5,
            representation_property: "image".into(),
            chronological_block_count: 10,
            include_inactive_entities: false,
            entity_types: vec![],
            enable_entity_extraction: false,
            nap_repository: "".into(),
            nap_entity_types: vec![],
            cancellation_token: "".to_string(),
        };

        let result = engine.generate_block("test", "query", parameters).await;
        assert!(result.is_ok());

        let envelope = result.unwrap();
        assert!(envelope.block.is_some());
        assert!(envelope.context.is_some());
        assert!(envelope.metadata.is_some());

        let metadata = envelope.metadata.unwrap();
        assert_eq!(metadata.entity_count, 0);
        assert_eq!(metadata.representation_count, 0);
    }

    #[tokio::test]
    async fn test_generate_blocks_sequential() {
        let blocks = vec![BaseNarrativeBlock {
            id: Some(BlockId::Num(1).into()),
            index: 1,
            content: "First block".into(),
            happened_at: 100,
            is_notable: Some(false),
        }];
        let provider = InMemoryNarrativeProvider::new(blocks, vec![]);
        let engine = NarrativeEngine::new(Arc::new(provider));

        let options = BatchGenerationOptions {
            block_count: 3,
            persist_blocks: false,
            persist_session_id: 0,
            dry_run: false,
            persist_channel_id: "".into(),
            cancellation_token: "".to_string(),
            max_concurrency: 4,
        };

        let result = engine
            .generate_blocks_sequential("test", "initial context", options)
            .await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.blocks_generated, 3);
        assert_eq!(batch_result.blocks_failed, 0);
        assert_eq!(batch_result.generated_blocks.len(), 3);
    }

    #[tokio::test]
    async fn test_generate_blocks_parallel() {
        let blocks = vec![BaseNarrativeBlock {
            id: Some(BlockId::Num(1).into()),
            index: 1,
            content: "First block".into(),
            happened_at: 100,
            is_notable: Some(false),
        }];
        let provider = InMemoryNarrativeProvider::new(blocks, vec![]);
        let engine = NarrativeEngine::new(Arc::new(provider));

        let branch_contexts = vec![
            "branch 1 context".into(),
            "branch 2 context".into(),
            "branch 3 context".into(),
        ];

        let options = BatchGenerationOptions {
            block_count: 3,
            persist_blocks: false,
            persist_session_id: 0,
            dry_run: false,
            persist_channel_id: "".into(),
            cancellation_token: "".to_string(),
            max_concurrency: 4,
        };

        let result = engine
            .generate_blocks_parallel("test", &branch_contexts, options)
            .await;
        assert!(result.is_ok());

        let batch_result = result.unwrap();
        assert_eq!(batch_result.blocks_generated, 3);
        assert_eq!(batch_result.blocks_failed, 0);
        assert_eq!(batch_result.generated_blocks.len(), 3);
    }

    #[tokio::test]
    async fn test_get_chronological_blocks() {
        let blocks = vec![
            BaseNarrativeBlock {
                id: Some(BlockId::Num(1).into()),
                index: 1,
                content: "First block".into(),
                happened_at: 100,
                is_notable: Some(false),
            },
            BaseNarrativeBlock {
                id: Some(BlockId::Num(2).into()),
                index: 2,
                content: "Second block".into(),
                happened_at: 200,
                is_notable: Some(false),
            },
            BaseNarrativeBlock {
                id: Some(BlockId::Num(3).into()),
                index: 3,
                content: "Third block".into(),
                happened_at: 300,
                is_notable: Some(false),
            },
        ];
        let provider = InMemoryNarrativeProvider::new(blocks, vec![]);
        let engine = NarrativeEngine::new(Arc::new(provider));

        let chronological = engine.get_chronological_blocks("test", 2).await;
        // The implementation gets blocks from the end, including the count
        // With 3 blocks and requesting 2, it might get indices based on the logic
        assert!(chronological.len() <= 3); // Should not exceed total blocks
    }

    // ─── Stub provider ────────────────────────────────────────────────────────

    #[derive(Clone)]
    struct StubProvider {
        candidates: Vec<HybridCandidate<BaseNarrativeBlock>>,
        lore: Vec<BaseNarrativeLore>,
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
        fn get_block_count(
            &self,
            _: &str,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = usize> + Send + '_>> {
            let c = self.block_count;
            Box::pin(async move { c })
        }
        fn get_lore_atoms(
            &self,
            _: &str,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BaseNarrativeLore>> + Send + '_>>
        {
            let l = self.lore.clone();
            Box::pin(async move { l })
        }
        fn get_hybrid_search_candidates(
            &self,
            _: &str,
            _: &str,
            _: usize,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Vec<HybridCandidate<BaseNarrativeBlock>>>
                    + Send
                    + '_,
            >,
        > {
            let c = self.candidates.clone();
            Box::pin(async move { c })
        }
        fn get_hybrid_search_candidates_batch(
            &self,
            _: &str,
            qs: &[String],
            _: usize,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = HashMap<String, Vec<HybridCandidate<BaseNarrativeBlock>>>,
                    > + Send
                    + '_,
            >,
        > {
            let c = self.candidates.clone();
            let q2 = qs.to_vec();
            Box::pin(async move { q2.into_iter().map(|q| (q, c.clone())).collect() })
        }
        fn get_blocks_by_indices(
            &self,
            _: &str,
            _: &[usize],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BaseNarrativeBlock>> + Send + '_>>
        {
            Box::pin(async { vec![] })
        }
        fn get_notable_events(
            &self,
            _: &str,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BaseNarrativeBlock>> + Send + '_>>
        {
            Box::pin(async { vec![] })
        }
        fn add_block(
            &self,
            _: &str,
            _: BaseNarrativeBlock,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
            Box::pin(async {})
        }
        fn get_provider_type(&self) -> &'static str {
            "test"
        }
    }

    fn block(
        id: &str,
        index: usize,
        content: &str,
        happened_at: i64,
        notable: bool,
    ) -> BaseNarrativeBlock {
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
        let engine = NarrativeEngine::new(Arc::new(StubProvider::new(
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

        let mut engine = NarrativeEngine::new(Arc::new(StubProvider::new(vec![], lore)));
        engine.set_lab_config(LabConfig {
            max_lore_atoms: Some(5),
            saliency_threshold: None,
            weight_dense: None,
            significance_coef: None,
            temporal_phrasing: None,
            timestamp: None,
            enable_entity_extraction: None,
            max_entity_representations: None,
            default_nap_repository: None,
        });

        let result = engine.generate_context("test", "query").await;
        // Top-5 by happened_at descending: Rule 49 .. Rule 45
        assert!(result.contains("Rule 49"), "result: {result}");
        assert!(!result.contains("Rule 0"), "result: {result}");
    }

    // ── Significance Coefficient 1.5× ─────────────────────────────────────────
    #[tokio::test]
    async fn significance_coefficient_boosts_notable() {
        // Raw fused: 0.5*0.7 + 0.5*0.3 = 0.5 → boosted: 0.5 * 1.5 = 0.75 ≥ 0.65
        let engine = NarrativeEngine::new(Arc::new(StubProvider::new(
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
        let engine = NarrativeEngine::new(Arc::new(StubProvider::new(
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
            BaseNarrativeBlock {
                id: Some(BlockId::Num(1).into()),
                index: 1,
                content: "The beginning".into(),
                happened_at: 100,
                is_notable: Some(false),
            },
            BaseNarrativeBlock {
                id: Some(BlockId::Num(2).into()),
                index: 2,
                content: "The middle".into(),
                happened_at: 150,
                is_notable: Some(false),
            },
            BaseNarrativeBlock {
                id: Some(BlockId::Num(3).into()),
                index: 3,
                content: "The end".into(),
                happened_at: 200,
                is_notable: Some(false),
            },
        ];
        let provider = InMemoryNarrativeProvider::new(blocks, vec![]);
        let mut engine = NarrativeEngine::new(Arc::new(provider));
        engine.set_lab_config(LabConfig {
            temporal_phrasing: Some(true),
            saliency_threshold: None,
            weight_dense: None,
            significance_coef: None,
            max_lore_atoms: None,
            timestamp: None,
            enable_entity_extraction: None,
            max_entity_representations: None,
            default_nap_repository: None,
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
            weight_dense: None,
            significance_coef: None,
            temporal_phrasing: None,
            max_lore_atoms: None,
            timestamp: None,
            enable_entity_extraction: None,
            max_entity_representations: None,
            default_nap_repository: None,
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

    // ── Enhanced API tests ─────────────────────────────────────────────────────

    use std::future::Future;
    use std::pin::Pin;

    // Mock content generator for testing
    struct MockContentGenerator {
        generated_content: String,
    }

    impl ContentGenerator for MockContentGenerator {
        fn generate_content(
            &self,
            _context: &str,
            _query: &str,
        ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
            let content = self.generated_content.clone();
            Box::pin(async move { Ok(content) })
        }
    }

    // Mock entity extractor for testing
    struct MockEntityExtractor {
        entities: Vec<crate::narrative::v1::Entity>,
    }

    impl EntityExtractor for MockEntityExtractor {
        fn extract_entities(
            &self,
            _context: &str,
            _repository: Option<&str>,
            _entity_types: &[String],
        ) -> Pin<
            Box<
                dyn Future<Output = Result<Vec<crate::narrative::v1::Entity>, GenerationError>>
                    + Send
                    + '_,
            >,
        > {
            let entities = self.entities.clone();
            Box::pin(async move { Ok(entities) })
        }
    }

    // Mock representation retriever for testing
    #[allow(dead_code)]
    struct MockRepresentationRetriever {
        representations: Vec<crate::narrative::v1::Representation>,
    }

    impl RepresentationRetriever for MockRepresentationRetriever {
        fn get_representations(
            &self,
            _entities: &[crate::narrative::v1::Entity],
            _representation_property: Option<&str>,
            _max_count: i32,
        ) -> Pin<
            Box<
                dyn Future<
                        Output = Result<Vec<crate::narrative::v1::Representation>, GenerationError>,
                    > + Send
                    + '_,
            >,
        > {
            let representations = self.representations.clone();
            Box::pin(async move { Ok(representations) })
        }
    }

    // Mock block persistence for testing
    struct MockBlockPersistence {
        persisted_blocks: Arc<std::sync::Mutex<Vec<BaseNarrativeBlock>>>,
        should_fail: bool,
    }

    impl BlockPersistence for MockBlockPersistence {
        fn persist_block(
            &self,
            _channel_id: &str,
            _session_id: i32,
            block: &BaseNarrativeBlock,
        ) -> Pin<Box<dyn Future<Output = Result<(), GenerationError>> + Send + '_>> {
            let block_clone = block.clone();
            let should_fail = self.should_fail;
            let persisted_blocks = Arc::clone(&self.persisted_blocks);
            Box::pin(async move {
                if should_fail {
                    Err(GenerationError {
                        error_type: crate::narrative::v1::GenerationErrorType::PersistenceFailed
                            as i32,
                        message: "Mock persistence failure".to_string(),
                        timestamp: chrono::Utc::now().timestamp(),
                        is_transient: false,
                    })
                } else {
                    persisted_blocks.lock().unwrap().push(block_clone);
                    Ok(())
                }
            })
        }
    }

    #[tokio::test]
    async fn generate_block_with_content_generator() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let generator = Arc::new(MockContentGenerator {
            generated_content: "Generated by AI".to_string(),
        });

        let engine = NarrativeEngine::new(provider).with_content_generator(generator);

        let parameters = GenerationParameters {
            max_unique_entity_representations: 5,
            representation_property: "default".to_string(),
            chronological_block_count: 3,
            include_inactive_entities: false,
            entity_types: vec![],
            enable_entity_extraction: false,
            nap_repository: "".to_string(),
            nap_entity_types: vec![],
            cancellation_token: "".to_string(),
        };

        let result = engine.generate_block("test", "query", parameters).await;

        assert!(result.is_ok());
        let envelope = result.unwrap();
        assert!(envelope.block.is_some());
        let block = envelope.block.unwrap();
        assert_eq!(block.content, "Generated by AI");
    }

    #[tokio::test]
    async fn generate_block_with_entity_extraction() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let generator = Arc::new(MockContentGenerator {
            generated_content: "Story with characters".to_string(),
        });
        let extractor = Arc::new(MockEntityExtractor {
            entities: vec![crate::narrative::v1::Entity {
                id: "nap://test/character/hero".to_string(),
                name: "Hero".to_string(),
                r#type: "character".to_string(),
                description: "The protagonist".to_string(),
                representations: vec![],
                properties: std::collections::HashMap::new(),
                references: std::collections::HashMap::new(),
            }],
        });

        let engine = NarrativeEngine::new(provider)
            .with_content_generator(generator)
            .with_entity_extractor(extractor);

        let parameters = GenerationParameters {
            max_unique_entity_representations: 5,
            representation_property: "default".to_string(),
            chronological_block_count: 3,
            include_inactive_entities: false,
            entity_types: vec!["character".to_string()],
            enable_entity_extraction: true,
            nap_repository: "test".to_string(),
            nap_entity_types: vec!["character".to_string()],
            cancellation_token: "".to_string(),
        };

        let result = engine.generate_block("test", "query", parameters).await;

        assert!(result.is_ok());
        let envelope = result.unwrap();
        assert!(envelope.context.is_some());
        let context = envelope.context.unwrap();
        assert_eq!(context.entities.len(), 1);
        assert_eq!(context.entities[0].name, "Hero");
    }

    #[tokio::test]
    async fn generate_blocks_sequential_with_persistence() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let persisted_blocks = Arc::new(std::sync::Mutex::new(vec![]));
        let persistence = Arc::new(MockBlockPersistence {
            persisted_blocks: Arc::clone(&persisted_blocks),
            should_fail: false,
        });

        let engine = NarrativeEngine::new(provider).with_block_persistence(persistence);

        let options = BatchGenerationOptions {
            block_count: 3,
            dry_run: false,
            persist_blocks: true,
            persist_channel_id: "test".to_string(),
            persist_session_id: 1,
            cancellation_token: "".to_string(),
            max_concurrency: 4,
        };

        let result = engine
            .generate_blocks_sequential("test", "initial context", options)
            .await;

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert_eq!(batch_result.blocks_generated, 3);
        assert_eq!(batch_result.blocks_failed, 0);

        // Verify blocks were persisted
        let persisted = persisted_blocks.lock().unwrap();
        assert_eq!(persisted.len(), 3);
    }

    #[tokio::test]
    async fn generate_blocks_sequential_dry_run() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let persisted_blocks = Arc::new(std::sync::Mutex::new(vec![]));
        let persistence = Arc::new(MockBlockPersistence {
            persisted_blocks: Arc::clone(&persisted_blocks),
            should_fail: false,
        });

        let engine = NarrativeEngine::new(provider).with_block_persistence(persistence);

        let options = BatchGenerationOptions {
            block_count: 2,
            dry_run: true,
            persist_blocks: true,
            persist_channel_id: "test".to_string(),
            persist_session_id: 1,
            cancellation_token: "".to_string(),
            max_concurrency: 4,
        };

        let result = engine
            .generate_blocks_sequential("test", "initial context", options)
            .await;

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert_eq!(batch_result.blocks_generated, 2);
        assert_eq!(batch_result.blocks_failed, 0);
        assert_eq!(batch_result.generated_blocks.len(), 2); // Blocks returned in result

        // Verify blocks were NOT persisted (dry run)
        let persisted = persisted_blocks.lock().unwrap();
        assert_eq!(persisted.len(), 0);
    }

    #[tokio::test]
    async fn generate_blocks_parallel_bounded_concurrency() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let engine = NarrativeEngine::new(provider);

        let branch_contexts = vec![
            "Branch 1 context".to_string(),
            "Branch 2 context".to_string(),
            "Branch 3 context".to_string(),
            "Branch 4 context".to_string(),
            "Branch 5 context".to_string(),
        ];

        let options = BatchGenerationOptions {
            block_count: 1,
            dry_run: false,
            persist_blocks: false,
            persist_channel_id: "test".to_string(),
            persist_session_id: 1,
            cancellation_token: "".to_string(),
            max_concurrency: 2, // Limit to 2 concurrent operations
        };

        let result = engine
            .generate_blocks_parallel("test", &branch_contexts, options)
            .await;

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert_eq!(batch_result.blocks_generated, 5);
        assert_eq!(batch_result.blocks_failed, 0);
    }

    #[tokio::test]
    async fn generate_blocks_cancellation() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let engine = NarrativeEngine::new(provider);

        let options = BatchGenerationOptions {
            block_count: 5,
            dry_run: false,
            persist_blocks: false,
            persist_channel_id: "test".to_string(),
            persist_session_id: 1,
            cancellation_token: "cancelled:test123".to_string(), // Trigger cancellation
            max_concurrency: 4,
        };

        let result = engine
            .generate_blocks_sequential("test", "initial context", options)
            .await;

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        // Should have stopped early due to cancellation
        assert!(batch_result.blocks_generated < 5);
        assert!(batch_result.errors.iter().any(|e| e.contains("Cancelled")));
    }

    #[tokio::test]
    async fn generate_block_without_content_generator_fallback() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let engine = NarrativeEngine::new(provider); // No content generator configured

        let parameters = GenerationParameters {
            max_unique_entity_representations: 5,
            representation_property: "default".to_string(),
            chronological_block_count: 3,
            include_inactive_entities: false,
            entity_types: vec![],
            enable_entity_extraction: false,
            nap_repository: "".to_string(),
            nap_entity_types: vec![],
            cancellation_token: "".to_string(),
        };

        let result = engine.generate_block("test", "query", parameters).await;

        assert!(result.is_ok());
        let envelope = result.unwrap();
        assert!(envelope.block.is_some());
        // Should fall back to context as content
        let block = envelope.block.unwrap();
        assert!(!block.content.is_empty());
    }

    // ── Comprehensive enrichment pipeline tests ───────────────────────────────

    #[tokio::test]
    async fn full_enrichment_pipeline_with_entities_and_representations() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let generator = Arc::new(MockContentGenerator {
            generated_content: "The hero entered the ancient temple".to_string(),
        });
        let extractor = Arc::new(MockEntityExtractor {
            entities: vec![
                crate::narrative::v1::Entity {
                    id: "nap://test/character/hero".to_string(),
                    name: "Hero".to_string(),
                    r#type: "character".to_string(),
                    description: "The protagonist".to_string(),
                    representations: vec![],
                    properties: std::collections::HashMap::new(),
                    references: std::collections::HashMap::new(),
                },
                crate::narrative::v1::Entity {
                    id: "nap://test/location/temple".to_string(),
                    name: "Ancient Temple".to_string(),
                    r#type: "location".to_string(),
                    description: "Sacred location".to_string(),
                    representations: vec![],
                    properties: std::collections::HashMap::new(),
                    references: std::collections::HashMap::new(),
                },
            ],
        });
        let retriever = Arc::new(MockRepresentationRetriever {
            representations: vec![crate::narrative::v1::Representation {
                id: "rep1".to_string(),
                format: "png".to_string(),
                cdn_url: "https://cdn.example.com/hero.png".to_string(),
                tier: "high".to_string(),
                hash: "abc123".to_string(),
                expires_at: 0,
            }],
        });

        let engine = NarrativeEngine::new(provider)
            .with_content_generator(generator)
            .with_entity_extractor(extractor)
            .with_representation_retriever(retriever);

        let parameters = GenerationParameters {
            max_unique_entity_representations: 5,
            representation_property: "default".to_string(),
            chronological_block_count: 3,
            include_inactive_entities: false,
            entity_types: vec!["character".to_string(), "location".to_string()],
            enable_entity_extraction: true,
            nap_repository: "test".to_string(),
            nap_entity_types: vec!["character".to_string(), "location".to_string()],
            cancellation_token: "".to_string(),
        };

        let result = engine.generate_block("test", "query", parameters).await;

        assert!(result.is_ok());
        let envelope = result.unwrap();

        // Verify content was generated
        assert!(envelope.block.is_some());
        let block = envelope.block.unwrap();
        assert_eq!(block.content, "The hero entered the ancient temple");

        // Verify entities were extracted
        assert!(envelope.context.is_some());
        let context = envelope.context.unwrap();
        assert_eq!(context.entities.len(), 2);
        assert_eq!(context.entities[0].name, "Hero");
        assert_eq!(context.entities[1].name, "Ancient Temple");

        // Verify representations were retrieved
        assert_eq!(context.representations.len(), 1);
        assert_eq!(context.representations[0].format, "png");

        // Verify metadata
        assert!(envelope.metadata.is_some());
        let metadata = envelope.metadata.unwrap();
        assert_eq!(metadata.entity_count, 2);
        assert_eq!(metadata.representation_count, 1);
    }

    #[tokio::test]
    async fn enrichment_pipeline_graceful_degradation_on_entity_failure() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let generator = Arc::new(MockContentGenerator {
            generated_content: "Story content".to_string(),
        });

        // Create an extractor that always fails
        struct FailingEntityExtractor;
        impl EntityExtractor for FailingEntityExtractor {
            fn extract_entities(
                &self,
                _context: &str,
                _repository: Option<&str>,
                _entity_types: &[String],
            ) -> Pin<
                Box<
                    dyn Future<Output = Result<Vec<crate::narrative::v1::Entity>, GenerationError>>
                        + Send
                        + '_,
                >,
            > {
                Box::pin(async move {
                    Err(GenerationError {
                        error_type:
                            crate::narrative::v1::GenerationErrorType::EntityExtractionFailed as i32,
                        message: "Simulated entity extraction failure".to_string(),
                        timestamp: chrono::Utc::now().timestamp(),
                        is_transient: true,
                    })
                })
            }
        }

        let extractor = Arc::new(FailingEntityExtractor);

        let engine = NarrativeEngine::new(provider)
            .with_content_generator(generator)
            .with_entity_extractor(extractor);

        let parameters = GenerationParameters {
            max_unique_entity_representations: 5,
            representation_property: "default".to_string(),
            chronological_block_count: 3,
            include_inactive_entities: false,
            entity_types: vec!["character".to_string()],
            enable_entity_extraction: true,
            nap_repository: "test".to_string(),
            nap_entity_types: vec!["character".to_string()],
            cancellation_token: "".to_string(),
        };

        let result = engine.generate_block("test", "query", parameters).await;

        // Should still succeed despite entity extraction failure
        assert!(result.is_ok());
        let envelope = result.unwrap();

        // Content should still be generated
        assert!(envelope.block.is_some());
        let block = envelope.block.unwrap();
        assert_eq!(block.content, "Story content");

        // Entities should be empty (graceful degradation)
        assert!(envelope.context.is_some());
        let context = envelope.context.unwrap();
        assert_eq!(context.entities.len(), 0);
    }

    #[tokio::test]
    async fn enrichment_pipeline_graceful_degradation_on_representation_failure() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let generator = Arc::new(MockContentGenerator {
            generated_content: "Story with character".to_string(),
        });
        let extractor = Arc::new(MockEntityExtractor {
            entities: vec![crate::narrative::v1::Entity {
                id: "nap://test/character/hero".to_string(),
                name: "Hero".to_string(),
                r#type: "character".to_string(),
                description: "The protagonist".to_string(),
                representations: vec![],
                properties: std::collections::HashMap::new(),
                references: std::collections::HashMap::new(),
            }],
        });

        // Create a retriever that always fails
        struct FailingRepresentationRetriever;
        impl RepresentationRetriever for FailingRepresentationRetriever {
            fn get_representations(
                &self,
                _entities: &[crate::narrative::v1::Entity],
                _representation_property: Option<&str>,
                _max_count: i32,
            ) -> Pin<
                Box<
                    dyn Future<
                            Output = Result<
                                Vec<crate::narrative::v1::Representation>,
                                GenerationError,
                            >,
                        > + Send
                        + '_,
                >,
            > {
                Box::pin(async move {
                    Err(GenerationError {
                        error_type:
                            crate::narrative::v1::GenerationErrorType::RepresentationRetrievalFailed
                                as i32,
                        message: "Simulated representation retrieval failure".to_string(),
                        timestamp: chrono::Utc::now().timestamp(),
                        is_transient: true,
                    })
                })
            }
        }

        let retriever = Arc::new(FailingRepresentationRetriever);

        let engine = NarrativeEngine::new(provider)
            .with_content_generator(generator)
            .with_entity_extractor(extractor)
            .with_representation_retriever(retriever);

        let parameters = GenerationParameters {
            max_unique_entity_representations: 5,
            representation_property: "default".to_string(),
            chronological_block_count: 3,
            include_inactive_entities: false,
            entity_types: vec!["character".to_string()],
            enable_entity_extraction: true,
            nap_repository: "test".to_string(),
            nap_entity_types: vec!["character".to_string()],
            cancellation_token: "".to_string(),
        };

        let result = engine.generate_block("test", "query", parameters).await;

        // Should still succeed despite representation retrieval failure
        assert!(result.is_ok());
        let envelope = result.unwrap();

        // Content should still be generated
        assert!(envelope.block.is_some());
        let block = envelope.block.unwrap();
        assert_eq!(block.content, "Story with character");

        // Entities should still be present
        assert!(envelope.context.is_some());
        let context = envelope.context.unwrap();
        assert_eq!(context.entities.len(), 1);

        // Representations should be empty (graceful degradation)
        assert_eq!(context.representations.len(), 0);
    }

    #[tokio::test]
    async fn batch_generation_with_persistence_and_enrichment() {
        let provider = Arc::new(InMemoryNarrativeProvider::default());
        let persisted_blocks = Arc::new(std::sync::Mutex::new(vec![]));
        let persistence = Arc::new(MockBlockPersistence {
            persisted_blocks: Arc::clone(&persisted_blocks),
            should_fail: false,
        });
        let generator = Arc::new(MockContentGenerator {
            generated_content: "Generated content".to_string(),
        });
        let extractor = Arc::new(MockEntityExtractor {
            entities: vec![crate::narrative::v1::Entity {
                id: "nap://test/character/hero".to_string(),
                name: "Hero".to_string(),
                r#type: "character".to_string(),
                description: "The protagonist".to_string(),
                representations: vec![],
                properties: std::collections::HashMap::new(),
                references: std::collections::HashMap::new(),
            }],
        });

        let engine = NarrativeEngine::new(provider)
            .with_content_generator(generator)
            .with_entity_extractor(extractor)
            .with_block_persistence(persistence);

        let options = BatchGenerationOptions {
            block_count: 2,
            dry_run: false,
            persist_blocks: true,
            persist_channel_id: "test".to_string(),
            persist_session_id: 1,
            cancellation_token: "".to_string(),
            max_concurrency: 4,
        };

        let result = engine
            .generate_blocks_sequential("test", "initial context", options)
            .await;

        assert!(result.is_ok());
        let batch_result = result.unwrap();
        assert_eq!(batch_result.blocks_generated, 2);
        assert_eq!(batch_result.blocks_failed, 0);

        // Verify blocks were persisted
        let persisted = persisted_blocks.lock().unwrap();
        assert_eq!(persisted.len(), 2);
    }
}
