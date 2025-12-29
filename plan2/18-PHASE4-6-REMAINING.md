# Phases 4-6: RAG, Personalization, Domain Config

> **Combined document for P1/P2 phases**

---

## Phase 4: RAG Enhancement (P1)

### Duration: 1 week
### Dependencies: Phase 1

---

### 4.1 RAG Timing Strategies

**File:** `crates/rag/src/timing.rs`

```rust
//! RAG timing strategies

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use voice_agent_core::Retriever;

/// RAG timing mode
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RAGTimingMode {
    /// Retrieve before LLM call
    Sequential,
    /// Start retrieval on speech detection
    PrefetchAsync,
    /// Retrieve in parallel with LLM
    ParallelInject,
}

/// Sequential RAG (simplest)
pub struct SequentialRAG {
    retriever: Arc<dyn Retriever>,
}

impl SequentialRAG {
    pub async fn retrieve(&self, query: &str) -> Vec<Document> {
        self.retriever.retrieve(query, &Default::default()).await.unwrap_or_default()
    }
}

/// Prefetch RAG (starts on VAD detection)
pub struct PrefetchRAG {
    retriever: Arc<dyn Retriever>,
    prefetch_handle: Mutex<Option<JoinHandle<Vec<Document>>>>,
    prefetch_query: Mutex<String>,
}

impl PrefetchRAG {
    pub fn new(retriever: Arc<dyn Retriever>) -> Self {
        Self {
            retriever,
            prefetch_handle: Mutex::new(None),
            prefetch_query: Mutex::new(String::new()),
        }
    }

    /// Called when VAD detects user speaking
    pub async fn start_prefetch(&self, partial_transcript: &str) {
        let retriever = self.retriever.clone();
        let query = partial_transcript.to_string();

        // Store query for later comparison
        *self.prefetch_query.lock().await = query.clone();

        let handle = tokio::spawn(async move {
            retriever.retrieve(&query, &Default::default()).await.unwrap_or_default()
        });

        *self.prefetch_handle.lock().await = Some(handle);
    }

    /// Get results when transcript is final
    pub async fn get_results(&self, final_query: &str) -> Vec<Document> {
        let prefetch_query = self.prefetch_query.lock().await.clone();

        if let Some(handle) = self.prefetch_handle.lock().await.take() {
            let docs = handle.await.unwrap_or_default();

            // Re-retrieve if query changed significantly
            if needs_reretrieval(&prefetch_query, final_query) {
                return self.retriever.retrieve(final_query, &Default::default())
                    .await.unwrap_or_default();
            }

            docs
        } else {
            self.retriever.retrieve(final_query, &Default::default())
                .await.unwrap_or_default()
        }
    }
}

fn needs_reretrieval(old_query: &str, new_query: &str) -> bool {
    // Simple heuristic: check word overlap
    let old_words: std::collections::HashSet<_> = old_query.split_whitespace().collect();
    let new_words: std::collections::HashSet<_> = new_query.split_whitespace().collect();
    let overlap = old_words.intersection(&new_words).count();
    let total = new_words.len();

    if total == 0 { return true; }
    (overlap as f32 / total as f32) < 0.5
}
```

### 4.2 Stage-Aware Context Sizing

**File:** `crates/rag/src/context.rs`

```rust
//! Stage-aware context budget

use voice_agent_core::ConversationState;

/// Context budget for RAG
#[derive(Debug, Clone)]
pub struct ContextBudget {
    /// Maximum tokens for context
    pub max_tokens: usize,
    /// Maximum documents to retrieve
    pub doc_limit: usize,
    /// History turns to include
    pub history_turns: usize,
}

impl Default for ContextBudget {
    fn default() -> Self {
        Self {
            max_tokens: 1000,
            doc_limit: 3,
            history_turns: 3,
        }
    }
}

/// Get context budget based on conversation state
pub fn get_context_budget(state: &ConversationState) -> ContextBudget {
    match state {
        ConversationState::Greeting => ContextBudget {
            max_tokens: 200,
            doc_limit: 1,
            history_turns: 0,
        },

        ConversationState::Discovery => ContextBudget {
            max_tokens: 800,
            doc_limit: 3,
            history_turns: 2,
        },

        ConversationState::Pitch => ContextBudget {
            max_tokens: 2000,
            doc_limit: 5,
            history_turns: 4,
        },

        ConversationState::ObjectionHandling { .. } => ContextBudget {
            max_tokens: 1500,
            doc_limit: 4,
            history_turns: 3,
        },

        ConversationState::Comparison => ContextBudget {
            max_tokens: 1800,
            doc_limit: 5,
            history_turns: 2,
        },

        ConversationState::Closing => ContextBudget {
            max_tokens: 500,
            doc_limit: 2,
            history_turns: 5,
        },

        _ => ContextBudget::default(),
    }
}
```

