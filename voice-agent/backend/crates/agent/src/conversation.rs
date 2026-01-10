//! Conversation Management
//!
//! Manages the overall conversation flow and state.
//!
//! ## Phase 2 (Domain-Agnosticism): ConversationContext Trait
//!
//! The `ConversationContext` trait abstracts conversation management, allowing
//! domain-agnostic agents to work with any conversation implementation. This
//! enables:
//! - Testing with mock conversations
//! - Alternative memory backends
//! - Custom stage transitions per domain

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

use crate::intent::{DetectedIntent, IntentDetector};
use crate::memory::{AgenticMemory, AgenticMemoryConfig, MemoryConfig};
use crate::memory_legacy::{ConversationMemory, MemoryEntry};
use crate::stage::{ConversationStage, StageManager, TransitionReason};
use crate::AgentError;
use voice_agent_config::domain::StagesConfig;
use voice_agent_core::{Turn, TurnRole};

// =============================================================================
// Phase 2: ConversationContext Trait (Domain-Agnostic Abstraction)
// =============================================================================

/// Trait for conversation context abstraction
///
/// This trait allows domain-agnostic agents to work with any conversation
/// implementation. Implementations must be thread-safe (Send + Sync).
///
/// # Example
/// ```ignore
/// // Agent uses trait bound instead of concrete Conversation
/// pub struct DomainAgent<C: ConversationContext> {
///     conversation: Arc<C>,
///     // ...
/// }
/// ```
pub trait ConversationContext: Send + Sync {
    /// Get session ID
    fn session_id(&self) -> &str;

    /// Get current conversation state
    fn state(&self) -> ConversationState;

    /// Get current conversation stage
    fn stage(&self) -> ConversationStage;

    /// Get conversation duration
    fn duration(&self) -> Duration;

    /// Get turn count
    fn turn_count(&self) -> usize;

    /// Check if conversation is active
    fn is_active(&self) -> bool;

    /// Add a user turn to the conversation
    ///
    /// Returns the detected intent from the user's input.
    fn add_user_turn(&self, content: &str) -> Result<DetectedIntent, AgentError>;

    /// Add an assistant turn to the conversation
    fn add_assistant_turn(&self, content: &str) -> Result<(), AgentError>;

    /// Transition to a new conversation stage
    fn transition_stage(&self, to: ConversationStage) -> Result<(), AgentError>;

    /// Get context string for the conversation
    fn get_context(&self) -> String;

    /// Get context optimized for a specific query with token budget
    fn get_context_for_query(&self, query: &str, max_tokens: usize) -> String;

    /// Get access to agentic memory
    fn agentic_memory(&self) -> &Arc<AgenticMemory>;

    /// Get legacy memory reference
    fn memory(&self) -> &ConversationMemory;

    /// Get legacy memory Arc for async operations
    fn memory_arc(&self) -> Arc<ConversationMemory>;

    /// Get stage manager reference
    fn stage_manager(&self) -> &StageManager;

    /// Record a fact in memory
    fn record_fact(&self, key: &str, value: &str, confidence: f32);

    /// Get recent messages for LLM context
    fn get_messages(&self) -> Vec<(String, String)>;

    /// Get stage guidance text
    fn get_stage_guidance(&self) -> &'static str;

    /// Get suggested questions for current stage
    fn get_suggested_questions(&self) -> Vec<&'static str>;

    /// End the conversation
    fn end(&self, reason: EndReason);

    /// Pause the conversation
    fn pause(&self);

    /// Resume the conversation
    fn resume(&self);

    // =========================================================================
    // Compliance Methods (RBI Requirements)
    // =========================================================================

    /// Get compliance status
    fn compliance(&self) -> ComplianceStatus;

    /// Check if AI disclosure has been given
    fn ai_disclosure_given(&self) -> bool;

    /// Mark AI disclosure as given, returns disclosure message
    fn mark_ai_disclosed(&self) -> String;

    /// Record recording consent
    fn record_recording_consent(&self, given: bool, method: ConsentMethod);

    /// Record PII processing consent
    fn record_pii_consent(&self, given: bool, method: ConsentMethod);

    /// Check if conversation is compliant
    fn is_compliant(&self) -> bool;

    /// Check if PII processing is allowed
    fn can_process_pii(&self) -> bool;

    /// Get pending compliance requirements
    fn pending_compliance(&self) -> Vec<String>;

    /// Get AI disclosure message for configured language
    fn get_ai_disclosure_message(&self) -> String;

    /// Subscribe to conversation events
    fn subscribe(&self) -> broadcast::Receiver<ConversationEvent>;
}

