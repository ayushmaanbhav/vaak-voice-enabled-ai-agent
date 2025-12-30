//! RAG (Retrieval-Augmented Generation) with hybrid search
//!
//! Features:
//! - Dense vector search via Qdrant
//! - Sparse BM25 search via Tantivy
//! - Hybrid fusion with RRF
//! - Early-exit cross-encoder reranking
//! - Candle BERT embeddings (native Rust inference)
//! - FP16/BF16 quantization for reduced memory and faster inference
//! - LRU embedding cache for repeated queries
//! - Agentic RAG multi-step retrieval flow
//! - Query expansion with domain synonyms
//! - Domain-specific term boosting
//! - Cross-lingual query normalization (Hindi/Hinglish/English)
//! - Core Retriever trait implementation

pub mod adapter;
pub mod agentic;
pub mod cache;
pub mod candle_embeddings;
pub mod cross_lingual;
pub mod domain_boost;
pub mod embeddings;
pub mod query_expansion;
pub mod reranker;
pub mod retriever;
pub mod sparse_search;
pub mod vector_store;
// P1 FIX: Context sizing by conversation stage
pub mod context;
// P2 FIX: Knowledge base loading
pub mod knowledge_loader;
// P2-2 FIX: Context compression for long conversations
pub mod compressor;

pub use adapter::{EnhancedRetriever, EnhancedRetrieverConfig};
pub use agentic::{
    AgenticRagConfig,
    AgenticRetriever,
    AgenticSearchResult,
    // P2-1 FIX: QueryContext is the new name, ConversationContext kept for backwards compat
    QueryContext,
    // P1 FIX: LLM-based sufficiency checking
    LlmSufficiencyChecker,
    LlmSufficiencyConfig,
    QueryRewriter,
    SufficiencyChecker,
    SufficiencyEvaluation,
};
// P2-1 FIX: Re-export deprecated alias separately to avoid warnings at use sites
#[allow(deprecated)]
pub use agentic::ConversationContext;
pub use cache::{CacheStats, CachedEmbedder, EmbeddingCache};
pub use candle_embeddings::{
    CandleBertEmbedder, CandleEmbeddingConfig, PoolingStrategy, QuantizationMode, UnifiedEmbedder,
};
pub use context::{context_budget_for_stage, ContextBudget, ContextConfig, ContextManager, Stage};
pub use cross_lingual::{
    CrossLingualNormalizer, DetectedScript, LanguageDetection, NormalizedQuery,
};
pub use domain_boost::{
    BoostResult, DomainBoostConfig, DomainBooster, DomainTerm, MatchedTerm, QueryIntent,
    TermCategory,
};
pub use embeddings::{Embedder, EmbeddingConfig, SimpleEmbedder};
pub use knowledge_loader::{KnowledgeDocument, KnowledgeFile, KnowledgeLoader};
pub use query_expansion::{
    ExpandedQuery, ExpansionStats, QueryExpander, QueryExpansionConfig, TermSource, WeightedTerm,
};
pub use reranker::{EarlyExitReranker, ExitStrategy, RerankerConfig};
pub use retriever::{HybridRetriever, RetrieverConfig, SearchResult};
pub use sparse_search::{SparseConfig, SparseIndex};
pub use vector_store::{VectorDistance, VectorStore, VectorStoreConfig};
// P2-2 FIX: Context compression exports
pub use compressor::{
    CompressedContext, CompressorConfig, ContextCompressor, RuleBasedSummarizer, Summarizer, Turn,
};

use thiserror::Error;

/// RAG errors
#[derive(Error, Debug)]
pub enum RagError {
    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Vector store error: {0}")]
    VectorStore(String),

    #[error("Search error: {0}")]
    Search(String),

    #[error("Reranker error: {0}")]
    Reranker(String),

    #[error("Model error: {0}")]
    Model(String),

    #[error("Index error: {0}")]
    Index(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Connection error: {0}")]
    Connection(String),
}

impl From<RagError> for voice_agent_core::Error {
    fn from(err: RagError) -> Self {
        voice_agent_core::Error::Rag(err.to_string())
    }
}