### 4.3 VAD → Prefetch Integration

**Update:** `crates/agent/src/voice_session.rs`

```rust
impl VoiceSession {
    async fn handle_pipeline_event(&mut self, event: PipelineEvent) {
        match event {
            PipelineEvent::PartialTranscript(transcript) => {
                // Trigger RAG prefetch on partial transcript
                if self.config.rag_prefetch_enabled {
                    if let Some(ref rag) = self.prefetch_rag {
                        rag.start_prefetch(&transcript.text).await;
                    }
                }
            }

            PipelineEvent::FinalTranscript(transcript) => {
                // Get prefetched results
                let docs = if let Some(ref rag) = self.prefetch_rag {
                    rag.get_results(&transcript.text).await
                } else {
                    self.retriever.retrieve(&transcript.text, &Default::default())
                        .await.unwrap_or_default()
                };

                // Process with agent
                // ...
            }

            // ...
        }
    }
}
```

### 4.4 Checklist

- [ ] Create RAGTimingMode enum
- [ ] Implement SequentialRAG
- [ ] Implement PrefetchRAG with JoinHandle
- [ ] Create ContextBudget struct
- [ ] Implement get_context_budget()
- [ ] Wire VAD events to prefetch
- [ ] Add configuration options
- [ ] Add tests

---

## Phase 5: Personalization Engine (P2)

### Duration: 1 week
### Dependencies: Phase 1

---

### 5.1 Crate Structure

```
crates/personalization/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── segments.rs
    ├── strategy.rs
    ├── disclosure.rs
    └── guardrails.rs
```

### 5.2 Segment Detection

**File:** `crates/personalization/src/segments.rs`

```rust
//! Customer segment detection

use voice_agent_core::{CustomerProfile, CustomerSegment};

/// Segment detector with heuristics
pub struct SegmentDetector;

impl SegmentDetector {
    pub fn new() -> Self {
        Self
    }

    /// Detect segment from profile
    pub fn detect(&self, profile: &CustomerProfile) -> CustomerSegment {
        // High value: >100g gold or >5L existing loan
        if profile.gold_weight.unwrap_or(0.0) > 100.0 {
            return CustomerSegment::HighValue;
        }

        // Trust seeker: existing competitor loan
        if profile.current_lender.is_some() {
            return CustomerSegment::TrustSeeker;
        }

        // Women segment
        if profile.gender.as_deref() == Some("female") {
            return CustomerSegment::Women;
        }

        // Young professional: age < 35
        if let Some(age) = profile.age {
            if age < 35 {
                return CustomerSegment::Professional;
            }
        }

        // First time: no existing gold assets mentioned
        if profile.gold_weight.is_none() && profile.current_lender.is_none() {
            return CustomerSegment::FirstTime;
        }

        // Default
        CustomerSegment::TrustSeeker
    }

    /// Get confidence score for segment
    pub fn confidence(&self, segment: CustomerSegment, profile: &CustomerProfile) -> f32 {
        // Higher confidence when more signals align
        let mut score = 0.5;

        match segment {
            CustomerSegment::HighValue => {
                if profile.gold_weight.unwrap_or(0.0) > 200.0 {
                    score += 0.3;
                }
            }
            CustomerSegment::TrustSeeker => {
                if profile.current_lender.is_some() {
                    score += 0.3;
                }
            }
            _ => {}
        }

        score.min(1.0)
    }
}
```

### 5.3 Persuasion Strategy

**File:** `crates/personalization/src/strategy.rs`