/// Conversation configuration
#[derive(Debug, Clone)]
pub struct ConversationConfig {
    /// Maximum duration in seconds
    pub max_duration_seconds: u32,
    /// Session timeout in seconds
    pub session_timeout_seconds: u32,
    /// Memory config
    pub memory: MemoryConfig,
    /// Enable intent detection
    pub intent_detection: bool,
    /// Default language
    pub language: String,
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            max_duration_seconds: 1800, // 30 minutes max conversation
            session_timeout_seconds: 300, // 5 minutes inactivity timeout
            memory: MemoryConfig::default(),
            intent_detection: true,
            language: "en".to_string(),
        }
    }
}

/// Conversation event
#[derive(Debug, Clone)]
pub enum ConversationEvent {
    /// Conversation started
    Started { session_id: String },
    /// Turn added
    TurnAdded { role: TurnRole, content: String },
    /// Intent detected
    IntentDetected(DetectedIntent),
    /// Stage changed
    StageChanged {
        from: ConversationStage,
        to: ConversationStage,
    },
    /// Fact learned
    FactLearned { key: String, value: String },
    /// Tool called
    ToolCalled { name: String, success: bool },
    /// Conversation ended
    Ended { reason: EndReason },
    /// Error occurred
    Error(String),
}

/// Reason for conversation end
#[derive(Debug, Clone)]
pub enum EndReason {
    UserEnded,
    AgentEnded,
    Timeout,
    MaxDuration,
    Error(String),
}

/// Conversation state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationState {
    Active,
    Paused,
    Ended,
}

// =============================================================================
// P0 FIX: AI Disclosure and Consent Tracking (RBI Compliance)
// =============================================================================

/// P0 FIX: AI Disclosure tracking for RBI compliance
///
/// RBI requires that customers are informed they are speaking with an AI,
/// and must be given the option to speak with a human agent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiDisclosure {
    /// Whether AI disclosure has been given to the customer
    pub given: bool,
    /// When the disclosure was given
    pub timestamp: Option<DateTime<Utc>>,
    /// Language in which disclosure was given
    pub language: Option<String>,
    /// The actual disclosure text that was used
    pub disclosure_text: Option<String>,
    /// Whether human escalation option was offered
    pub human_option_offered: bool,
}

impl AiDisclosure {
    /// Mark AI disclosure as given
    pub fn mark_disclosed(&mut self, language: &str, text: &str, human_option: bool) {
        self.given = true;
        self.timestamp = Some(Utc::now());
        self.language = Some(language.to_string());
        self.disclosure_text = Some(text.to_string());
        self.human_option_offered = human_option;
    }

    /// Get localized AI disclosure message
    pub fn get_disclosure_message(language: &str) -> &'static str {
        match language {
            "hi" | "hindi" => "यह एक AI सहायक है। आप किसी भी समय 'एजेंट से बात करें' कहकर मानव एजेंट से बात कर सकते हैं।",
            "mr" | "marathi" => "हे एक AI सहाय्यक आहे. तुम्ही कधीही 'एजंटशी बोला' असे सांगून मानवी एजंटशी बोलू शकता.",
            "ta" | "tamil" => "இது ஒரு AI உதவியாளர். நீங்கள் எந்த நேரத்திலும் 'ஏஜெண்டுடன் பேசுங்கள்' என்று சொல்லி மனித ஏஜெண்டுடன் பேசலாம்.",
            "te" | "telugu" => "ఇది AI సహాయకుడు. మీరు ఎప్పుడైనా 'ఏజెంట్‌తో మాట్లాడండి' అని చెప్పి మానవ ఏజెంట్‌తో మాట్లాడవచ్చు.",
            "bn" | "bengali" => "এটি একটি AI সহায়ক। আপনি যেকোনো সময় 'এজেন্টের সাথে কথা বলুন' বলে একজন মানব এজেন্টের সাথে কথা বলতে পারেন।",
            "gu" | "gujarati" => "આ એક AI સહાયક છે. તમે ગમે ત્યારે 'એજન્ટ સાથે વાત કરો' કહીને માનવ એજન્ટ સાથે વાત કરી શકો છો.",
            "kn" | "kannada" => "ಇದು AI ಸಹಾಯಕ. ನೀವು ಯಾವುದೇ ಸಮಯದಲ್ಲಿ 'ಏಜೆಂಟ್‌ನೊಂದಿಗೆ ಮಾತನಾಡಿ' ಎಂದು ಹೇಳುವ ಮೂಲಕ ಮಾನವ ಏಜೆಂಟ್‌ನೊಂದಿಗೆ ಮಾತನಾಡಬಹುದು.",
            "ml" | "malayalam" => "ഇത് ഒരു AI അസിസ്റ്റന്റ് ആണ്. 'ഏജന്റിനോട് സംസാരിക്കുക' എന്ന് പറഞ്ഞ് നിങ്ങൾക്ക് എപ്പോൾ വേണമെങ്കിലും ഒരു മനുഷ്യ ഏജന്റുമായി സംസാരിക്കാം.",
            "pa" | "punjabi" => "ਇਹ ਇੱਕ AI ਸਹਾਇਕ ਹੈ। ਤੁਸੀਂ ਕਿਸੇ ਵੀ ਸਮੇਂ 'ਏਜੰਟ ਨਾਲ ਗੱਲ ਕਰੋ' ਕਹਿ ਕੇ ਮਨੁੱਖੀ ਏਜੰਟ ਨਾਲ ਗੱਲ ਕਰ ਸਕਦੇ ਹੋ।",
            "or" | "odia" => "ଏହା ଏକ AI ସହାୟକ। ଆପଣ ଯେକୌଣସି ସମୟରେ 'ଏଜେଣ୍ଟଙ୍କ ସହ କଥା ହୁଅନ୍ତୁ' କହି ମାନବ ଏଜେଣ୍ଟଙ୍କ ସହ କଥା ହୋଇପାରିବେ।",
            _ => "This is an AI assistant. You can speak with a human agent at any time by saying 'speak to agent'.",
        }
    }
}

