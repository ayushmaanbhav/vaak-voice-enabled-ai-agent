# Rust Voice Agent Framework

> Stage-based state machine with hierarchical memory for gold loan sales
>
> **Design:** Production-ready | Conversation memory | Persuasion strategies

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Conversation Stages](#conversation-stages)
3. [Agent Core](#agent-core)
4. [Memory System](#memory-system)
5. [Persuasion Engine](#persuasion-engine)
6. [State Management](#state-management)
7. [Integration](#integration)

---

## Architecture Overview

### Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Stage-based FSM** | Clear conversation progression |
| **Hierarchical memory** | Working + Episodic + Semantic |
| **Separation of concerns** | State, Strategy, Tools |
| **Async-native** | Tokio-based concurrency |
| **Trait-based** | Pluggable components |

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        VOICE AGENT FRAMEWORK                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐        │
│  │  Pipeline Input │───▶│  Agent Core     │───▶│  Pipeline Output │        │
│  │  (transcript)   │    │                 │    │  (response)      │        │
│  └─────────────────┘    └────────┬────────┘    └─────────────────┘        │
│                                  │                                          │
│                    ┌─────────────┼─────────────┐                           │
│                    │             │             │                           │
│                    ▼             ▼             ▼                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│  │  State Manager  │  │  Memory System  │  │  Tool Registry  │            │
│  │                 │  │                 │  │                 │            │
│  │  - Stage FSM    │  │  - Working      │  │  - MCP Tools    │            │
│  │  - Entity store │  │  - Episodic     │  │  - Domain tools │            │
│  │  - Intent       │  │  - Semantic     │  │  - Execution    │            │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘            │
│           │                    │                    │                      │
│           └────────────────────┼────────────────────┘                      │
│                                │                                           │
│                                ▼                                           │
│  ┌─────────────────────────────────────────────────────────────┐          │
│  │                    Persuasion Engine                         │          │
│  │                                                              │          │
│  │  - Objection handling                                        │          │
│  │  - Value proposition selection                               │          │
│  │  - Urgency/scarcity triggers                                 │          │
│  │  - Personalization                                           │          │
│  └─────────────────────────────────────────────────────────────┘          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Conversation Stages

### Stage Definitions

```rust
// crates/agent/src/stages.rs

/// Conversation stages for sales flow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConversationStage {
    /// Initial greeting and introduction
    Greeting,
    /// Understanding customer needs
    Discovery,
    /// Assessing eligibility and fit
    Qualification,
    /// Presenting the offer
    Presentation,
    /// Handling objections
    ObjectionHandling,
    /// Closing the deal
    Closing,
    /// Final wrap-up
    Farewell,
}

impl ConversationStage {
    /// Get allowed transitions from current stage
    pub fn allowed_transitions(&self) -> Vec<ConversationStage> {
        match self {
            ConversationStage::Greeting => vec![
                ConversationStage::Discovery,
                ConversationStage::Farewell,
            ],
            ConversationStage::Discovery => vec![
                ConversationStage::Qualification,
                ConversationStage::Presentation,
                ConversationStage::Farewell,
            ],
            ConversationStage::Qualification => vec![
                ConversationStage::Presentation,
                ConversationStage::Discovery,
                ConversationStage::Farewell,
            ],
            ConversationStage::Presentation => vec![
                ConversationStage::ObjectionHandling,
                ConversationStage::Closing,
                ConversationStage::Farewell,
            ],
            ConversationStage::ObjectionHandling => vec![
                ConversationStage::Presentation,
                ConversationStage::Closing,
                ConversationStage::Farewell,
            ],
            ConversationStage::Closing => vec![
                ConversationStage::ObjectionHandling,
                ConversationStage::Farewell,
            ],
            ConversationStage::Farewell => vec![],
        }
    }

    /// Get stage-specific prompt guidance
    pub fn prompt_guidance(&self) -> &'static str {
        match self {
            ConversationStage::Greeting => {
                "Introduce yourself warmly. Acknowledge any previous relationship with the bank. \
                 Ask an open question to understand their current situation."
            }
            ConversationStage::Discovery => {
                "Ask about their current gold loan situation. Understand pain points with \
                 current lender. Identify gold quantity and purpose of loan."
            }
            ConversationStage::Qualification => {
                "Assess eligibility based on gold quantity and purity. Understand loan amount \
                 needs. Check for any documentation requirements."
            }
            ConversationStage::Presentation => {
                "Present personalized benefits. Show savings calculator results. Emphasize \
                 trust and safety of Kotak. Mention the Switch & Save program."
            }
            ConversationStage::ObjectionHandling => {
                "Listen empathetically. Address specific concerns. Provide evidence and \
                 testimonials. Never be pushy or dismissive."
            }
            ConversationStage::Closing => {
                "Summarize benefits. Ask for commitment. Offer to schedule appointment. \
                 Create appropriate urgency without pressure."
            }
            ConversationStage::Farewell => {
                "Thank them for their time. Provide clear next steps. Leave door open \
                 for future contact. End on a positive note."
            }
        }
    }

    /// Get stage duration limits (soft)
    pub fn suggested_duration_seconds(&self) -> u32 {
        match self {
            ConversationStage::Greeting => 30,
            ConversationStage::Discovery => 120,
            ConversationStage::Qualification => 60,
            ConversationStage::Presentation => 90,
            ConversationStage::ObjectionHandling => 60,
            ConversationStage::Closing => 60,
            ConversationStage::Farewell => 30,
        }
    }
}

/// Stage transition event
#[derive(Debug, Clone)]
pub struct StageTransition {
    pub from: ConversationStage,
    pub to: ConversationStage,
    pub reason: TransitionReason,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
pub enum TransitionReason {
    /// Natural conversation progression
    Natural,
    /// User explicitly asked to move on
    UserRequest,
    /// Stage objectives completed
    ObjectivesComplete,
    /// Stage timeout (soft limit exceeded)
    Timeout,
    /// User disengagement detected
    Disengagement,
}
```

### Stage Manager

```rust
// crates/agent/src/stage_manager.rs

/// Manages conversation stage transitions
pub struct StageManager {
    current_stage: ConversationStage,
    stage_history: Vec<StageTransition>,
    stage_start_time: Instant,
    stage_objectives: HashMap<ConversationStage, Vec<StageObjective>>,
}

#[derive(Debug, Clone)]
pub struct StageObjective {
    pub id: String,
    pub description: String,
    pub completed: bool,
    pub required: bool,
}

impl StageManager {
    pub fn new() -> Self {
        Self {
            current_stage: ConversationStage::Greeting,
            stage_history: Vec::new(),
            stage_start_time: Instant::now(),
            stage_objectives: Self::default_objectives(),
        }
    }

    /// Attempt to transition to a new stage
    pub fn transition(&mut self, to: ConversationStage, reason: TransitionReason) -> Result<(), StageError> {
        let allowed = self.current_stage.allowed_transitions();

        if !allowed.contains(&to) {
            return Err(StageError::InvalidTransition {
                from: self.current_stage,
                to,
            });
        }

        let transition = StageTransition {
            from: self.current_stage,
            to,
            reason,
            timestamp: Instant::now(),
        };

        self.stage_history.push(transition);
        self.current_stage = to;
        self.stage_start_time = Instant::now();

        Ok(())
    }

    /// Check if stage should transition based on objectives
    pub fn should_transition(&self) -> Option<ConversationStage> {
        let objectives = self.stage_objectives.get(&self.current_stage)?;

        // Check if all required objectives are complete
        let required_complete = objectives
            .iter()
            .filter(|o| o.required)
            .all(|o| o.completed);

        if required_complete {
            // Suggest next stage
            match self.current_stage {
                ConversationStage::Greeting => Some(ConversationStage::Discovery),
                ConversationStage::Discovery => Some(ConversationStage::Qualification),
                ConversationStage::Qualification => Some(ConversationStage::Presentation),
                ConversationStage::Presentation => Some(ConversationStage::Closing),
                ConversationStage::ObjectionHandling => Some(ConversationStage::Presentation),
                ConversationStage::Closing => Some(ConversationStage::Farewell),
                ConversationStage::Farewell => None,
            }
        } else {
            None
        }
    }

    /// Mark an objective as complete
    pub fn complete_objective(&mut self, stage: ConversationStage, objective_id: &str) {
        if let Some(objectives) = self.stage_objectives.get_mut(&stage) {
            for obj in objectives {
                if obj.id == objective_id {
                    obj.completed = true;
                    break;
                }
            }
        }
    }

    /// Get current stage duration
    pub fn stage_duration(&self) -> Duration {
        self.stage_start_time.elapsed()
    }

    fn default_objectives() -> HashMap<ConversationStage, Vec<StageObjective>> {
        let mut map = HashMap::new();

        map.insert(ConversationStage::Greeting, vec![
            StageObjective {
                id: "introduce".to_string(),
                description: "Introduce self and purpose".to_string(),
                completed: false,
                required: true,
            },
            StageObjective {
                id: "acknowledge".to_string(),
                description: "Acknowledge customer relationship".to_string(),
                completed: false,
                required: false,
            },
        ]);

        map.insert(ConversationStage::Discovery, vec![
            StageObjective {
                id: "current_lender".to_string(),
                description: "Identify current gold loan lender".to_string(),
                completed: false,
                required: true,
            },
            StageObjective {
                id: "pain_points".to_string(),
                description: "Understand pain points".to_string(),
                completed: false,
                required: true,
            },
            StageObjective {
                id: "gold_details".to_string(),
                description: "Get gold weight and purity".to_string(),
                completed: false,
                required: true,
            },
        ]);

        // ... more stages

        map
    }
}
```

---

## Agent Core

### Agent Trait

```rust
// crates/agent/src/core.rs

use async_trait::async_trait;

/// Configuration for the agent
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Agent name for introduction
    pub name: String,
    /// Language preference
    pub language: String,
    /// Persona traits
    pub persona: PersonaTraits,
    /// Maximum conversation duration
    pub max_duration_seconds: u32,
    /// Enable tools
    pub tools_enabled: bool,
    /// Model configuration
    pub model_config: ModelConfig,
}

#[derive(Debug, Clone)]
pub struct PersonaTraits {
    pub warmth: f32,      // 0-1
    pub formality: f32,   // 0-1
    pub urgency: f32,     // 0-1
    pub empathy: f32,     // 0-1
}

impl Default for PersonaTraits {
    fn default() -> Self {
        Self {
            warmth: 0.8,
            formality: 0.6,
            urgency: 0.4,
            empathy: 0.9,
        }
    }
}

/// Agent response
#[derive(Debug, Clone)]
pub struct AgentResponse {
    /// Text response to speak
    pub text: String,
    /// Whether this ends the agent's turn
    pub should_end_turn: bool,
    /// Tool calls to execute
    pub tool_calls: Vec<ToolCall>,
    /// Tool results (if any)
    pub tool_results: Vec<ToolResult>,
    /// Suggested next stage
    pub suggested_stage: Option<ConversationStage>,
    /// Metadata
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone)]
pub struct ResponseMetadata {
    pub confidence: f32,
    pub intent_detected: Option<Intent>,
    pub entities_extracted: Vec<Entity>,
    pub latency_ms: u64,
}

/// Core agent trait
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get agent configuration
    fn config(&self) -> &AgentConfig;

    /// Process user input and generate response
    async fn process(
        &self,
        input: &str,
        state: &mut AgentState,
    ) -> Result<AgentResponse, AgentError>;

    /// Handle tool execution result
    async fn handle_tool_result(
        &self,
        tool_name: &str,
        result: ToolResult,
        state: &mut AgentState,
    ) -> Result<AgentResponse, AgentError>;

    /// Get available tools
    fn tools(&self) -> Vec<ToolDefinition>;

    /// Generate greeting
    async fn greeting(&self, state: &AgentState) -> Result<String, AgentError>;

    /// Generate farewell
    async fn farewell(&self, state: &AgentState) -> Result<String, AgentError>;
}
```

### Gold Loan Voice Agent

```rust
// crates/agent/src/gold_loan_agent.rs

/// Gold loan sales voice agent
pub struct GoldLoanVoiceAgent {
    config: AgentConfig,
    llm: Arc<dyn LlmProvider>,
    rag: Arc<AgenticRag>,
    tools: Arc<ToolRegistry>,
    stage_manager: Arc<RwLock<StageManager>>,
    memory: Arc<RwLock<ConversationMemory>>,
    persuasion: Arc<PersuasionEngine>,
}

impl GoldLoanVoiceAgent {
    pub async fn new(
        config: AgentConfig,
        llm: Arc<dyn LlmProvider>,
        rag: Arc<AgenticRag>,
        tools: Arc<ToolRegistry>,
    ) -> Result<Self, AgentError> {
        Ok(Self {
            config,
            llm,
            rag,
            tools,
            stage_manager: Arc::new(RwLock::new(StageManager::new())),
            memory: Arc::new(RwLock::new(ConversationMemory::new())),
            persuasion: Arc::new(PersuasionEngine::new()),
        })
    }

    /// Build system prompt for current state
    fn build_system_prompt(&self, state: &AgentState) -> String {
        let stage = state.stage;
        let guidance = stage.prompt_guidance();

        let mut prompt = format!(
            "You are {}, a friendly gold loan advisor at Kotak Mahindra Bank.\n\n",
            self.config.name
        );

        prompt.push_str(&format!(
            "## Current Stage: {:?}\n{}\n\n",
            stage, guidance
        ));

        // Add persona guidance
        prompt.push_str(&format!(
            "## Communication Style\n\
             - Warmth: {:.0}% (be {})\n\
             - Formality: {:.0}% (speak {})\n\
             - Empathy: {:.0}% (be {} to concerns)\n\n",
            self.config.persona.warmth * 100.0,
            if self.config.persona.warmth > 0.5 { "warm and friendly" } else { "professional" },
            self.config.persona.formality * 100.0,
            if self.config.persona.formality > 0.5 { "formally" } else { "casually" },
            self.config.persona.empathy * 100.0,
            if self.config.persona.empathy > 0.5 { "very attentive" } else { "acknowledge" },
        ));

        // Add customer context
        if let Some(ref customer) = state.customer {
            prompt.push_str(&format!(
                "## Customer Context\n\
                 - Name: {}\n\
                 - Current Lender: {:?}\n\
                 - Gold Weight: {:?}g\n\
                 - Segment: {:?}\n\n",
                customer.name.as_deref().unwrap_or("Unknown"),
                customer.current_lender,
                customer.gold_weight,
                customer.segment,
            ));
        }

        // Add extracted entities
        if !state.entities.is_empty() {
            prompt.push_str("## Known Information\n");
            for (key, value) in &state.entities {
                prompt.push_str(&format!("- {}: {}\n", key, value));
            }
            prompt.push_str("\n");
        }

        // Add objection handling guidance
        if stage == ConversationStage::ObjectionHandling {
            prompt.push_str(&self.persuasion.get_objection_guidance(&state.entities));
        }

        // Add available tools
        if self.config.tools_enabled {
            let tool_defs = self.tools();
            prompt.push_str("\n## Available Tools\n");
            prompt.push_str(&format_tools_for_prompt(&tool_defs));
        }

        // Add compliance reminders
        prompt.push_str("\n## Important Rules\n\
            - Never make false promises about rates or terms\n\
            - Always recommend verifying details at the branch\n\
            - If unsure, say you'll have someone follow up\n\
            - Respect if customer wants to end the call\n");

        prompt
    }
}

#[async_trait]
impl Agent for GoldLoanVoiceAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }

    async fn process(
        &self,
        input: &str,
        state: &mut AgentState,
    ) -> Result<AgentResponse, AgentError> {
        let start = Instant::now();

        // Update memory with user input
        {
            let mut memory = self.memory.write().await;
            memory.add_turn(Turn::user(input));
        }

        // Extract intent and entities
        let (intent, entities) = self.extract_intent_entities(input, state).await?;

        // Update state with extracted information
        if let Some(ref intent) = intent {
            state.intent = Some(intent.clone());
        }
        for entity in &entities {
            state.entities.insert(entity.name.clone(), entity.value.clone());
        }

        // Check for stage transition triggers
        if let Some(new_stage) = self.detect_stage_trigger(input, &intent, state).await? {
            let mut stage_manager = self.stage_manager.write().await;
            stage_manager.transition(new_stage, TransitionReason::Natural)?;
            state.stage = new_stage;
        }

        // Get RAG context if needed
        let rag_context = if self.should_retrieve(state) {
            Some(self.rag.retrieve(input, 3).await?)
        } else {
            None
        };

        // Build prompt
        let system_prompt = self.build_system_prompt(state);
        let conversation = self.build_conversation_messages(state).await;

        // Generate response
        let llm_response = self.llm.generate(
            &system_prompt,
            &conversation,
            rag_context.as_deref(),
        ).await?;

        // Parse for tool calls
        let (text, tool_calls) = self.parse_response(&llm_response)?;

        // Execute tools if any
        let tool_results = if !tool_calls.is_empty() {
            self.execute_tools(&tool_calls, state).await?
        } else {
            vec![]
        };

        // Update memory with agent response
        {
            let mut memory = self.memory.write().await;
            memory.add_turn(Turn::assistant(&text));
        }

        // Check if we should suggest stage transition
        let suggested_stage = {
            let stage_manager = self.stage_manager.read().await;
            stage_manager.should_transition()
        };

        let latency = start.elapsed().as_millis() as u64;

        Ok(AgentResponse {
            text,
            should_end_turn: true,
            tool_calls,
            tool_results,
            suggested_stage,
            metadata: ResponseMetadata {
                confidence: 0.9, // Would come from LLM
                intent_detected: intent,
                entities_extracted: entities,
                latency_ms: latency,
            },
        })
    }

    async fn handle_tool_result(
        &self,
        tool_name: &str,
        result: ToolResult,
        state: &mut AgentState,
    ) -> Result<AgentResponse, AgentError> {
        // Store tool result in state
        state.entities.insert(
            format!("tool_{}_result", tool_name),
            serde_json::to_value(&result).unwrap_or_default(),
        );

        // Generate response incorporating tool result
        let tool_context = self.format_tool_result(tool_name, &result.data)?;

        let prompt = format!(
            "The {} tool returned the following information:\n{}\n\n\
             Incorporate this naturally into your response to the customer.",
            tool_name, tool_context
        );

        let conversation = self.build_conversation_messages(state).await;
        let llm_response = self.llm.generate(
            &self.build_system_prompt(state),
            &conversation,
            Some(&prompt),
        ).await?;

        Ok(AgentResponse {
            text: llm_response,
            should_end_turn: true,
            tool_calls: vec![],
            tool_results: vec![result],
            suggested_stage: None,
            metadata: ResponseMetadata {
                confidence: 0.9,
                intent_detected: None,
                entities_extracted: vec![],
                latency_ms: 0,
            },
        })
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        // Return tool definitions
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.tools.list_definitions().await
            })
        })
    }

    async fn greeting(&self, state: &AgentState) -> Result<String, AgentError> {
        let name = state.customer
            .as_ref()
            .and_then(|c| c.name.as_deref())
            .unwrap_or("Sir/Madam");

        let greeting = if self.config.language == "hi" {
            format!("Namaste {} ji! Main {} bol raha/rahi hoon Kotak Mahindra Bank se. \
                     Aapka kuch samay mil sakta hai?", name, self.config.name)
        } else {
            format!("Hello {}! This is {} calling from Kotak Mahindra Bank. \
                     Do you have a moment to speak?", name, self.config.name)
        };

        Ok(greeting)
    }

    async fn farewell(&self, state: &AgentState) -> Result<String, AgentError> {
        let name = state.customer
            .as_ref()
            .and_then(|c| c.name.as_deref())
            .unwrap_or("Sir/Madam");

        let farewell = if self.config.language == "hi" {
            format!("Dhanyavaad {} ji aapke samay ke liye. \
                     Koi bhi sawaal ho toh hume zaroor call karein. Namaste!", name)
        } else {
            format!("Thank you {} for your time today. \
                     Please feel free to reach out if you have any questions. Have a great day!", name)
        };

        Ok(farewell)
    }
}
```

---

## Memory System

### Hierarchical Memory

```rust
// crates/agent/src/memory.rs

/// Conversation memory with hierarchical storage
pub struct ConversationMemory {
    /// Recent turns (5-10) for immediate context
    working_memory: VecDeque<Turn>,
    /// Summarized conversation history
    episodic_memory: Vec<EpisodicSummary>,
    /// Key facts and entities
    semantic_memory: HashMap<String, MemoryFact>,
    /// Configuration
    config: MemoryConfig,
}

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Maximum working memory turns
    pub working_memory_size: usize,
    /// Turns before summarization
    pub summarization_threshold: usize,
    /// Maximum episodic summaries
    pub max_episodic_summaries: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            working_memory_size: 8,
            summarization_threshold: 6,
            max_episodic_summaries: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Turn {
    pub role: TurnRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<TurnMetadata>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TurnRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnMetadata {
    pub intent: Option<String>,
    pub entities: Vec<String>,
    pub stage: Option<ConversationStage>,
}

impl Turn {
    pub fn user(content: &str) -> Self {
        Self {
            role: TurnRole::User,
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self {
            role: TurnRole::Assistant,
            content: content.to_string(),
            timestamp: chrono::Utc::now(),
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicSummary {
    /// Summary text
    pub summary: String,
    /// Key points extracted
    pub key_points: Vec<String>,
    /// Time range covered
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    /// Number of turns summarized
    pub turn_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFact {
    pub value: Value,
    pub source: String,
    pub confidence: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ConversationMemory {
    pub fn new() -> Self {
        Self::with_config(MemoryConfig::default())
    }

    pub fn with_config(config: MemoryConfig) -> Self {
        Self {
            working_memory: VecDeque::with_capacity(config.working_memory_size),
            episodic_memory: Vec::new(),
            semantic_memory: HashMap::new(),
            config,
        }
    }

    /// Add a turn to working memory
    pub fn add_turn(&mut self, turn: Turn) {
        self.working_memory.push_back(turn);

        // Check if we need to summarize
        if self.working_memory.len() >= self.config.summarization_threshold {
            self.maybe_summarize();
        }

        // Trim working memory
        while self.working_memory.len() > self.config.working_memory_size {
            self.working_memory.pop_front();
        }
    }

    /// Add a fact to semantic memory
    pub fn add_fact(&mut self, key: String, value: Value, source: &str, confidence: f32) {
        self.semantic_memory.insert(key, MemoryFact {
            value,
            source: source.to_string(),
            confidence,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Get fact from semantic memory
    pub fn get_fact(&self, key: &str) -> Option<&MemoryFact> {
        self.semantic_memory.get(key)
    }

    /// Get recent turns for context
    pub fn get_recent_turns(&self, count: usize) -> Vec<&Turn> {
        self.working_memory
            .iter()
            .rev()
            .take(count)
            .rev()
            .collect()
    }

    /// Get full context for LLM
    pub fn get_context(&self) -> ConversationContext {
        ConversationContext {
            recent_turns: self.working_memory.iter().cloned().collect(),
            summaries: self.episodic_memory.clone(),
            facts: self.semantic_memory.clone(),
        }
    }

    /// Format memory for prompt
    pub fn format_for_prompt(&self) -> String {
        let mut output = String::new();

        // Add episodic summaries
        if !self.episodic_memory.is_empty() {
            output.push_str("## Previous Conversation Summary\n");
            for summary in &self.episodic_memory {
                output.push_str(&summary.summary);
                output.push('\n');
            }
            output.push('\n');
        }

        // Add key facts
        if !self.semantic_memory.is_empty() {
            output.push_str("## Key Information\n");
            for (key, fact) in &self.semantic_memory {
                output.push_str(&format!("- {}: {}\n", key, fact.value));
            }
            output.push('\n');
        }

        // Add recent conversation
        output.push_str("## Recent Conversation\n");
        for turn in &self.working_memory {
            let role = match turn.role {
                TurnRole::User => "Customer",
                TurnRole::Assistant => "Agent",
                TurnRole::System => "System",
            };
            output.push_str(&format!("{}: {}\n", role, turn.content));
        }

        output
    }

    fn maybe_summarize(&mut self) {
        if self.working_memory.len() < self.config.summarization_threshold {
            return;
        }

        // Take oldest turns for summarization
        let turns_to_summarize: Vec<Turn> = self.working_memory
            .drain(..self.config.summarization_threshold / 2)
            .collect();

        if turns_to_summarize.is_empty() {
            return;
        }

        // Create summary (in production, use LLM)
        let summary = self.create_summary(&turns_to_summarize);

        // Add to episodic memory
        self.episodic_memory.push(summary);

        // Trim episodic memory if needed
        while self.episodic_memory.len() > self.config.max_episodic_summaries {
            self.episodic_memory.remove(0);
        }
    }

    fn create_summary(&self, turns: &[Turn]) -> EpisodicSummary {
        // Simple extractive summary (production would use LLM)
        let key_points: Vec<String> = turns
            .iter()
            .filter(|t| t.role == TurnRole::User)
            .map(|t| t.content.clone())
            .take(3)
            .collect();

        let summary = format!(
            "Customer discussed: {}",
            key_points.join("; ")
        );

        EpisodicSummary {
            summary,
            key_points,
            start_time: turns.first().map(|t| t.timestamp).unwrap_or_else(chrono::Utc::now),
            end_time: turns.last().map(|t| t.timestamp).unwrap_or_else(chrono::Utc::now),
            turn_count: turns.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub recent_turns: Vec<Turn>,
    pub summaries: Vec<EpisodicSummary>,
    pub facts: HashMap<String, MemoryFact>,
}
```

---

## Persuasion Engine

```rust
// crates/agent/src/persuasion.rs

/// Engine for persuasion strategies and objection handling
pub struct PersuasionEngine {
    objection_handlers: HashMap<ObjectionType, ObjectionHandler>,
    value_propositions: Vec<ValueProposition>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ObjectionType {
    HighInterestRate,
    TrustConcern,
    ProcessComplexity,
    TimeConstraint,
    AttachmentToCurrentLender,
    NeedToThink,
    CompetitorBetter,
    NoNeed,
}

#[derive(Debug, Clone)]
pub struct ObjectionHandler {
    pub objection_type: ObjectionType,
    pub acknowledge: String,
    pub reframe: String,
    pub evidence: Vec<String>,
    pub questions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValueProposition {
    pub id: String,
    pub headline: String,
    pub description: String,
    pub target_segments: Vec<String>,
    pub supporting_facts: Vec<String>,
}

impl PersuasionEngine {
    pub fn new() -> Self {
        Self {
            objection_handlers: Self::default_handlers(),
            value_propositions: Self::default_value_props(),
        }
    }

    /// Detect objection type from user input
    pub fn detect_objection(&self, input: &str) -> Option<ObjectionType> {
        let input_lower = input.to_lowercase();

        if input_lower.contains("interest") || input_lower.contains("rate") || input_lower.contains("expensive") {
            return Some(ObjectionType::HighInterestRate);
        }
        if input_lower.contains("trust") || input_lower.contains("safe") || input_lower.contains("fraud") {
            return Some(ObjectionType::TrustConcern);
        }
        if input_lower.contains("complicated") || input_lower.contains("difficult") || input_lower.contains("process") {
            return Some(ObjectionType::ProcessComplexity);
        }
        if input_lower.contains("busy") || input_lower.contains("time") || input_lower.contains("later") {
            return Some(ObjectionType::TimeConstraint);
        }
        if input_lower.contains("happy") && input_lower.contains("current") {
            return Some(ObjectionType::AttachmentToCurrentLender);
        }
        if input_lower.contains("think") || input_lower.contains("consider") {
            return Some(ObjectionType::NeedToThink);
        }

        None
    }

    /// Get handler for objection type
    pub fn get_handler(&self, objection: &ObjectionType) -> Option<&ObjectionHandler> {
        self.objection_handlers.get(objection)
    }

    /// Get objection handling guidance
    pub fn get_objection_guidance(&self, entities: &HashMap<String, Value>) -> String {
        let mut guidance = String::new();
        guidance.push_str("## Objection Handling Principles\n");
        guidance.push_str("1. Listen fully before responding\n");
        guidance.push_str("2. Acknowledge the concern (\"I understand...\")\n");
        guidance.push_str("3. Ask clarifying question if needed\n");
        guidance.push_str("4. Provide specific evidence, not generic claims\n");
        guidance.push_str("5. Check if concern is addressed\n\n");

        // Add context-specific handlers
        if let Some(Value::String(lender)) = entities.get("current_lender") {
            guidance.push_str(&format!(
                "Customer is with {}. Key differentiators:\n",
                lender
            ));
            guidance.push_str("- Our interest rate is X% lower\n");
            guidance.push_str("- Free locker facility\n");
            guidance.push_str("- Bridge loan for switching\n");
        }

        guidance
    }

    /// Select best value proposition for context
    pub fn select_value_prop(&self, segment: Option<&str>, objection: Option<&ObjectionType>) -> Option<&ValueProposition> {
        // Priority: address objection first
        if let Some(obj) = objection {
            return self.value_propositions.iter().find(|vp| {
                match obj {
                    ObjectionType::HighInterestRate => vp.id == "low_rate",
                    ObjectionType::TrustConcern => vp.id == "bank_safety",
                    ObjectionType::ProcessComplexity => vp.id == "easy_switch",
                    _ => false,
                }
            });
        }

        // Otherwise match segment
        if let Some(seg) = segment {
            return self.value_propositions.iter().find(|vp| {
                vp.target_segments.iter().any(|s| s == seg)
            });
        }

        // Default to first
        self.value_propositions.first()
    }

    fn default_handlers() -> HashMap<ObjectionType, ObjectionHandler> {
        let mut handlers = HashMap::new();

        handlers.insert(ObjectionType::HighInterestRate, ObjectionHandler {
            objection_type: ObjectionType::HighInterestRate,
            acknowledge: "I completely understand that interest rates are a key consideration.".to_string(),
            reframe: "Let me show you how our overall cost compares when you factor in all charges.".to_string(),
            evidence: vec![
                "Our rate of 10.5% is among the lowest in the industry".to_string(),
                "No hidden charges or foreclosure fees".to_string(),
                "Customers typically save 20-30% vs NBFCs".to_string(),
            ],
            questions: vec![
                "What rate are you currently paying?".to_string(),
                "Have you factored in processing and other charges?".to_string(),
            ],
        });

        handlers.insert(ObjectionType::TrustConcern, ObjectionHandler {
            objection_type: ObjectionType::TrustConcern,
            acknowledge: "Trust is absolutely the most important factor when it comes to your gold.".to_string(),
            reframe: "As a scheduled bank regulated by RBI, we have the highest safety standards.".to_string(),
            evidence: vec![
                "RBI-regulated scheduled bank since 1985".to_string(),
                "Bank-grade vaults with 24/7 surveillance".to_string(),
                "Full insurance coverage for your gold".to_string(),
                "Digital tracking of your gold at all times".to_string(),
            ],
            questions: vec![
                "What specific concerns do you have about safety?".to_string(),
            ],
        });

        // ... more handlers

        handlers
    }

    fn default_value_props() -> Vec<ValueProposition> {
        vec![
            ValueProposition {
                id: "low_rate".to_string(),
                headline: "Lowest Interest Rate in the Market".to_string(),
                description: "At 10.5%, save up to 30% compared to NBFCs".to_string(),
                target_segments: vec!["price_sensitive".to_string(), "high_value".to_string()],
                supporting_facts: vec![
                    "10.5% vs 18-24% at NBFCs".to_string(),
                    "Zero foreclosure charges".to_string(),
                ],
            },
            ValueProposition {
                id: "bank_safety".to_string(),
                headline: "Bank-Level Security for Your Gold".to_string(),
                description: "RBI-regulated with full insurance and bank-grade vaults".to_string(),
                target_segments: vec!["trust_seekers".to_string(), "first_time".to_string()],
                supporting_facts: vec![
                    "Scheduled bank since 1985".to_string(),
                    "Full insurance coverage".to_string(),
                ],
            },
            ValueProposition {
                id: "easy_switch".to_string(),
                headline: "Switch & Save - We Handle Everything".to_string(),
                description: "Bridge loan covers your old loan while we process the switch".to_string(),
                target_segments: vec!["switchers".to_string()],
                supporting_facts: vec![
                    "Zero out-of-pocket for switching".to_string(),
                    "Dedicated relationship manager".to_string(),
                ],
            },
        ]
    }
}
```

---

## State Management

### Agent State

```rust
// crates/agent/src/state.rs

/// Complete agent state for a conversation
#[derive(Debug, Clone)]
pub struct AgentState {
    /// Session identifier
    pub session_id: String,
    /// Current conversation stage
    pub stage: ConversationStage,
    /// Customer profile
    pub customer: Option<CustomerProfile>,
    /// Conversation history
    pub history: Vec<Turn>,
    /// Extracted entities
    pub entities: HashMap<String, Value>,
    /// Detected intent
    pub intent: Option<Intent>,
    /// Session metadata
    pub session: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerProfile {
    pub id: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub segment: Option<CustomerSegment>,
    pub current_lender: Option<String>,
    pub gold_weight: Option<f64>,
    pub gold_purity: Option<String>,
    pub loan_amount: Option<f64>,
    pub preferred_language: String,
    pub relationship_with_kotak: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CustomerSegment {
    HighValue,      // >100g gold, sophisticated
    TrustSeeker,    // Safety-focused, burned by NBFC
    FirstTime,      // New to gold loans
    PriceSensitive, // Rate-focused comparison shoppers
    Women,          // Shakti Gold segment
    Professional,   // Young urban professionals
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub name: String,
    pub confidence: f32,
    pub slots: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct SessionMetadata {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub turn_count: u32,
    pub channel: ConversationChannel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConversationChannel {
    PhoneCall,
    WebChat,
    WhatsApp,
    MobileApp,
}

impl AgentState {
    pub fn new(session_id: String, channel: ConversationChannel) -> Self {
        let now = chrono::Utc::now();
        Self {
            session_id,
            stage: ConversationStage::Greeting,
            customer: None,
            history: Vec::new(),
            entities: HashMap::new(),
            intent: None,
            session: SessionMetadata {
                start_time: now,
                last_activity: now,
                turn_count: 0,
                channel,
            },
        }
    }

    /// Update customer profile from extracted entities
    pub fn update_customer_from_entities(&mut self) {
        let customer = self.customer.get_or_insert(CustomerProfile {
            id: None,
            name: None,
            phone: None,
            segment: None,
            current_lender: None,
            gold_weight: None,
            gold_purity: None,
            loan_amount: None,
            preferred_language: "en".to_string(),
            relationship_with_kotak: None,
        });

        if let Some(Value::String(name)) = self.entities.get("customer_name") {
            customer.name = Some(name.clone());
        }
        if let Some(Value::String(lender)) = self.entities.get("current_lender") {
            customer.current_lender = Some(lender.clone());
        }
        if let Some(Value::Number(weight)) = self.entities.get("gold_weight") {
            customer.gold_weight = weight.as_f64();
        }
        if let Some(Value::String(purity)) = self.entities.get("gold_purity") {
            customer.gold_purity = Some(purity.clone());
        }
    }

    /// Increment turn count
    pub fn increment_turn(&mut self) {
        self.session.turn_count += 1;
        self.session.last_activity = chrono::Utc::now();
    }

    /// Get conversation duration
    pub fn duration(&self) -> chrono::Duration {
        chrono::Utc::now() - self.session.start_time
    }
}
```

---

## Integration

### Pipeline Integration

```rust
// crates/server/src/voice_handler.rs

/// Voice conversation handler
pub struct VoiceConversationHandler {
    agent: Arc<GoldLoanVoiceAgent>,
    pipeline: Arc<VoicePipeline>,
    state: Arc<RwLock<AgentState>>,
}

impl VoiceConversationHandler {
    pub async fn new(
        session_id: String,
        config: AgentConfig,
        customer: Option<CustomerProfile>,
    ) -> Result<Self, Error> {
        let agent = Arc::new(GoldLoanVoiceAgent::new(
            config,
            // ... dependencies
        ).await?);

        let pipeline = Arc::new(VoicePipeline::new(/* config */).await?);

        let mut state = AgentState::new(session_id, ConversationChannel::PhoneCall);
        state.customer = customer;

        Ok(Self {
            agent,
            pipeline,
            state: Arc::new(RwLock::new(state)),
        })
    }

    /// Handle incoming audio
    pub async fn handle_audio(&self, audio: AudioFrame) -> Result<(), Error> {
        self.pipeline.process_audio(audio).await
    }

    /// Handle transcript from STT
    pub async fn handle_transcript(&self, transcript: TranscriptResult) -> Result<String, Error> {
        let mut state = self.state.write().await;
        state.increment_turn();

        let response = self.agent.process(&transcript.text, &mut state).await?;

        // Execute any tool calls
        for tool_call in &response.tool_calls {
            let result = self.agent.tools
                .execute(&tool_call.name, tool_call.arguments.clone())
                .await?;

            // Handle tool result
            let _tool_response = self.agent
                .handle_tool_result(&tool_call.name, result, &mut state)
                .await?;
        }

        Ok(response.text)
    }

    /// Get greeting to start conversation
    pub async fn get_greeting(&self) -> Result<String, Error> {
        let state = self.state.read().await;
        self.agent.greeting(&state).await
    }

    /// End conversation
    pub async fn end_conversation(&self) -> Result<String, Error> {
        let state = self.state.read().await;
        self.agent.farewell(&state).await
    }
}
```