```rust
//! Segment-specific persuasion strategies

use voice_agent_core::CustomerSegment;

/// Persuasion strategy for a segment
pub struct PersuasionStrategy {
    pub segment: CustomerSegment,
    pub key_messages: Vec<String>,
    pub warmth: f32,
    pub formality: f32,
    pub urgency: f32,
}

impl PersuasionStrategy {
    /// Get strategy for segment
    pub fn for_segment(segment: CustomerSegment) -> Self {
        match segment {
            CustomerSegment::HighValue => Self {
                segment,
                key_messages: vec![
                    "Premium relationship manager".to_string(),
                    "Higher loan limits".to_string(),
                    "Priority processing".to_string(),
                    "Dedicated branch service".to_string(),
                ],
                warmth: 0.8,
                formality: 0.7,
                urgency: 0.4,
            },

            CustomerSegment::TrustSeeker => Self {
                segment,
                key_messages: vec![
                    "RBI-regulated scheduled bank".to_string(),
                    "Bank-grade security vaults".to_string(),
                    "Full insurance coverage".to_string(),
                    "Digital tracking of gold".to_string(),
                ],
                warmth: 0.95,
                formality: 0.6,
                urgency: 0.3,
            },

            CustomerSegment::Women => Self {
                segment,
                key_messages: vec![
                    "Women-friendly staff".to_string(),
                    "Private evaluation rooms".to_string(),
                    "Special women's rates".to_string(),
                    "Shakti program benefits".to_string(),
                ],
                warmth: 0.9,
                formality: 0.5,
                urgency: 0.3,
            },

            CustomerSegment::Professional => Self {
                segment,
                key_messages: vec![
                    "Quick digital process".to_string(),
                    "Instant approval".to_string(),
                    "Mobile app tracking".to_string(),
                    "Flexible repayment".to_string(),
                ],
                warmth: 0.7,
                formality: 0.5,
                urgency: 0.6,
            },

            CustomerSegment::FirstTime => Self {
                segment,
                key_messages: vec![
                    "Simple process".to_string(),
                    "Guidance at every step".to_string(),
                    "No hidden charges".to_string(),
                    "Safe and secure".to_string(),
                ],
                warmth: 0.9,
                formality: 0.4,
                urgency: 0.2,
            },

            CustomerSegment::PriceSensitive => Self {
                segment,
                key_messages: vec![
                    "Lowest interest rates".to_string(),
                    "Zero processing fee".to_string(),
                    "No prepayment penalty".to_string(),
                    "Save vs competitors".to_string(),
                ],
                warmth: 0.8,
                formality: 0.5,
                urgency: 0.5,
            },
        }
    }

    /// Get key messages for prompt injection
    pub fn get_prompt_context(&self) -> String {
        format!(
            "Key messages to emphasize:\n{}",
            self.key_messages.iter()
                .map(|m| format!("- {}", m))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}
```

### 5.4 AI Disclosure

**File:** `crates/personalization/src/disclosure.rs`

```rust
//! AI identity disclosure timing

/// When to disclose AI identity
#[derive(Debug, Clone, Copy)]
pub enum DisclosureTiming {
    /// Disclose immediately in greeting
    Immediate,
    /// Disclose after initial greeting
    AfterGreeting,
    /// Only when asked
    WhenAsked,
    /// Weave naturally into conversation
    NaturalMention,
}

/// Disclosure handler
pub struct DisclosureHandler {
    timing: DisclosureTiming,
    disclosed: bool,
    disclosure_text: String,
}

impl DisclosureHandler {
    pub fn new(timing: DisclosureTiming) -> Self {
        Self {
            timing,
            disclosed: false,
            disclosure_text: "I'm Priya, an AI assistant from Kotak Bank".to_string(),
        }
    }

    /// Check if should disclose on this turn
    pub fn should_disclose(&self, turn: usize, user_asked: bool) -> bool {
        if self.disclosed {
            return false;
        }

        match self.timing {
            DisclosureTiming::Immediate => turn == 0,
            DisclosureTiming::AfterGreeting => turn == 1,
            DisclosureTiming::WhenAsked => user_asked,
            DisclosureTiming::NaturalMention => turn >= 2 && turn <= 4,
        }
    }

    /// Mark as disclosed
    pub fn mark_disclosed(&mut self) {
        self.disclosed = true;
    }

    /// Get disclosure text
    pub fn get_disclosure(&self) -> &str {
        &self.disclosure_text
    }

    /// Weave disclosure into response
    pub fn weave_into_response(&self, response: &str) -> String {
        if self.disclosed {
            return response.to_string();
        }

        // Natural mention style
        format!(
            "{} As {}, I want to help you get the best deal.",
            response,
            self.disclosure_text
        )
    }
}
```

### 5.5 Checklist

- [ ] Create personalization crate
- [ ] Implement SegmentDetector
- [ ] Implement PersuasionStrategy
- [ ] Implement DisclosureHandler
- [ ] Add psychology guardrails
- [ ] Integrate with PromptBuilder
- [ ] Add tests

---

## Phase 6: Domain Configuration (P2)

### Duration: 2 weeks
### Dependencies: Phases 2, 5

---

### 6.1 Domain Directory Structure

```
voice-agent-rust/domains/
└── gold_loan/
    ├── knowledge/
    │   ├── products.yaml
    │   ├── competitors.yaml
    │   ├── objections.yaml
    │   └── faq.yaml
    ├── prompts/
    │   ├── system.tera
    │   ├── greeting.tera
    │   ├── pitch.tera
    │   └── objection_handlers.tera
    ├── segments.toml
    ├── tools.toml
    ├── compliance.toml
    └── experiments.toml
```