/// P0 FIX: Consent tracking for RBI compliance
///
/// Tracks various consents required for voice banking:
/// - Recording consent: Customer agrees to call recording
/// - PII processing consent: Customer agrees to personal data processing
/// - Marketing consent: Optional consent for promotional communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// Consent for call recording
    pub recording_consent: bool,
    /// When recording consent was given
    pub recording_consent_timestamp: Option<DateTime<Utc>>,
    /// Consent for PII processing
    pub pii_processing_consent: bool,
    /// When PII consent was given
    pub pii_consent_timestamp: Option<DateTime<Utc>>,
    /// Optional marketing consent
    pub marketing_consent: Option<bool>,
    /// When marketing consent was given/denied
    pub marketing_consent_timestamp: Option<DateTime<Utc>>,
    /// Method by which consent was obtained
    pub consent_method: ConsentMethod,
    /// Language of consent
    pub consent_language: String,
}

/// Method by which consent was obtained
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsentMethod {
    /// Customer explicitly said "yes" or equivalent
    Voice,
    /// Customer typed confirmation
    Text,
    /// Implied by continuing after disclosure
    Implied,
    /// Customer explicitly clicked/tapped consent
    Explicit,
}

impl Default for ConsentMethod {
    fn default() -> Self {
        Self::Implied
    }
}

impl Default for ConsentRecord {
    fn default() -> Self {
        Self {
            recording_consent: false,
            recording_consent_timestamp: None,
            pii_processing_consent: false,
            pii_consent_timestamp: None,
            marketing_consent: None,
            marketing_consent_timestamp: None,
            consent_method: ConsentMethod::Implied,
            consent_language: "en".to_string(),
        }
    }
}

impl ConsentRecord {
    /// Record recording consent
    pub fn record_recording_consent(&mut self, given: bool, method: ConsentMethod) {
        self.recording_consent = given;
        self.recording_consent_timestamp = Some(Utc::now());
        self.consent_method = method;
    }

    /// Record PII processing consent
    pub fn record_pii_consent(&mut self, given: bool, method: ConsentMethod) {
        self.pii_processing_consent = given;
        self.pii_consent_timestamp = Some(Utc::now());
        self.consent_method = method;
    }

    /// Record marketing consent
    pub fn record_marketing_consent(&mut self, given: Option<bool>) {
        self.marketing_consent = given;
        self.marketing_consent_timestamp = Some(Utc::now());
    }

    /// Check if minimum required consents are given
    pub fn has_minimum_consent(&self) -> bool {
        // For voice banking, recording consent is essential
        // PII consent is required before processing personal data
        self.recording_consent
    }

    /// Check if all consents are given
    pub fn has_full_consent(&self) -> bool {
        self.recording_consent && self.pii_processing_consent
    }
}

/// P0 FIX: Compliance status for the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// AI disclosure tracking
    pub ai_disclosure: AiDisclosure,
    /// Consent tracking
    pub consent: ConsentRecord,
    /// Whether compliance requirements are met
    pub compliant: bool,
    /// List of pending compliance requirements
    pub pending_requirements: Vec<String>,
}

