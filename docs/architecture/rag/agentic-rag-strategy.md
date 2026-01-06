# Agentic RAG Strategy

> Multi-step retrieval with query rewriting, sufficiency checking, and stage-aware context
>
> **Design Goal:** Accurate knowledge retrieval within latency budget (<100ms for simple, <400ms for complex)

---

## Table of Contents

1. [Overview](#overview)
2. [Why Agentic RAG?](#why-agentic-rag)
3. [Architecture](#architecture)
4. [Timing Strategies](#timing-strategies)
5. [Retrieval Pipeline](#retrieval-pipeline)
6. [Knowledge Base Design](#knowledge-base-design)
7. [Context Management](#context-management)
8. [Implementation](#implementation)
9. [Configuration](#configuration)
10. [Metrics & Optimization](#metrics--optimization)

---

## Overview

### What is Agentic RAG?

Traditional RAG:
```
Query → Single Retrieval → Context → LLM → Response
```

Agentic RAG:
```
Query → Intent Classification → Multi-Step Retrieval → Sufficiency Check
                                       ↓ (if insufficient)
                              Query Rewriting → Re-Retrieve
                                       ↓
                              Reranking → Context Sizing → LLM → Response
```

### Key Capabilities

| Capability | Description | Benefit |
|------------|-------------|---------|
| **Intent Classification** | Classify query type before retrieval | Target correct doc types |
| **Multi-Step Retrieval** | Iterate until sufficient context | Better coverage |
| **Query Rewriting** | Reformulate if initial results poor | Handle vague queries |
| **Hybrid Search** | Semantic + BM25 fusion | Best of both approaches |
| **Reranking** | Cross-encoder scoring | Precision over recall |
| **Stage-Aware Context** | Adjust context size by conversation stage | Token efficiency |

---

## Why Agentic RAG?

### Research Evidence

> "Traditional RAG systems rely on one-shot retrieval, which limits their ability to adapt during complex, multistep reasoning tasks."
> — [NVIDIA Technical Blog](https://developer.nvidia.com/blog/traditional-rag-vs-agentic-rag-why-ai-agents-need-dynamic-knowledge-to-get-smarter/)

> "An industry study found 87% of enterprise RAG deployments fail to meet expected ROI, often due to challenges like fixed retrieval logic."
> — [MarkTechPost](https://www.marktechpost.com/2025/08/22/native-rag-vs-agentic-rag-which-approach-advances-enterprise-ai-decision-making/)

### Voice Agent Specific Needs

| Challenge | Traditional RAG Problem | Agentic RAG Solution |
|-----------|------------------------|----------------------|
| Vague queries | "Tell me about rates" → wrong docs | Query rewriting with context |
| Multi-topic | User asks about 2 things | Multi-step retrieval |
| Follow-ups | "What about that?" | Context-aware query expansion |
| Objections | Need specific rebuttals | Intent-targeted retrieval |

### Trade-off: Latency vs Accuracy

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    LATENCY vs ACCURACY TRADE-OFF                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  Approach              Latency      Accuracy     Use Case               │
│  ─────────────────────────────────────────────────────────────────────  │
│  Simple Semantic       ~30ms        Medium       FAQ, simple queries    │
│  Hybrid (Sem + BM25)   ~50ms        Good         Most queries           │
│  Agentic (1 iter)      ~100ms       Better       Complex queries        │
│  Agentic (2-3 iter)    ~200-400ms   Best         Objections, edge cases │
│                                                                         │
│  STRATEGY: Use simple for most, agentic for detected complex queries   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Architecture

### High-Level Flow

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           AGENTIC RAG ARCHITECTURE                            │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────────┐                                                             │
│  │   Query     │                                                             │
│  │   Input     │                                                             │
│  └──────┬──────┘                                                             │
│         │                                                                    │
│         ▼                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    INTENT CLASSIFICATION                             │    │
│  │                                                                      │    │
│  │  • FAQ           → faqs.yaml                                        │    │
│  │  • Product       → products.yaml, rates.yaml                        │    │
│  │  • Competitor    → competitors.yaml, comparison.yaml                │    │
│  │  • Objection     → objections.yaml                                  │    │
│  │  • Process       → process.yaml, eligibility.yaml                   │    │
│  │  • Regulation    → regulations.yaml                                 │    │
│  │  • Generic       → all sources                                      │    │
│  └──────┬──────────────────────────────────────────────────────────────┘    │
│         │                                                                    │
│         ▼                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    HYBRID RETRIEVAL                                  │    │
│  │                                                                      │    │
│  │  ┌─────────────┐              ┌─────────────┐                       │    │
│  │  │  Semantic   │              │    BM25     │                       │    │
│  │  │  (Qdrant)   │              │  (Tantivy)  │                       │    │
│  │  └──────┬──────┘              └──────┬──────┘                       │    │
│  │         │                            │                               │    │
│  │         └───────────┬────────────────┘                               │    │
│  │                     ▼                                                │    │
│  │              ┌─────────────┐                                         │    │
│  │              │   Fusion    │  (Reciprocal Rank Fusion)               │    │
│  │              │   RRF/RFF   │                                         │    │
│  │              └──────┬──────┘                                         │    │
│  └─────────────────────┼───────────────────────────────────────────────┘    │
│                        │                                                     │
│                        ▼                                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    SUFFICIENCY CHECK                                 │    │
│  │                                                                      │    │
│  │  LLM evaluates: "Can these documents answer the query?"              │    │
│  │                                                                      │    │
│  │  IF score < threshold:                                               │    │
│  │    → Rewrite query with more specific terms                          │    │
│  │    → Include conversation context                                    │    │
│  │    → Retry retrieval (max 3 iterations)                              │    │
│  └──────┬──────────────────────────────────────────────────────────────┘    │
│         │                                                                    │
│         ▼                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    RERANKING                                         │    │
│  │                                                                      │    │
│  │  Cross-encoder model scores each doc against query                   │    │
│  │  Reorder by relevance score                                          │    │
│  │  Select top-K documents                                              │    │
│  └──────┬──────────────────────────────────────────────────────────────┘    │
│         │                                                                    │
│         ▼                                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                    CONTEXT SIZING                                    │    │
│  │                                                                      │    │
│  │  Stage: GREETING    → 200 tokens   (minimal context)                │    │
│  │  Stage: DISCOVERY   → 800 tokens   (exploration)                    │    │
│  │  Stage: PITCH       → 2000 tokens  (comprehensive)                  │    │
│  │  Stage: OBJECTION   → 1500 tokens  (targeted rebuttals)             │    │
│  │  Stage: CLOSING     → 500 tokens   (concise)                        │    │
│  └──────┬──────────────────────────────────────────────────────────────┘    │
│         │                                                                    │
│         ▼                                                                    │
│  ┌─────────────┐                                                             │
│  │   Context   │                                                             │
│  │   Output    │                                                             │
│  └─────────────┘                                                             │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Timing Strategies

Three configurable timing modes to balance latency and accuracy:

### 1. Sequential (Default)

```
User speaks → STT → [RAG retrieval] → LLM → TTS
                         │
                    Blocking
                    ~50-400ms
```

**Pros:** Simple, predictable
**Cons:** Adds full RAG latency to response time
**Use when:** Latency budget allows, accuracy critical

### 2. Prefetch Async

```
User starts speaking → [Start RAG with partial transcript]
User finishes → STT final → [RAG results ready] → LLM → TTS
                                    │
                              Often 0ms wait
                              (completed during speech)
```

**Pros:** Near-zero RAG latency perceived
**Cons:** May retrieve wrong docs if speech changes
**Use when:** Latency critical, queries predictable

### 3. Parallel Inject

```
User speaks → STT → LLM starts generating
                        │
                   [RAG in parallel]
                        │
                   Inject context mid-stream
                        │
                   LLM continues → TTS
```

**Pros:** Lowest latency, context arrives during generation
**Cons:** Complex, may cause inconsistency
**Use when:** Experimental, advanced scenarios

### Configuration

```toml
# domains/gold_loan/rag.toml

[timing]
# Default mode
default = "sequential"  # "sequential" | "prefetch_async" | "parallel_inject"

# Mode selection by query complexity
[timing.adaptive]
enabled = true
simple_query_mode = "sequential"      # Fast for simple queries
complex_query_mode = "prefetch_async" # Prepare for complex

# Complexity detection
[timing.complexity]
# Queries with these keywords are "complex"
complex_keywords = ["compare", "vs", "difference", "why", "how much"]
# Questions are complex
question_is_complex = true
# Short queries are simple
max_simple_length = 20
```

---

## Retrieval Pipeline

### Step 1: Intent Classification

Use a fast classifier to route queries to relevant doc types:

```rust
#[derive(Debug, Clone, Copy)]
pub enum QueryIntent {
    FAQ,              // General questions
    Product,          // Product details, features
    Competitor,       // Competitor comparisons
    Objection,        // Handling objections
    Process,          // How-to, eligibility
    Rate,             // Interest rates, fees
    Regulation,       // Compliance, RBI rules
    Generic,          // Unclear, search all
}

impl QueryIntent {
    /// Document types to search for this intent
    pub fn doc_types(&self) -> Vec<DocumentType> {
        match self {
            QueryIntent::FAQ => vec![DocumentType::FAQ],
            QueryIntent::Product => vec![DocumentType::Product, DocumentType::Rate],
            QueryIntent::Competitor => vec![DocumentType::Competitor, DocumentType::Comparison],
            QueryIntent::Objection => vec![DocumentType::Objection],
            QueryIntent::Process => vec![DocumentType::Process, DocumentType::Eligibility],
            QueryIntent::Rate => vec![DocumentType::Rate, DocumentType::Product],
            QueryIntent::Regulation => vec![DocumentType::Regulation],
            QueryIntent::Generic => vec![], // Search all
        }
    }
}

/// Fast intent classification
pub async fn classify_intent(query: &str, context: &ConversationContext) -> QueryIntent {
    // Rule-based first (fast)
    if query.contains("rate") || query.contains("interest") || query.contains("byaaj") {
        return QueryIntent::Rate;
    }
    if query.contains("Muthoot") || query.contains("Manappuram") || query.contains("IIFL") {
        return QueryIntent::Competitor;
    }
    // ... more rules

    // LLM-based for ambiguous (slower, more accurate)
    // Use small model with constrained output
    QueryIntent::Generic
}
```

### Step 2: Hybrid Retrieval

Combine semantic and keyword search:

```rust
pub struct HybridRetriever {
    semantic: QdrantClient,
    keyword: TantivyIndex,
    embedder: Arc<dyn Embedder>,
}

impl HybridRetriever {
    pub async fn retrieve(
        &self,
        query: &str,
        options: &RetrieveOptions,
    ) -> Result<Vec<Document>> {
        // Generate embedding
        let embedding = self.embedder.embed(query).await?;

        // Parallel retrieval
        let (semantic_results, keyword_results) = tokio::join!(
            self.semantic_search(&embedding, options),
            self.keyword_search(query, options),
        );

        // Reciprocal Rank Fusion
        let fused = self.fuse_results(
            semantic_results?,
            keyword_results?,
            options.semantic_weight,
        );

        Ok(fused)
    }

    /// Reciprocal Rank Fusion (RRF)
    fn fuse_results(
        &self,
        semantic: Vec<Document>,
        keyword: Vec<Document>,
        semantic_weight: f32,
    ) -> Vec<Document> {
        let k = 60.0; // RRF constant

        let mut scores: HashMap<String, f32> = HashMap::new();
        let mut docs: HashMap<String, Document> = HashMap::new();

        // Score semantic results
        for (rank, doc) in semantic.into_iter().enumerate() {
            let score = semantic_weight / (k + rank as f32 + 1.0);
            *scores.entry(doc.id.clone()).or_insert(0.0) += score;
            docs.insert(doc.id.clone(), doc);
        }

        // Score keyword results
        for (rank, doc) in keyword.into_iter().enumerate() {
            let score = (1.0 - semantic_weight) / (k + rank as f32 + 1.0);
            *scores.entry(doc.id.clone()).or_insert(0.0) += score;
            docs.entry(doc.id.clone()).or_insert(doc);
        }

        // Sort by combined score
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        results
            .into_iter()
            .filter_map(|(id, score)| {
                docs.remove(&id).map(|mut doc| {
                    doc.score = score;
                    doc
                })
            })
            .collect()
    }
}
```

### Step 3: Sufficiency Check

Evaluate if retrieved docs can answer the query:

```rust
pub struct SufficiencyChecker {
    llm: Arc<dyn LanguageModel>,
}

impl SufficiencyChecker {
    pub async fn check(
        &self,
        query: &str,
        documents: &[Document],
    ) -> Result<SufficiencyResult> {
        let doc_summary = documents
            .iter()
            .map(|d| format!("- {}", d.content.chars().take(200).collect::<String>()))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(r#"
You are evaluating if retrieved documents can answer a user query.

QUERY: {query}

RETRIEVED DOCUMENTS:
{doc_summary}

EVALUATE:
1. Can these documents fully answer the query? (yes/partial/no)
2. What information is missing? (brief)
3. Suggested query refinement if needed

OUTPUT FORMAT (JSON):
{{"sufficient": true/false, "coverage": 0.0-1.0, "missing": "...", "refined_query": "..."}}
"#);

        let response = self.llm.generate(GenerateRequest {
            prompt,
            max_tokens: 100,
            temperature: 0.1,
            ..Default::default()
        }).await?;

        // Parse JSON response
        let result: SufficiencyResult = serde_json::from_str(&response.text)?;
        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
pub struct SufficiencyResult {
    pub sufficient: bool,
    pub coverage: f32,
    pub missing: String,
    pub refined_query: Option<String>,
}
```

### Step 4: Query Rewriting

Reformulate query for better retrieval:

```rust
pub async fn rewrite_query(
    original: &str,
    documents: &[Document],
    context: &ConversationContext,
    llm: &dyn LanguageModel,
) -> Result<String> {
    let history = context.history
        .iter()
        .take(3)
        .map(|t| format!("{}: {}", t.role, t.content))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(r#"
Rewrite the query to be more specific for document retrieval.

ORIGINAL QUERY: {original}

CONVERSATION CONTEXT:
{history}

ALREADY RETRIEVED (not helpful):
{docs}

TASK: Write a more specific query that will find relevant documents.
Focus on specific terms, product names, or exact questions.

REWRITTEN QUERY:"#,
        docs = documents.iter().map(|d| &d.content).take(2).collect::<Vec<_>>().join("\n")
    );

    let response = llm.generate(GenerateRequest {
        prompt,
        max_tokens: 50,
        temperature: 0.3,
        ..Default::default()
    }).await?;

    Ok(response.text.trim().to_string())
}
```

### Step 5: Reranking

Use cross-encoder for precision:

```rust
pub struct CrossEncoderReranker {
    model: ort::Session,
    tokenizer: Tokenizer,
}

impl CrossEncoderReranker {
    pub async fn rerank(
        &self,
        query: &str,
        documents: Vec<Document>,
        top_k: usize,
    ) -> Result<Vec<Document>> {
        let mut scored_docs: Vec<(Document, f32)> = Vec::new();

        for doc in documents {
            // Tokenize query-document pair
            let encoding = self.tokenizer.encode(
                (query, &doc.content),
                true, // Add special tokens
            )?;

            // Run cross-encoder
            let inputs = vec![
                ort::Value::from_array(encoding.get_ids())?,
                ort::Value::from_array(encoding.get_attention_mask())?,
            ];
            let outputs = self.model.run(inputs)?;

            // Extract score
            let score: f32 = outputs[0].try_extract()?[0];
            scored_docs.push((doc, score));
        }

        // Sort by score descending
        scored_docs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return top-k
        Ok(scored_docs
            .into_iter()
            .take(top_k)
            .map(|(mut doc, score)| {
                doc.score = score;
                doc
            })
            .collect())
    }
}
```

---

## Knowledge Base Design

### Document Types

```yaml
# domains/gold_loan/knowledge/products.yaml
- id: prod_001
  type: product
  title: "Kotak Gold Loan - Standard"
  content: |
    Kotak Gold Loan offers:
    - Interest rate: 10.5% - 17% p.a.
    - LTV: Up to 75% of gold value
    - Tenure: 3 months to 36 months
    - Processing fee: 1% of loan amount
    - Minimum loan: ₹25,000
    - Maximum loan: ₹2 crore
  keywords: ["gold loan", "interest rate", "LTV", "tenure"]
  segment_relevance: ["P1", "P2", "P3", "P4"]

# domains/gold_loan/knowledge/competitors.yaml
- id: comp_001
  type: competitor
  title: "Muthoot Finance Comparison"
  content: |
    Muthoot vs Kotak Gold Loan:

    | Aspect | Muthoot | Kotak |
    |--------|---------|-------|
    | Rate | 12% - 24% | 10.5% - 17% |
    | LTV | 75% | 75% |
    | Safety | Standard | Bank-grade vaults |
    | Trust | NBFC | Scheduled Bank |

    Key Kotak Advantages:
    - Lower interest rates (save up to 7%)
    - RBI-regulated bank (higher trust)
    - Modern vaults with insurance
  keywords: ["Muthoot", "comparison", "rate", "safety", "NBFC"]
  segment_relevance: ["P1", "P2"]

# domains/gold_loan/knowledge/objections.yaml
- id: obj_001
  type: objection
  objection_type: "rate"
  title: "Interest Rate Objection"
  trigger_phrases: ["rate is high", "too expensive", "cheaper elsewhere"]
  content: |
    Response to "Your rate is high":

    I understand rate is important. Let me show you the complete picture:

    1. Our rate of {rate}% may seem similar, but consider:
       - No hidden charges
       - Transparent processing fee
       - No prepayment penalty

    2. Total cost comparison (for ₹1 lakh over 12 months):
       - Muthoot: ₹{muthoot_total}
       - Kotak: ₹{kotak_total}
       - You save: ₹{savings}

    Would you like me to calculate the exact savings for your loan amount?
  keywords: ["rate", "interest", "expensive", "costly"]
  segment_relevance: ["P1", "P4"]
```

### Embedding Strategy

```rust
pub struct KnowledgeIndexer {
    embedder: Arc<dyn Embedder>,
    qdrant: QdrantClient,
    tantivy: TantivyIndex,
}

impl KnowledgeIndexer {
    pub async fn index_document(&self, doc: &KnowledgeDocument) -> Result<()> {
        // 1. Generate embedding
        let embedding = self.embedder.embed(&doc.content).await?;

        // 2. Store in Qdrant (vector search)
        self.qdrant.upsert_points(
            "knowledge",
            vec![PointStruct::new(
                doc.id.clone(),
                embedding,
                json!({
                    "type": doc.doc_type,
                    "title": doc.title,
                    "keywords": doc.keywords,
                    "segments": doc.segment_relevance,
                }),
            )],
            None,
        ).await?;

        // 3. Store in Tantivy (keyword search)
        let mut writer = self.tantivy.index_writer(50_000_000)?;
        writer.add_document(doc!(
            self.tantivy.schema.get_field("id")? => doc.id.clone(),
            self.tantivy.schema.get_field("content")? => doc.content.clone(),
            self.tantivy.schema.get_field("keywords")? => doc.keywords.join(" "),
        ))?;
        writer.commit()?;

        Ok(())
    }
}
```

---

## Context Management

### Stage-Aware Context Sizing

```rust
/// Get context budget based on conversation stage
pub fn context_budget(state: &ConversationState) -> ContextBudget {
    match state {
        ConversationState::Greeting => ContextBudget {
            max_tokens: 200,
            max_documents: 1,
            max_history_turns: 0,
            strategy: ContextStrategy::Minimal,
        },

        ConversationState::Discovery => ContextBudget {
            max_tokens: 800,
            max_documents: 3,
            max_history_turns: 2,
            strategy: ContextStrategy::Exploratory,
        },

        ConversationState::Pitch => ContextBudget {
            max_tokens: 2000,
            max_documents: 5,
            max_history_turns: 4,
            strategy: ContextStrategy::Comprehensive,
        },

        ConversationState::ObjectionHandling { objection_type } => ContextBudget {
            max_tokens: 1500,
            max_documents: 4,
            max_history_turns: 3,
            strategy: ContextStrategy::Targeted { target: objection_type.clone() },
        },

        ConversationState::Comparison => ContextBudget {
            max_tokens: 1800,
            max_documents: 5,
            max_history_turns: 2,
            strategy: ContextStrategy::Comprehensive,
        },

        ConversationState::Closing => ContextBudget {
            max_tokens: 500,
            max_documents: 2,
            max_history_turns: 5,
            strategy: ContextStrategy::ActionFocused,
        },

        _ => ContextBudget::default(),
    }
}

pub struct ContextBudget {
    pub max_tokens: usize,
    pub max_documents: usize,
    pub max_history_turns: usize,
    pub strategy: ContextStrategy,
}

pub enum ContextStrategy {
    Minimal,           // Just answer the question
    Exploratory,       // Provide options
    Comprehensive,     // Full details
    Targeted { target: String }, // Focus on specific topic
    ActionFocused,     // Call-to-action oriented
}
```

### Context Compression

When context exceeds budget, compress older content:

```rust
pub struct ContextCompressor {
    llm: Arc<dyn LanguageModel>,
}

impl ContextCompressor {
    /// Compress conversation history to fit budget
    pub async fn compress_history(
        &self,
        history: &[Turn],
        max_tokens: usize,
    ) -> Result<String> {
        let current_tokens = count_tokens(history);

        if current_tokens <= max_tokens {
            return Ok(format_history(history));
        }

        // Summarize older turns
        let split_point = history.len() / 2;
        let (old, recent) = history.split_at(split_point);

        let summary = self.summarize_turns(old).await?;
        let recent_text = format_history(recent);

        Ok(format!(
            "[Previous conversation summary: {}]\n\n{}",
            summary,
            recent_text
        ))
    }

    async fn summarize_turns(&self, turns: &[Turn]) -> Result<String> {
        let text = format_history(turns);

        let prompt = format!(r#"
Summarize this conversation excerpt in 2-3 sentences.
Focus on: customer needs, objections raised, information shared.

CONVERSATION:
{text}

SUMMARY:"#);

        let response = self.llm.generate(GenerateRequest {
            prompt,
            max_tokens: 100,
            temperature: 0.1,
            ..Default::default()
        }).await?;

        Ok(response.text.trim().to_string())
    }
}
```

---

## Implementation

### Complete Agentic RAG

```rust
pub struct AgenticRAG {
    retriever: Arc<HybridRetriever>,
    reranker: Arc<CrossEncoderReranker>,
    llm: Arc<dyn LanguageModel>,
    config: RAGConfig,
}

impl AgenticRAG {
    pub async fn retrieve(
        &self,
        query: &str,
        context: &ConversationContext,
    ) -> Result<Vec<Document>> {
        let span = tracing::info_span!("agentic_rag", query = %query);
        let _guard = span.enter();

        // Step 1: Intent classification
        let intent = classify_intent(query, context).await;
        tracing::debug!(?intent, "Classified intent");

        // Step 2: Initial retrieval
        let options = RetrieveOptions {
            limit: self.config.initial_limit,
            doc_types: Some(intent.doc_types()),
            hybrid: true,
            semantic_weight: self.config.semantic_weight,
            ..Default::default()
        };

        let mut documents = self.retriever.retrieve(query, &options).await?;
        let mut current_query = query.to_string();

        // Step 3: Iterative refinement
        for iteration in 0..self.config.max_iterations {
            // Check sufficiency
            let sufficiency = self.check_sufficiency(&current_query, &documents).await?;

            tracing::debug!(
                iteration,
                sufficient = sufficiency.sufficient,
                coverage = sufficiency.coverage,
                "Sufficiency check"
            );

            if sufficiency.sufficient || sufficiency.coverage > self.config.coverage_threshold {
                break;
            }

            // Rewrite query
            if let Some(refined) = sufficiency.refined_query {
                current_query = refined;
            } else {
                current_query = rewrite_query(
                    &current_query,
                    &documents,
                    context,
                    &*self.llm,
                ).await?;
            }

            tracing::debug!(refined_query = %current_query, "Query rewritten");

            // Re-retrieve
            let new_docs = self.retriever.retrieve(&current_query, &options).await?;
            documents.extend(new_docs);
            documents.dedup_by(|a, b| a.id == b.id);
        }

        // Step 4: Rerank
        let budget = context_budget(&context.state);
        let reranked = self.reranker
            .rerank(query, documents, budget.max_documents)
            .await?;

        // Step 5: Apply context budget
        let final_docs = self.apply_budget(reranked, &budget);

        tracing::info!(
            doc_count = final_docs.len(),
            total_tokens = count_doc_tokens(&final_docs),
            "RAG complete"
        );

        Ok(final_docs)
    }

    fn apply_budget(&self, docs: Vec<Document>, budget: &ContextBudget) -> Vec<Document> {
        let mut result = Vec::new();
        let mut token_count = 0;

        for doc in docs {
            let doc_tokens = count_tokens(&doc.content);
            if token_count + doc_tokens > budget.max_tokens {
                break;
            }
            token_count += doc_tokens;
            result.push(doc);

            if result.len() >= budget.max_documents {
                break;
            }
        }

        result
    }
}
```

---

## Configuration

### Complete RAG Configuration

```toml
# domains/gold_loan/rag.toml

[retrieval]
# Hybrid search settings
semantic_weight = 0.6          # 0.0 = all BM25, 1.0 = all semantic
initial_limit = 20             # Docs to retrieve initially
min_score = 0.3                # Minimum relevance score

[retrieval.semantic]
collection = "gold_loan_knowledge"
embedding_model = "e5-multilingual-base"
embedding_dim = 384

[retrieval.keyword]
index_path = "data/tantivy/gold_loan"
language = "hindi"  # For stemming

[agentic]
enabled = true
max_iterations = 3             # Max query refinement iterations
coverage_threshold = 0.7       # Stop if coverage above this
sufficiency_model = "qwen2.5:7b-q4"  # Fast model for checks

[reranking]
enabled = true
model = "cross-encoder/ms-marco-MiniLM-L-12-v2"
top_k = 5

[context_sizing]
# Override defaults per stage
[context_sizing.greeting]
max_tokens = 200
max_documents = 1

[context_sizing.pitch]
max_tokens = 2000
max_documents = 5

[context_sizing.objection]
max_tokens = 1500
max_documents = 4

[timing]
default = "sequential"

[timing.prefetch]
enabled = false
min_speech_duration_ms = 500   # Start prefetch after this

[experiments]
# A/B test RAG configurations
[experiments.semantic_weight_test]
enabled = true
variants = [
    { name = "low_semantic", semantic_weight = 0.4 },
    { name = "balanced", semantic_weight = 0.6 },
    { name = "high_semantic", semantic_weight = 0.8 },
]
traffic_split = [33, 34, 33]
metric = "answer_relevance"

[experiments.agentic_vs_simple]
enabled = true
variants = [
    { name = "simple", agentic_enabled = false },
    { name = "agentic", agentic_enabled = true },
]
traffic_split = [50, 50]
metric = "answer_completeness"
```

---

## Metrics & Optimization

### Key Metrics

```rust
pub struct RAGMetrics {
    // Latency
    pub retrieval_latency_ms: Histogram,
    pub reranking_latency_ms: Histogram,
    pub total_latency_ms: Histogram,

    // Quality
    pub documents_retrieved: Histogram,
    pub iterations_used: Histogram,
    pub coverage_score: Histogram,

    // Efficiency
    pub cache_hit_rate: Gauge,
    pub embedding_cache_size: Gauge,
}

impl RAGMetrics {
    pub fn record_retrieval(&self, result: &RAGResult) {
        self.retrieval_latency_ms.record(result.retrieval_ms as f64);
        self.reranking_latency_ms.record(result.reranking_ms as f64);
        self.total_latency_ms.record(result.total_ms as f64);
        self.documents_retrieved.record(result.doc_count as f64);
        self.iterations_used.record(result.iterations as f64);
        self.coverage_score.record(result.coverage);
    }
}
```

### Optimization Strategies

1. **Embedding Caching:**
   ```rust
   pub struct EmbeddingCache {
       cache: moka::future::Cache<String, Vec<f32>>,
   }

   impl EmbeddingCache {
       pub async fn get_or_compute(
           &self,
           text: &str,
           embedder: &dyn Embedder,
       ) -> Result<Vec<f32>> {
           let key = hash(text);
           self.cache
               .try_get_with(key, async { embedder.embed(text).await })
               .await
       }
   }
   ```

2. **Query Result Caching:**
   ```rust
   // Cache recent query results (semantic similarity based)
   pub struct SemanticCache {
       entries: Vec<(Vec<f32>, Vec<Document>)>,
       similarity_threshold: f32,
   }

   impl SemanticCache {
       pub fn get(&self, query_embedding: &[f32]) -> Option<Vec<Document>> {
           for (cached_emb, docs) in &self.entries {
               if cosine_similarity(query_embedding, cached_emb) > self.similarity_threshold {
                   return Some(docs.clone());
               }
           }
           None
       }
   }
   ```

3. **Batch Embedding:**
   ```rust
   // Embed multiple texts in one call
   pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
       // More efficient than individual calls
       self.model.embed_batch(texts).await
   }
   ```

---

## References

- [NVIDIA - Traditional vs Agentic RAG](https://developer.nvidia.com/blog/traditional-rag-vs-agentic-rag-why-ai-agents-need-dynamic-knowledge-to-get-smarter/)
- [Weaviate - What is Agentic RAG](https://weaviate.io/blog/what-is-agentic-rag)
- [Redis - Agentic RAG in Enterprises](https://redis.io/blog/agentic-rag-how-enterprises-are-surmounting-the-limits-of-traditional-rag/)
- [arXiv - Agentic RAG Survey](https://arxiv.org/abs/2501.09136)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [Tantivy Documentation](https://docs.rs/tantivy/)