### 6.2 Domain Loader

**File:** `crates/config/src/domain.rs`

```rust
//! Domain configuration loader

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::Deserialize;

/// Domain configuration
pub struct Domain {
    pub name: String,
    pub knowledge: KnowledgeBase,
    pub prompts: PromptTemplates,
    pub segments: SegmentConfig,
    pub tools: ToolConfig,
    pub compliance: ComplianceRulesConfig,
    pub experiments: ExperimentConfig,
}

/// Domain loader
pub struct DomainLoader {
    base_path: PathBuf,
}

impl DomainLoader {
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }

    /// Load domain by name
    pub fn load_domain(&self, name: &str) -> Result<Domain, DomainError> {
        let domain_path = self.base_path.join(name);

        if !domain_path.exists() {
            return Err(DomainError::NotFound(name.to_string()));
        }

        Ok(Domain {
            name: name.to_string(),
            knowledge: self.load_knowledge(&domain_path)?,
            prompts: self.load_prompts(&domain_path)?,
            segments: self.load_toml(&domain_path.join("segments.toml"))?,
            tools: self.load_toml(&domain_path.join("tools.toml"))?,
            compliance: self.load_toml(&domain_path.join("compliance.toml"))?,
            experiments: self.load_toml(&domain_path.join("experiments.toml"))?,
        })
    }

    fn load_knowledge(&self, path: &Path) -> Result<KnowledgeBase, DomainError> {
        let knowledge_path = path.join("knowledge");
        let mut knowledge = KnowledgeBase::default();

        for entry in std::fs::read_dir(&knowledge_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "yaml").unwrap_or(false) {
                let content = std::fs::read_to_string(&path)?;
                let docs: Vec<KnowledgeDoc> = serde_yaml::from_str(&content)?;
                knowledge.documents.extend(docs);
            }
        }

        Ok(knowledge)
    }

    fn load_prompts(&self, path: &Path) -> Result<PromptTemplates, DomainError> {
        let prompts_path = path.join("prompts");
        PromptTemplates::from_directory(&prompts_path)
    }

    fn load_toml<T: for<'de> Deserialize<'de>>(&self, path: &Path) -> Result<T, DomainError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Domain not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Template error: {0}")]
    Template(String),
}

/// Knowledge base
#[derive(Default)]
pub struct KnowledgeBase {
    pub documents: Vec<KnowledgeDoc>,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeDoc {
    pub id: String,
    pub title: String,
    pub content: String,
    pub tags: Vec<String>,
}
```

### 6.3 Prompt Templates with Tera

**File:** `crates/config/src/prompts.rs`

```rust
//! Tera-based prompt templates

use tera::{Tera, Context};
use std::path::Path;

/// Prompt template engine
pub struct PromptTemplates {
    tera: Tera,
}

impl PromptTemplates {
    pub fn from_directory(path: &Path) -> Result<Self, super::DomainError> {
        let pattern = path.join("**/*.tera").to_string_lossy().to_string();

        let tera = Tera::new(&pattern)
            .map_err(|e| super::DomainError::Template(e.to_string()))?;

        Ok(Self { tera })
    }

    /// Render a template
    pub fn render(&self, name: &str, context: &Context) -> Result<String, String> {
        self.tera.render(name, context)
            .map_err(|e| e.to_string())
    }

    /// Render system prompt
    pub fn render_system(&self, context: &PromptContext) -> Result<String, String> {
        let mut ctx = Context::new();
        ctx.insert("agent_name", &context.agent_name);
        ctx.insert("domain", &context.domain);
        ctx.insert("segment", &context.segment);
        ctx.insert("key_messages", &context.key_messages);
        ctx.insert("warmth", &context.warmth);

        self.render("system.tera", &ctx)
    }
}

/// Context for prompt rendering
pub struct PromptContext {
    pub agent_name: String,
    pub domain: String,
    pub segment: String,
    pub key_messages: Vec<String>,
    pub warmth: f32,
}
```

### 6.4 Sample Domain Files

**domains/gold_loan/segments.toml:**
```toml
[[segments]]
name = "high_value"
display_name = "High Value"
description = "MSME, 5-25L loans"
warmth = 0.8
formality = 0.7

[[segments.key_messages]]
text = "Premium relationship manager"

[[segments]]
name = "trust_seeker"
display_name = "Trust Seeker"
description = "Safety-focused customers"
warmth = 0.95
formality = 0.6

[[segments.key_messages]]
text = "RBI-regulated scheduled bank"
text = "Bank-grade security vaults"
```