impl Default for ComplianceStatus {
    fn default() -> Self {
        Self {
            ai_disclosure: AiDisclosure::default(),
            consent: ConsentRecord::default(),
            compliant: false,
            pending_requirements: vec![
                "ai_disclosure".to_string(),
                "recording_consent".to_string(),
            ],
        }
    }
}

impl ComplianceStatus {
    /// Update compliance status based on current state
    pub fn update(&mut self) {
        self.pending_requirements.clear();

        if !self.ai_disclosure.given {
            self.pending_requirements.push("ai_disclosure".to_string());
        }

        if !self.consent.recording_consent {
            self.pending_requirements
                .push("recording_consent".to_string());
        }

        self.compliant = self.pending_requirements.is_empty();
    }

    /// Check if ready for PII processing
    pub fn can_process_pii(&self) -> bool {
        self.ai_disclosure.given && self.consent.pii_processing_consent
    }
}

/// Conversation manager
pub struct Conversation {
    /// Session ID
    session_id: String,
    /// Configuration
    config: ConversationConfig,
    /// Start time
    start_time: Instant,
    /// Last activity time
    last_activity: Mutex<Instant>,
    /// Current state
    state: Mutex<ConversationState>,
    /// Stage manager
    stage_manager: Arc<StageManager>,
    /// Memory (legacy)
    memory: Arc<ConversationMemory>,
    /// Phase 10: Agentic memory for archival retrieval
    agentic_memory: Arc<AgenticMemory>,
    /// Intent detector
    intent_detector: Arc<IntentDetector>,
    /// Event sender
    event_tx: broadcast::Sender<ConversationEvent>,
    /// Turn counter
    turn_count: Mutex<usize>,
    /// P0 FIX: Compliance status (AI disclosure, consent)
    compliance: Mutex<ComplianceStatus>,
    /// P16 FIX: Config-driven stage transitions
    /// When set, intent-to-stage transitions are loaded from config instead of hardcoded
    stages_config: Option<Arc<StagesConfig>>,
    /// P16 FIX: AI disclosure message loaded from config (RBI compliance)
    /// Stored at construction to avoid needing view reference later
    ai_disclosure_message: String,
}

impl Conversation {
    /// Create a new conversation
    ///
    /// NOTE: For config-driven operation, use `from_view()` instead to wire
    /// domain-specific competitor patterns into the intent detector.
    pub fn new(session_id: impl Into<String>, config: ConversationConfig) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        let session_id_str = session_id.into();

        // Phase 10: Create agentic memory with session ID for archival retrieval
        let agentic_config = AgenticMemoryConfig::default();

        // P16 FIX: Use static fallback for AI disclosure (config-driven version uses from_view)
        let ai_disclosure = AiDisclosure::get_disclosure_message(&config.language).to_string();

