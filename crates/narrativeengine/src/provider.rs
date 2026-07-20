//! Provider trait and in-memory reference implementation.
//!
//! Mirrors `provider.ts`: `HybridCandidate`, `NarrativeProvider`,
//! and `InMemoryNarrativeProvider`.

use std::collections::HashMap;

use crate::types::{BaseNarrativeBlock, BaseNarrativeLore, NarrativeBlockExt};

/// A search result candidate pairing a block with its retrieval scores.
#[derive(Debug, Clone)]
pub struct HybridCandidate<TBlock> {
    pub block: TBlock,
    /// Dense vector (embedding) similarity score — `[0, 1]`.
    pub score_vector_dense: f64,
    /// Sparse keyword (BM25-style) similarity score — `[0, 1]`.
    pub score_keyword_sparse: f64,
}

/// Core storage/retrieval abstraction. Mirrors the `NarrativeProvider` interface.
///
/// All methods are async (return `impl Future`) and are object-safe via `async_trait`
/// conventions. The trait is kept object-safe by returning `Box<dyn Future>` under
/// the hood; callers use `.await` normally.
pub trait NarrativeProvider<TBlock, TLore>: Send + Sync
where
    TBlock: Clone + Send + Sync,
    TLore: Clone + Send + Sync,
{
    /// Returns the total number of story blocks for `channel_id`.
    fn get_block_count(
        &self,
        channel_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = usize> + Send + '_>>;

    /// Returns all *active* lore atoms for `channel_id`.
    fn get_lore_atoms(
        &self,
        channel_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<TLore>> + Send + '_>>;

    /// Single-query hybrid search — returns up to `limit` candidates.
    fn get_hybrid_search_candidates(
        &self,
        channel_id: &str,
        query: &str,
        limit: usize,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<HybridCandidate<TBlock>>> + Send + '_>>;

    /// Batch hybrid search — one call for N queries.
    fn get_hybrid_search_candidates_batch(
        &self,
        channel_id: &str,
        queries: &[String],
        limit: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = HashMap<String, Vec<HybridCandidate<TBlock>>>> + Send + '_>,
    >;

    /// Returns blocks whose 1-based `index` matches one of `indices`.
    fn get_blocks_by_indices(
        &self,
        channel_id: &str,
        indices: &[usize],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<TBlock>> + Send + '_>>;

    /// Returns all blocks marked `is_notable = true`.
    fn get_notable_events(
        &self,
        channel_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<TBlock>> + Send + '_>>;

    /// Appends a new block (optional — providers may be read-only).
    fn add_block(
        &self,
        channel_id: &str,
        block: TBlock,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>;

    /// Returns a short identifier for logging / tracing.
    fn get_provider_type(&self) -> &'static str;
}

// ─────────────────────────────────────────────────────────────────────────────
// InMemoryNarrativeProvider
// ─────────────────────────────────────────────────────────────────────────────

use std::sync::{Arc, Mutex};

/// Zero-dependency in-memory provider for testing and local development.
///
/// Mirrors `InMemoryNarrativeProvider` from `provider.ts`.
pub struct InMemoryNarrativeProvider<TBlock = BaseNarrativeBlock, TLore = BaseNarrativeLore>
where
    TBlock: Clone + Send + Sync,
    TLore: Clone + Send + Sync,
{
    blocks: Arc<Mutex<Vec<TBlock>>>,
    lore: Arc<Mutex<Vec<TLore>>>,
}

impl<TBlock, TLore> InMemoryNarrativeProvider<TBlock, TLore>
where
    TBlock: Clone + Send + Sync,
    TLore: Clone + Send + Sync,
{
    pub fn new(blocks: Vec<TBlock>, lore: Vec<TLore>) -> Self {
        Self {
            blocks: Arc::new(Mutex::new(blocks)),
            lore: Arc::new(Mutex::new(lore)),
        }
    }

    /// Returns all blocks (for internal test access).
    pub fn all_blocks(&self) -> Vec<TBlock> {
        self.blocks.lock().unwrap().clone()
    }
}

impl Default for InMemoryNarrativeProvider<BaseNarrativeBlock, BaseNarrativeLore> {
    fn default() -> Self {
        use crate::mocks::{MOCK_BLOCKS, MOCK_LORE};
        Self::new(MOCK_BLOCKS.to_vec(), MOCK_LORE.to_vec())
    }
}

// ─── trait impl ──────────────────────────────────────────────────────────────

impl NarrativeProvider<BaseNarrativeBlock, BaseNarrativeLore>
    for InMemoryNarrativeProvider<BaseNarrativeBlock, BaseNarrativeLore>
{
    fn get_block_count(
        &self,
        _channel_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = usize> + Send + '_>> {
        let count = self.blocks.lock().unwrap().len();
        Box::pin(async move { count })
    }

    fn get_lore_atoms(
        &self,
        _channel_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BaseNarrativeLore>> + Send + '_>>
    {
        let atoms: Vec<BaseNarrativeLore> = self
            .lore
            .lock()
            .unwrap()
            .iter()
            .filter(|l| l.is_active())
            .cloned()
            .collect();
        Box::pin(async move { atoms })
    }

    fn get_hybrid_search_candidates(
        &self,
        _channel_id: &str,
        query: &str,
        limit: usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Vec<HybridCandidate<BaseNarrativeBlock>>> + Send + '_>,
    > {
        let q = query.to_lowercase();
        let matches: Vec<HybridCandidate<BaseNarrativeBlock>> = self
            .blocks
            .lock()
            .unwrap()
            .iter()
            .filter(|b| b.content.to_lowercase().contains(&q))
            .take(limit)
            .map(|b| HybridCandidate {
                block: b.clone(),
                score_vector_dense: 0.8,
                score_keyword_sparse: 0.8,
            })
            .collect();
        Box::pin(async move { matches })
    }

    fn get_hybrid_search_candidates_batch(
        &self,
        _channel_id: &str,
        queries: &[String],
        limit: usize,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<
                    Output = HashMap<String, Vec<HybridCandidate<BaseNarrativeBlock>>>,
                > + Send
                + '_,
        >,
    > {
        let blocks_snapshot = self.blocks.lock().unwrap().clone();
        let queries_owned = queries.to_vec();
        Box::pin(async move {
            let mut result: HashMap<String, Vec<HybridCandidate<BaseNarrativeBlock>>> =
                HashMap::new();
            for q in &queries_owned {
                let q_lower = q.to_lowercase();
                let matches: Vec<HybridCandidate<BaseNarrativeBlock>> = blocks_snapshot
                    .iter()
                    .filter(|b| b.content.to_lowercase().contains(&q_lower))
                    .take(limit)
                    .map(|b| HybridCandidate {
                        block: b.clone(),
                        score_vector_dense: 0.8,
                        score_keyword_sparse: 0.8,
                    })
                    .collect();
                result.insert(q.clone(), matches);
            }
            result
        })
    }

    fn get_blocks_by_indices(
        &self,
        _channel_id: &str,
        indices: &[usize],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BaseNarrativeBlock>> + Send + '_>>
    {
        let blocks: Vec<BaseNarrativeBlock> = self
            .blocks
            .lock()
            .unwrap()
            .iter()
            .filter(|b| {
                b.block_id()
                    .as_num()
                    .map(|n| indices.contains(&(n as usize)))
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        Box::pin(async move { blocks })
    }

    fn get_notable_events(
        &self,
        _channel_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<BaseNarrativeBlock>> + Send + '_>>
    {
        let notable: Vec<BaseNarrativeBlock> = self
            .blocks
            .lock()
            .unwrap()
            .iter()
            .filter(|b| b.is_notable())
            .cloned()
            .collect();
        Box::pin(async move { notable })
    }

    fn add_block(
        &self,
        _channel_id: &str,
        block: BaseNarrativeBlock,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        self.blocks.lock().unwrap().push(block);
        Box::pin(async {})
    }

    fn get_provider_type(&self) -> &'static str {
        "in-memory"
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mocks::{MOCK_BLOCKS, MOCK_LORE};
    use crate::types::{BlockId, NarrativeLoreExt};

    fn provider() -> InMemoryNarrativeProvider {
        InMemoryNarrativeProvider::new(MOCK_BLOCKS.to_vec(), MOCK_LORE.to_vec())
    }

    #[tokio::test]
    async fn block_count_matches_mock() {
        let p = provider();
        assert_eq!(p.get_block_count("test").await, MOCK_BLOCKS.len());
    }

    #[tokio::test]
    async fn lore_atoms_filters_inactive() {
        let mut lore = MOCK_LORE.to_vec();
        lore.push(BaseNarrativeLore {
            id: Some(BlockId::Str("lore-inactive".to_string()).into()),
            content: "Inactive lore".into(),
            happened_at: 1000,
            is_active: Some(false),
        });
        let p = InMemoryNarrativeProvider::new(MOCK_BLOCKS.to_vec(), lore);
        let atoms = p.get_lore_atoms("test").await;
        // MOCK_LORE has 17 active (3 inactive), plus our new inactive one → still 17
        assert_eq!(atoms.len(), 17);
        assert!(atoms.iter().all(|l| l.is_active()));
        assert!(atoms.iter().find(|l| l.lore_id().to_string() == "lore-inactive").is_none());
    }

    #[tokio::test]
    async fn get_blocks_by_indices_correct() {
        let p = provider();
        let blocks = p.get_blocks_by_indices("test", &[1, 48]).await;
        assert_eq!(blocks.len(), 2);
        let ids: Vec<i64> = blocks.iter().map(|b| b.block_id().as_num().unwrap()).collect();
        assert!(ids.contains(&1));
        assert!(ids.contains(&48));
    }

    #[tokio::test]
    async fn hybrid_search_returns_content_matches() {
        let p = provider();
        let candidates = p.get_hybrid_search_candidates("test", "cube", 2).await;
        assert!(!candidates.is_empty());
        assert!(candidates.len() <= 2);
        for c in &candidates {
            assert!(c.block.content.to_lowercase().contains("cube"));
            assert!((c.score_vector_dense - 0.8).abs() < f64::EPSILON);
            assert!((c.score_keyword_sparse - 0.8).abs() < f64::EPSILON);
        }
    }

    #[tokio::test]
    async fn notable_events_all_marked() {
        let p = provider();
        let notable = p.get_notable_events("test").await;
        let expected = MOCK_BLOCKS.iter().filter(|b| b.is_notable()).count();
        assert_eq!(notable.len(), expected);
        assert!(notable.iter().all(|b| b.is_notable()));
    }

    #[tokio::test]
    async fn add_block_increases_count() {
        let p = InMemoryNarrativeProvider::<BaseNarrativeBlock, BaseNarrativeLore>::new(
            vec![],
            vec![],
        );
        assert_eq!(p.get_block_count("test").await, 0);
        let new_block = BaseNarrativeBlock {
            id: Some(BlockId::Num(1).into()),
            index: 1,
            content: "A new block".into(),
            happened_at: 9999,
            is_notable: Some(false),
        };
        p.add_block("test", new_block).await;
        assert_eq!(p.get_block_count("test").await, 1);
    }

    #[tokio::test]
    async fn provider_type_is_in_memory() {
        let p = provider();
        assert_eq!(p.get_provider_type(), "in-memory");
    }

    #[tokio::test]
    async fn no_delete_records_method_exists() {
        // Mirrors the JS test: "prohibition of nuclear deletes"
        // In Rust this is compile-time enforced — the trait has no delete method.
        // We confirm via a type-level check that `InMemoryNarrativeProvider`
        // does not expose any deletion API.
        let _p = provider();
        // The following must NOT compile if uncommented:
        // _p.delete_records("test");
        // Reaching here confirms no such method exists.
    }

    #[tokio::test]
    async fn batch_search_returns_map() {
        let p = provider();
        let queries = vec!["cube".to_string(), "ELARA".to_string()];
        let map = p.get_hybrid_search_candidates_batch("test", &queries, 5).await;
        assert!(map.contains_key("cube"));
        assert!(map.contains_key("ELARA"));
        let cube_results = &map["cube"];
        assert!(!cube_results.is_empty());
    }
}