**domains/gold_loan/compliance.toml:**
```toml
[rules]
version = "1.0"

[[rules.forbidden_phrases]]
phrase = "guaranteed returns"
severity = "critical"

[[rules.forbidden_phrases]]
phrase = "100% safe"
severity = "error"

[[rules.claims_requiring_disclaimer]]
pattern = "lowest.*rate"
disclaimer = "Subject to eligibility and market conditions"

[rate_rules]
min_rate = 7.0
max_rate = 24.0
```

**domains/gold_loan/prompts/system.tera:**
```
You are {{ agent_name }}, a friendly Gold Loan specialist at Kotak Mahindra Bank.

Domain: {{ domain }}
Customer Segment: {{ segment }}
Warmth Level: {{ warmth }}

Key Messages to Emphasize:
{% for message in key_messages %}
- {{ message }}
{% endfor %}

Guidelines:
- Be helpful and informative
- Never make false promises
- Always disclose terms clearly
- Respect customer's pace
```

### 6.5 Experiment Framework

**File:** `crates/experiments/src/lib.rs`

```rust
//! A/B Testing experiment framework

use std::collections::HashMap;
use serde::Deserialize;
use rand::Rng;

/// Experiment definition
#[derive(Debug, Deserialize)]
pub struct Experiment {
    pub id: String,
    pub name: String,
    pub description: String,
    pub variants: Vec<Variant>,
    pub allocation: AllocationStrategy,
    pub enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct Variant {
    pub id: String,
    pub name: String,
    pub weight: f32,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AllocationStrategy {
    Random,
    SessionBased,
    UserBased,
}

/// Experiment runner
pub struct ExperimentRunner {
    experiments: HashMap<String, Experiment>,
}

impl ExperimentRunner {
    pub fn new() -> Self {
        Self {
            experiments: HashMap::new(),
        }
    }

    pub fn load_experiments(&mut self, config: Vec<Experiment>) {
        for exp in config {
            self.experiments.insert(exp.id.clone(), exp);
        }
    }

    /// Assign variant for session
    pub fn assign_variant(&self, experiment_id: &str, session_id: &str) -> Option<&Variant> {
        let experiment = self.experiments.get(experiment_id)?;

        if !experiment.enabled {
            return None;
        }

        match experiment.allocation {
            AllocationStrategy::Random => {
                self.random_assignment(&experiment.variants)
            }
            AllocationStrategy::SessionBased => {
                self.deterministic_assignment(&experiment.variants, session_id)
            }
            AllocationStrategy::UserBased => {
                // Same as session-based for now
                self.deterministic_assignment(&experiment.variants, session_id)
            }
        }
    }

    fn random_assignment<'a>(&self, variants: &'a [Variant]) -> Option<&'a Variant> {
        let total_weight: f32 = variants.iter().map(|v| v.weight).sum();
        let mut rng = rand::thread_rng();
        let point = rng.gen::<f32>() * total_weight;

        let mut cumulative = 0.0;
        for variant in variants {
            cumulative += variant.weight;
            if point < cumulative {
                return Some(variant);
            }
        }

        variants.last()
    }

    fn deterministic_assignment<'a>(&self, variants: &'a [Variant], key: &str) -> Option<&'a Variant> {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        let total_weight: f32 = variants.iter().map(|v| v.weight).sum();
        let point = (hash as f32 / u64::MAX as f32) * total_weight;

        let mut cumulative = 0.0;
        for variant in variants {
            cumulative += variant.weight;
            if point < cumulative {
                return Some(variant);
            }
        }

        variants.last()
    }
}
```

### 6.6 Checklist

- [ ] Create domains/ directory structure
- [ ] Create sample YAML knowledge files
- [ ] Create Tera prompt templates
- [ ] Create TOML config files
- [ ] Implement DomainLoader
- [ ] Implement PromptTemplates with Tera
- [ ] Create experiments crate
- [ ] Implement ExperimentRunner
- [ ] Migrate hardcoded values to config
- [ ] Add hot-reload for configs
- [ ] Add validation
- [ ] Add tests

---

## Combined Summary

| Phase | Components | Effort | Files |
|-------|------------|--------|-------|
| **4. RAG** | Timing strategies, Context sizing, VAD→Prefetch | 1 week | 3 |
| **5. Personalization** | Segments, Strategies, Disclosure | 1 week | 4 |
| **6. Domain Config** | Loader, Templates, Experiments | 2 weeks | 10+ |

---

*These phases complete the architecture alignment. After implementation, the codebase will match ARCHITECTURE_v2.md.*