        Self {
            session_id: session_id_str.clone(),
            config: config.clone(),
            start_time: Instant::now(),
            last_activity: Mutex::new(Instant::now()),
            state: Mutex::new(ConversationState::Active),
            stage_manager: Arc::new(StageManager::new()),
            memory: Arc::new(ConversationMemory::new(config.memory)),
            agentic_memory: Arc::new(AgenticMemory::new(agentic_config, session_id_str)),
            intent_detector: Arc::new(IntentDetector::new()),
            event_tx,
            turn_count: Mutex::new(0),
            compliance: Mutex::new(ComplianceStatus::default()),
            stages_config: None, // No config-driven transitions in basic constructor
            ai_disclosure_message: ai_disclosure,
        }
    }

    /// Create a new conversation with config-driven intent detection
    ///
    /// P18 FIX: This constructor wires the domain-specific competitor patterns
    /// from config into the intent detector, replacing hardcoded defaults.
    pub fn from_view(
        session_id: impl Into<String>,
        config: ConversationConfig,
        view: &voice_agent_config::domain::AgentDomainView,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        let session_id_str = session_id.into();

        // Create agentic memory with config-driven compressor
        let agentic_config = AgenticMemoryConfig::default();
        let agentic_memory = AgenticMemory::from_view(agentic_config, &session_id_str, view);

        // Create intent detector with config-driven patterns
        let mut intent_detector = IntentDetector::new();

        // Wire competitor patterns from config
        // Note: We need to convert the owned Strings to &str references
        let competitor_patterns_owned = view.competitor_intent_patterns();
        if !competitor_patterns_owned.is_empty() {
            // Convert Vec<(&str, &str, String)> to Vec<(&str, &str, &str)>
            let patterns: Vec<(&str, &str, &str)> = competitor_patterns_owned
                .iter()
                .map(|(id, name, pattern)| (*id, *name, pattern.as_str()))
                .collect();
            intent_detector.add_competitor_patterns(patterns);
        }

        // P2.1 FIX: Wire quality tier patterns from config
        // This replaces hardcoded purity patterns (24K, 22K, etc.) with config-driven patterns
        let quality_patterns_owned = view.quality_tier_intent_patterns();
        if !quality_patterns_owned.is_empty() {
            let patterns: Vec<(&str, &str)> = quality_patterns_owned
                .iter()
                .map(|(value, pattern)| (value.as_str(), pattern.as_str()))
                .collect();
            intent_detector.add_variant_patterns(patterns);
        }

        // P2.1 FIX: Wire location patterns from config
        // This replaces hardcoded 9-city list with config-driven cities
        let location_pattern = view.location_intent_pattern();
        intent_detector.set_location_pattern(&location_pattern);

        // P16 FIX: Store stages config for config-driven intent transitions
        let stages_config = Arc::new(view.stages_config().clone());

        // P16 FIX: Load AI disclosure message from compliance config (RBI requirement)
        let ai_disclosure_message = view.ai_disclosure(&config.language).to_string();

        Self {
            session_id: session_id_str.clone(),
            config: config.clone(),
            start_time: Instant::now(),
            last_activity: Mutex::new(Instant::now()),
            state: Mutex::new(ConversationState::Active),
            stage_manager: Arc::new(StageManager::new()),
            memory: Arc::new(ConversationMemory::new(config.memory)),
            agentic_memory: Arc::new(agentic_memory),
            intent_detector: Arc::new(intent_detector),
            event_tx,
            turn_count: Mutex::new(0),
            compliance: Mutex::new(ComplianceStatus::default()),
            stages_config: Some(stages_config), // P16 FIX: Config-driven transitions
            ai_disclosure_message, // P16 FIX: Config-driven AI disclosure
        }
    }

    /// Subscribe to conversation events
    pub fn subscribe(&self) -> broadcast::Receiver<ConversationEvent> {
        self.event_tx.subscribe()
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get current state
    pub fn state(&self) -> ConversationState {
        *self.state.lock()
    }

    /// Get current stage
    pub fn stage(&self) -> ConversationStage {
        self.stage_manager.current()
    }

    /// Get duration
    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get turn count
    pub fn turn_count(&self) -> usize {
        *self.turn_count.lock()
    }

    /// Check if conversation is active
    pub fn is_active(&self) -> bool {
        *self.state.lock() == ConversationState::Active
    }

    /// Add user turn
    pub fn add_user_turn(&self, content: &str) -> Result<DetectedIntent, AgentError> {
        self.check_active()?;
        self.update_activity();

        // Create turn
        let turn = Turn {
            role: TurnRole::User,
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: None,
        };

        // Add to memory
        let mut entry = MemoryEntry::from(&turn);
        entry.stage = Some(self.stage().display_name().to_string());

        // Detect intent
        let detected = if self.config.intent_detection {
            self.intent_detector.detect(content)
        } else {
            DetectedIntent {
                intent: "unknown".to_string(),
                confidence: 0.0,
                slots: std::collections::HashMap::new(),
                alternatives: vec![],
            }
        };

        entry.intents = vec![detected.intent.clone()];

        // Extract and store entities
        for (key, slot) in &detected.slots {
            if let Some(ref value) = slot.value {
                entry.entities.insert(key.clone(), value.clone());
                self.memory.add_fact(key, value, slot.confidence);

                let _ = self.event_tx.send(ConversationEvent::FactLearned {
                    key: key.clone(),
                    value: value.clone(),
                });
            }
        }

        self.memory.add(entry);
        self.stage_manager.record_turn();
        *self.turn_count.lock() += 1;

        // Emit events
        let _ = self.event_tx.send(ConversationEvent::TurnAdded {
            role: TurnRole::User,
            content: content.to_string(),
        });

        let _ = self
            .event_tx
            .send(ConversationEvent::IntentDetected(detected.clone()));

        // Check for stage transition triggers
        self.check_stage_transitions(&detected);

        Ok(detected)
    }

    /// Add assistant turn
    pub fn add_assistant_turn(&self, content: &str) -> Result<(), AgentError> {
        self.check_active()?;
        self.update_activity();

        let turn = Turn {
            role: TurnRole::Assistant,
            content: content.to_string(),
            timestamp: Utc::now(),
            metadata: None,
        };

        let mut entry = MemoryEntry::from(&turn);
        entry.stage = Some(self.stage().display_name().to_string());

        self.memory.add(entry);
        *self.turn_count.lock() += 1;

        let _ = self.event_tx.send(ConversationEvent::TurnAdded {
            role: TurnRole::Assistant,
            content: content.to_string(),
        });

        Ok(())
    }

    /// Transition to a new stage
    pub fn transition_stage(&self, to: ConversationStage) -> Result<(), AgentError> {
        let from = self.stage();

        match self
            .stage_manager
            .transition(to, TransitionReason::NaturalFlow)
        {
            Ok(_) => {
                let _ = self
                    .event_tx
                    .send(ConversationEvent::StageChanged { from, to });
                Ok(())
            },
            Err(e) => Err(AgentError::Stage(e)),
        }
    }

    /// Check and perform stage transitions based on intent
    ///
    /// P16 FIX: Config-driven intent→stage transitions loaded from stages.yaml.
    /// Falls back to hardcoded transitions for backward compatibility.
    fn check_stage_transitions(&self, intent: &DetectedIntent) {
        let current = self.stage();
        let current_stage_str = current.as_str();
        let current_turns = self.stage_manager.current_stage_turns();

        // Record intent for stage requirement tracking
        self.stage_manager.record_intent(&intent.intent);

        // P21 FIX: Config-driven transitions only (deprecated fallback removed)
        let new_stage = if let Some(ref stages_config) = self.stages_config {
            // Config-driven: look up intent→stage transition from config
            stages_config
                .can_transition_on_intent(&intent.intent, current_stage_str, current_turns)
                .and_then(|target| ConversationStage::from_str(target))
        } else {
            // No config = no automatic transitions (config is required in production)
            tracing::warn!(
                "No stages_config provided - stage transitions disabled. \
                 Configure stages.yaml with intent_transitions for production use."
            );
            None
        };

        if let Some(to) = new_stage {
            // Validate transition is allowed
            let is_valid = if let Some(ref stages_config) = self.stages_config {
                stages_config.is_valid_transition(current_stage_str, to.as_str())
            } else {
                current.valid_transitions().contains(&to)
            };

            if is_valid {
                let _ = self
                    .stage_manager
                    .transition(to, TransitionReason::IntentDetected(intent.intent.clone()));
                let _ = self
                    .event_tx
                    .send(ConversationEvent::StageChanged { from: current, to });
            }
        }

        // Check if we should suggest next stage
        if let Some(suggested) = self.stage_manager.suggest_next() {
            // Don't auto-transition, just note it
            tracing::debug!("Suggested next stage: {:?}", suggested);
        }
    }

    /// Get memory context
    pub fn get_context(&self) -> String {
        self.memory.get_context()
    }

    /// Get memory context with query-based archival retrieval
    ///
    /// Phase 10: Enhanced context that includes relevant archival memories
    /// based on the user's query. This implements Anthropic-style selective
    /// context injection for better response quality.
    pub fn get_context_for_query(&self, query: &str, max_tokens: usize) -> String {
        self.agentic_memory.get_context_for_query(query, max_tokens)
    }

    /// Get access to agentic memory for archival operations
    pub fn agentic_memory(&self) -> &Arc<AgenticMemory> {
        &self.agentic_memory
    }

    /// Get recent messages for LLM
    pub fn get_messages(&self) -> Vec<(String, String)> {
        self.memory.get_recent_messages()
    }

    /// Get stage guidance
    pub fn get_stage_guidance(&self) -> &'static str {
        self.stage().guidance()
    }

    /// Get suggested questions
    pub fn get_suggested_questions(&self) -> Vec<&'static str> {
        self.stage().suggested_questions()
    }

    /// Record a fact
    pub fn record_fact(&self, key: &str, value: &str, confidence: f32) {
        self.memory.add_fact(key, value, confidence);
        self.stage_manager.record_info(key, value);

        let _ = self.event_tx.send(ConversationEvent::FactLearned {
            key: key.to_string(),
            value: value.to_string(),
        });
    }

    /// End the conversation
    pub fn end(&self, reason: EndReason) {
        *self.state.lock() = ConversationState::Ended;
        let _ = self.event_tx.send(ConversationEvent::Ended { reason });
    }

    /// Pause the conversation
    pub fn pause(&self) {
        *self.state.lock() = ConversationState::Paused;
    }

    /// Resume the conversation
    pub fn resume(&self) {
        let mut state = self.state.lock();
        if *state == ConversationState::Paused {
            *state = ConversationState::Active;
            self.update_activity();
        }
    }

    /// Check if active
    fn check_active(&self) -> Result<(), AgentError> {
        let state = *self.state.lock();
        match state {
            ConversationState::Active => {
                // Check timeouts
                if self.duration() > Duration::from_secs(self.config.max_duration_seconds as u64) {
                    self.end(EndReason::MaxDuration);
                    return Err(AgentError::Conversation(
                        "Max duration exceeded".to_string(),
                    ));
                }

                let last = *self.last_activity.lock();
                if last.elapsed() > Duration::from_secs(self.config.session_timeout_seconds as u64)
                {
                    self.end(EndReason::Timeout);
                    return Err(AgentError::Conversation("Session timeout".to_string()));
                }

                Ok(())
            },
            ConversationState::Paused => Err(AgentError::Conversation(
                "Conversation is paused".to_string(),
            )),
            ConversationState::Ended => Err(AgentError::Conversation(
                "Conversation has ended".to_string(),
            )),
        }
    }

    /// Update last activity time
    fn update_activity(&self) {
        *self.last_activity.lock() = Instant::now();
    }

    /// Get memory reference
    pub fn memory(&self) -> &ConversationMemory {
        &self.memory
    }

    /// P1 FIX: Get memory Arc for async operations
    pub fn memory_arc(&self) -> Arc<ConversationMemory> {
        Arc::clone(&self.memory)
    }

    /// Get stage manager reference
    pub fn stage_manager(&self) -> &StageManager {
        &self.stage_manager
    }

    // =========================================================================
    // P0 FIX: Compliance Methods (AI Disclosure, Consent)
    // =========================================================================

    /// Get compliance status
    pub fn compliance(&self) -> ComplianceStatus {
        self.compliance.lock().clone()
    }

    /// Check if AI disclosure has been given
    pub fn ai_disclosure_given(&self) -> bool {
        self.compliance.lock().ai_disclosure.given
    }

    /// Mark AI disclosure as given
    ///
    /// Should be called at the start of conversation after the greeting.
    /// Returns the disclosure message that should be spoken to the customer.
    ///
    /// P16 FIX: Uses config-driven AI disclosure message loaded at construction
    /// instead of hardcoded static messages.
    pub fn mark_ai_disclosed(&self) -> String {
        let language = &self.config.language;
        let disclosure_text = &self.ai_disclosure_message;

        let mut compliance = self.compliance.lock();
        compliance
            .ai_disclosure
            .mark_disclosed(language, disclosure_text, true);
        compliance.update();

        disclosure_text.clone()
    }

    /// Mark AI disclosure with custom text
    pub fn mark_ai_disclosed_with_text(&self, language: &str, text: &str, human_option: bool) {
        let mut compliance = self.compliance.lock();
        compliance
            .ai_disclosure
            .mark_disclosed(language, text, human_option);
        compliance.update();
    }

    /// Record recording consent
    pub fn record_recording_consent(&self, given: bool, method: ConsentMethod) {
        let mut compliance = self.compliance.lock();
        compliance.consent.record_recording_consent(given, method);
        compliance.consent.consent_language = self.config.language.clone();
        compliance.update();
    }

    /// Record PII processing consent
    pub fn record_pii_consent(&self, given: bool, method: ConsentMethod) {
        let mut compliance = self.compliance.lock();
        compliance.consent.record_pii_consent(given, method);
        compliance.update();
    }

    /// Record marketing consent
    pub fn record_marketing_consent(&self, given: Option<bool>) {
        let mut compliance = self.compliance.lock();
        compliance.consent.record_marketing_consent(given);
    }

    /// Check if conversation is compliant (AI disclosure given, minimum consent obtained)
    pub fn is_compliant(&self) -> bool {
        self.compliance.lock().compliant
    }

    /// Check if PII processing is allowed
    pub fn can_process_pii(&self) -> bool {
        self.compliance.lock().can_process_pii()
    }

    /// Get pending compliance requirements
    pub fn pending_compliance(&self) -> Vec<String> {
        self.compliance.lock().pending_requirements.clone()
    }

    /// Get AI disclosure message for the configured language
    ///
    /// P16 FIX: Returns the config-driven AI disclosure message loaded at construction.
    /// This replaces the hardcoded static message lookup with config from compliance.yaml.
    pub fn get_ai_disclosure_message(&self) -> String {
        self.ai_disclosure_message.clone()
    }
}

// =============================================================================
// Phase 2: ConversationContext Implementation for Conversation
// =============================================================================

impl ConversationContext for Conversation {
    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn state(&self) -> ConversationState {
        *self.state.lock()
    }

    fn stage(&self) -> ConversationStage {
        self.stage_manager.current()
    }

    fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn turn_count(&self) -> usize {
        *self.turn_count.lock()
    }

    fn is_active(&self) -> bool {
        *self.state.lock() == ConversationState::Active
    }

    fn add_user_turn(&self, content: &str) -> Result<DetectedIntent, AgentError> {
        Conversation::add_user_turn(self, content)
    }

    fn add_assistant_turn(&self, content: &str) -> Result<(), AgentError> {
        Conversation::add_assistant_turn(self, content)
    }

    fn transition_stage(&self, to: ConversationStage) -> Result<(), AgentError> {
        Conversation::transition_stage(self, to)
    }

    fn get_context(&self) -> String {
        self.memory.get_context()
    }

    fn get_context_for_query(&self, query: &str, max_tokens: usize) -> String {
        self.agentic_memory.get_context_for_query(query, max_tokens)
    }

    fn agentic_memory(&self) -> &Arc<AgenticMemory> {
        &self.agentic_memory
    }

    fn memory(&self) -> &ConversationMemory {
        &self.memory
    }

    fn memory_arc(&self) -> Arc<ConversationMemory> {
        Arc::clone(&self.memory)
    }

    fn stage_manager(&self) -> &StageManager {
        &self.stage_manager
    }

    fn record_fact(&self, key: &str, value: &str, confidence: f32) {
        Conversation::record_fact(self, key, value, confidence)
    }

    fn get_messages(&self) -> Vec<(String, String)> {
        self.memory.get_recent_messages()
    }

    fn get_stage_guidance(&self) -> &'static str {
        self.stage().guidance()
    }

    fn get_suggested_questions(&self) -> Vec<&'static str> {
        self.stage().suggested_questions()
    }

    fn end(&self, reason: EndReason) {
        Conversation::end(self, reason)
    }

    fn pause(&self) {
        *self.state.lock() = ConversationState::Paused;
    }

    fn resume(&self) {
        let mut state = self.state.lock();
        if *state == ConversationState::Paused {
            *state = ConversationState::Active;
            *self.last_activity.lock() = Instant::now();
        }
    }

    fn compliance(&self) -> ComplianceStatus {
        self.compliance.lock().clone()
    }

    fn ai_disclosure_given(&self) -> bool {
        self.compliance.lock().ai_disclosure.given
    }

    fn mark_ai_disclosed(&self) -> String {
        Conversation::mark_ai_disclosed(self)
    }

    fn record_recording_consent(&self, given: bool, method: ConsentMethod) {
        Conversation::record_recording_consent(self, given, method)
    }

    fn record_pii_consent(&self, given: bool, method: ConsentMethod) {
        Conversation::record_pii_consent(self, given, method)
    }

    fn is_compliant(&self) -> bool {
        self.compliance.lock().compliant
    }

    fn can_process_pii(&self) -> bool {
        self.compliance.lock().can_process_pii()
    }

    fn pending_compliance(&self) -> Vec<String> {
        self.compliance.lock().pending_requirements.clone()
    }

    fn get_ai_disclosure_message(&self) -> String {
        Conversation::get_ai_disclosure_message(self)
    }

    fn subscribe(&self) -> broadcast::Receiver<ConversationEvent> {
        self.event_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_creation() {
        let conv = Conversation::new("test-session", ConversationConfig::default());

        assert_eq!(conv.session_id(), "test-session");
        assert!(conv.is_active());
        assert_eq!(conv.stage(), ConversationStage::Greeting);
    }

    #[test]
    fn test_add_turns() {
        let conv = Conversation::new("test", ConversationConfig::default());

        let intent = conv.add_user_turn("Hello").unwrap();
        assert_eq!(intent.intent, "greeting");

        conv.add_assistant_turn("Hello! How can I help you?")
            .unwrap();

        assert_eq!(conv.turn_count(), 2);
    }

    #[test]
    fn test_stage_transition() {
        let conv = Conversation::new("test", ConversationConfig::default());

        conv.transition_stage(ConversationStage::Discovery).unwrap();
        assert_eq!(conv.stage(), ConversationStage::Discovery);
    }

    #[test]
    fn test_fact_recording() {
        let conv = Conversation::new("test", ConversationConfig::default());

        conv.record_fact("customer_name", "Rajesh", 0.9);

        let fact = conv.memory().get_fact("customer_name");
        assert!(fact.is_some());
        assert_eq!(fact.unwrap().value, "Rajesh");
    }
}
