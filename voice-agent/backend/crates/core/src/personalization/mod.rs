//! Personalization Engine
//!
//! Comprehensive personalization system for adapting agent responses:
//! - Persona management (tone, warmth, style)
//! - Segment-aware response adaptation
//! - Behavior signal detection
//! - Dynamic response customization
//!
//! # Architecture
//!
//! ```text
//! CustomerProfile → SegmentAdapter → Persona Selection
//!                                           ↓
//! User Input → SignalDetector → PersonalizationEngine → Response Adaptation
//!                                           ↓
//!                              System Prompt + Context
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use voice_agent_core::personalization::*;
//! use voice_agent_core::{CustomerProfile, CustomerSegment};
//!
//! // Create engine with config-driven adapter
//! let adapter = SegmentAdapter::from_config(config);
//! let engine = PersonalizationEngine::with_adapter(adapter);
//!
//! // Create profile
//! let profile = CustomerProfile::new()
//!     .segment(CustomerSegment::TrustSeeker);
//!
//! // Create context
//! let mut ctx = PersonalizationContext::for_profile(&profile);
//!
//! // Detect signals and adapt
//! if let Some(signal) = engine.detect_signal("Is my asset safe?") {
//!     ctx = ctx.with_signal(signal);
//! }
//!
//! // Get personalized instructions
//! let instructions = engine.generate_instructions(&ctx);
//! ```

pub mod adaptation;
pub mod persona;
pub mod signals;

// Export config-driven types
pub use adaptation::{
    feature_ids, objection_ids, parse_segment_id, segment_to_id, FeatureId, ObjectionId,
    ObjectionResponse, ObjectionResponseConfig, SegmentAdapter, SegmentAdapterConfig,
};
pub use persona::{LanguageComplexity, Persona, PersonaTemplates, ResponseUrgency, Tone};
pub use signals::{
    BehaviorSignal, SignalDetection, SignalDetector, SignalDetectorConfig, TrendAnalysis,
};

use crate::{CustomerProfile, CustomerSegment};
use serde::{Deserialize, Serialize};

/// Personalization context for a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizationContext {
    /// Customer segment
    pub segment: Option<CustomerSegment>,
    /// Active persona
    pub persona: Persona,
    /// Detected behavior signals
    pub signals: Vec<BehaviorSignal>,
    /// Current sentiment (-1.0 to 1.0)
    pub sentiment: f32,
    /// Conversation turn count
    pub turn_count: usize,
    /// Whether objection was detected
    pub has_objection: bool,
    /// Detected objection ID (config-driven)
    pub current_objection_id: Option<ObjectionId>,
    /// Customer name for personalization
    pub customer_name: Option<String>,
    /// Preferred language
    pub preferred_language: String,
}

impl PersonalizationContext {
    /// Create context for a customer profile with config-driven segment detection.
    ///
    /// # Arguments
    /// * `profile` - Customer profile data
    pub fn for_profile(profile: &CustomerProfile) -> Self {
        Self::for_profile_with_segment(profile, None)
    }

    /// Create context with explicit segment override
    pub fn for_profile_with_segment(
        profile: &CustomerProfile,
        segment_override: Option<CustomerSegment>,
    ) -> Self {
        let segment = segment_override.or_else(|| profile.infer_segment());
        let persona = segment.map(Persona::for_segment).unwrap_or_default();

        Self {
            segment,
            persona,
            signals: Vec::new(),
            sentiment: 0.0,
            turn_count: 0,
            has_objection: false,
            current_objection_id: None,
            customer_name: profile.name.clone(),
            preferred_language: profile.preferred_language.clone(),
        }
    }

    /// Create default context
    pub fn new() -> Self {
        Self {
            segment: None,
            persona: Persona::default(),
            signals: Vec::new(),
            sentiment: 0.0,
            turn_count: 0,
            has_objection: false,
            current_objection_id: None,
            customer_name: None,
            preferred_language: "en".to_string(),
        }
    }

    /// Set segment
    pub fn with_segment(mut self, segment: CustomerSegment) -> Self {
        self.segment = Some(segment);
        self.persona = Persona::for_segment(segment);
        self
    }

    /// Set persona
    pub fn with_persona(mut self, persona: Persona) -> Self {
        self.persona = persona;
        self
    }

    /// Add behavior signal
    pub fn with_signal(mut self, signal: BehaviorSignal) -> Self {
        self.signals.push(signal);
        self
    }

    /// Set customer name
    pub fn with_customer_name(mut self, name: impl Into<String>) -> Self {
        self.customer_name = Some(name.into());
        self
    }

    /// Update from signal detection
    pub fn update_from_detection(&mut self, detection: &SignalDetection) {
        self.signals.push(detection.primary);
        self.sentiment = (self.sentiment + detection.sentiment()) / 2.0;

        for (signal, _) in &detection.secondary {
            self.signals.push(*signal);
        }
    }

    /// Record objection by ID (config-driven)
    pub fn record_objection(&mut self, objection_id: impl Into<String>) {
        self.has_objection = true;
        self.current_objection_id = Some(objection_id.into());
    }

    /// Clear objection after handling
    pub fn clear_objection(&mut self) {
        self.current_objection_id = None;
    }

    /// Increment turn count
    pub fn next_turn(&mut self) {
        self.turn_count += 1;
    }

    /// Get recent signals (last N)
    pub fn recent_signals(&self, n: usize) -> &[BehaviorSignal] {
        let start = self.signals.len().saturating_sub(n);
        &self.signals[start..]
    }

    /// Check if sentiment is positive
    pub fn is_positive(&self) -> bool {
        self.sentiment > 0.2
    }

    /// Check if sentiment is negative
    pub fn is_negative(&self) -> bool {
        self.sentiment < -0.2
    }

    /// Get segment ID string
    pub fn segment_id(&self) -> Option<String> {
        self.segment.map(segment_to_id)
    }
}

impl Default for PersonalizationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Main personalization engine
pub struct PersonalizationEngine {
    /// Signal detector
    signal_detector: SignalDetector,
    /// Segment adapter (config-driven)
    segment_adapter: SegmentAdapter,
    /// Enable adaptive persona
    adaptive_persona: bool,
}

impl PersonalizationEngine {
    /// Create a new personalization engine with empty adapter
    ///
    /// For production, use `with_adapter()` to inject a config-driven adapter.
    pub fn new() -> Self {
        Self {
            signal_detector: SignalDetector::new(),
            segment_adapter: SegmentAdapter::empty(),
            adaptive_persona: true,
        }
    }

    /// Create with a config-driven segment adapter
    pub fn with_adapter(adapter: SegmentAdapter) -> Self {
        Self {
            signal_detector: SignalDetector::new(),
            segment_adapter: adapter,
            adaptive_persona: true,
        }
    }

    /// Create with custom signal detector
    pub fn with_signal_detector(mut self, detector: SignalDetector) -> Self {
        self.signal_detector = detector;
        self
    }

    /// Create with custom segment adapter
    pub fn with_segment_adapter(mut self, adapter: SegmentAdapter) -> Self {
        self.segment_adapter = adapter;
        self
    }

    /// Enable/disable adaptive persona
    pub fn with_adaptive_persona(mut self, enabled: bool) -> Self {
        self.adaptive_persona = enabled;
        self
    }

    /// Detect signal from user input
    pub fn detect_signal(&self, text: &str) -> Option<SignalDetection> {
        self.signal_detector.detect(text)
    }

    /// Detect signal with timing information
    pub fn detect_signal_with_timing(
        &self,
        text: &str,
        pause_ms: Option<u64>,
        speech_rate: Option<f32>,
    ) -> Option<SignalDetection> {
        self.signal_detector
            .detect_with_timing(text, pause_ms, speech_rate)
    }

    /// Handle objection and return response (config-driven)
    pub fn handle_objection(
        &self,
        ctx: &PersonalizationContext,
        objection_id: &str,
    ) -> Option<String> {
        let segment_id = ctx.segment_id().unwrap_or_else(|| "first_time".to_string());
        self.segment_adapter
            .handle_objection(&segment_id, objection_id, ctx.customer_name.as_deref())
    }

    /// Get priority feature IDs for segment
    pub fn get_feature_ids(&self, segment: CustomerSegment) -> Vec<String> {
        self.segment_adapter.get_features_for_segment(segment)
    }

    /// Get value propositions for segment
    pub fn get_value_propositions(&self, segment: CustomerSegment) -> Vec<String> {
        self.segment_adapter.get_value_propositions_for_segment(segment)
    }

    /// Analyze conversation trend
    pub fn analyze_trend(&self, ctx: &PersonalizationContext) -> TrendAnalysis {
        self.signal_detector.analyze_trend(&ctx.signals)
    }

    /// Get adapted persona based on context
    pub fn get_adapted_persona(&self, ctx: &PersonalizationContext) -> Persona {
        if !self.adaptive_persona {
            return ctx.persona.clone();
        }

        let mut persona = ctx.persona.clone();

        // Adjust based on recent signals
        let recent = ctx.recent_signals(3);

        // Increase empathy if frustration or confusion detected
        if recent
            .iter()
            .any(|s| matches!(s, BehaviorSignal::Frustration | BehaviorSignal::Confusion))
        {
            persona.empathy = (persona.empathy + 0.2).min(1.0);
            persona.warmth = (persona.warmth + 0.1).min(1.0);
        }

        // Increase urgency if interest detected
        if recent.iter().any(|s| {
            matches!(
                s,
                BehaviorSignal::StrongInterest | BehaviorSignal::Commitment
            )
        }) {
            persona.urgency = ResponseUrgency::Efficient;
        }

        // Slow down if hesitation detected
        if recent
            .iter()
            .filter(|s| matches!(s, BehaviorSignal::Hesitation))
            .count()
            >= 2
        {
            persona.urgency = ResponseUrgency::Relaxed;
        }

        // If exit intent, maximize empathy
        if recent
            .iter()
            .any(|s| matches!(s, BehaviorSignal::ExitIntent))
        {
            persona.empathy = 0.95;
            persona.warmth = 0.9;
            persona.acknowledge_emotions = true;
        }

        persona
    }

    /// Generate system prompt instructions
    ///
    /// For feature emphasis, use `generate_instructions_with_features()` to provide
    /// feature display names from your FeatureProvider.
    pub fn generate_instructions(&self, ctx: &PersonalizationContext) -> String {
        self.generate_instructions_with_features(ctx, &[])
    }

    /// Generate system prompt instructions with feature emphasis
    ///
    /// # Arguments
    /// * `ctx` - Personalization context
    /// * `feature_names` - Feature display names to emphasize (from FeatureProvider)
    pub fn generate_instructions_with_features(
        &self,
        ctx: &PersonalizationContext,
        feature_names: &[String],
    ) -> String {
        let persona = self.get_adapted_persona(ctx);
        let mut instructions = persona.system_prompt_instructions();

        // Add feature emphasis if provided
        if !feature_names.is_empty() {
            instructions.push_str(&format!(
                " Emphasize these features: {}.",
                feature_names.join(", ")
            ));
        }

        // Add signal-based guidance
        let trend = self.analyze_trend(ctx);
        instructions.push_str(&format!(
            " Current strategy: {}",
            trend.recommended_action()
        ));

        // Add customer name guidance
        if let Some(ref name) = ctx.customer_name {
            instructions.push_str(&format!(" Customer name is {}.", name));
        }

        // Add language guidance
        if ctx.preferred_language != "en" {
            instructions.push_str(&format!(
                " Customer prefers {}. Use code-switching if appropriate.",
                ctx.preferred_language
            ));
        }

        // Add objection guidance
        if ctx.has_objection {
            instructions.push_str(
                " Customer has raised an objection. Address it empathetically before proceeding.",
            );
        }

        instructions
    }

    /// Process user input and update context
    pub fn process_input(&self, ctx: &mut PersonalizationContext, text: &str) {
        ctx.next_turn();

        // Detect signals
        if let Some(detection) = self.detect_signal(text) {
            ctx.update_from_detection(&detection);
        }
    }

    /// Get segment adapter
    pub fn segment_adapter(&self) -> &SegmentAdapter {
        &self.segment_adapter
    }

    /// Get mutable segment adapter
    pub fn segment_adapter_mut(&mut self) -> &mut SegmentAdapter {
        &mut self.segment_adapter
    }

    /// Get signal detector
    pub fn signal_detector(&self) -> &SignalDetector {
        &self.signal_detector
    }
}

impl Default for PersonalizationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_from_profile() {
        // Create a profile with a current lender (triggers TrustSeeker)
        let profile = CustomerProfile::new()
            .name("Raj Kumar")
            .current_lender("some_provider");

        let ctx = PersonalizationContext::for_profile(&profile);

        assert_eq!(ctx.customer_name, Some("Raj Kumar".to_string()));
        assert_eq!(ctx.segment, Some(CustomerSegment::TrustSeeker));
    }

    #[test]
    fn test_context_signals() {
        let mut ctx = PersonalizationContext::new().with_signal(BehaviorSignal::Interest);

        ctx.update_from_detection(&SignalDetection::new(BehaviorSignal::StrongInterest, 0.9));

        assert_eq!(ctx.signals.len(), 2);
        assert!(ctx.sentiment > 0.0);
    }

    #[test]
    fn test_engine_detect_signal() {
        let engine = PersonalizationEngine::new();

        let detection = engine.detect_signal("Tell me more about this").unwrap();
        assert_eq!(detection.primary, BehaviorSignal::Interest);
    }

    #[test]
    fn test_adapted_persona() {
        let engine = PersonalizationEngine::new();

        // Context with frustration
        let ctx = PersonalizationContext::new()
            .with_segment(CustomerSegment::Professional)
            .with_signal(BehaviorSignal::Frustration);

        let persona = engine.get_adapted_persona(&ctx);
        // Empathy should be increased
        assert!(persona.empathy > Persona::for_segment(CustomerSegment::Professional).empathy);
    }

    #[test]
    fn test_generate_instructions() {
        let engine = PersonalizationEngine::new();

        let ctx = PersonalizationContext::new()
            .with_segment(CustomerSegment::TrustSeeker)
            .with_customer_name("Priya");

        let instructions = engine.generate_instructions(&ctx);

        // Should contain customer name
        assert!(instructions.contains("Priya"));
        // Should contain some guidance
        assert!(instructions.len() > 10);
    }

    #[test]
    fn test_generate_instructions_with_features() {
        let engine = PersonalizationEngine::new();

        let ctx = PersonalizationContext::new()
            .with_segment(CustomerSegment::TrustSeeker)
            .with_customer_name("Priya");

        let features = vec!["Security".to_string(), "Transparency".to_string()];
        let instructions = engine.generate_instructions_with_features(&ctx, &features);

        // Should contain feature names
        assert!(instructions.contains("Security"));
        assert!(instructions.contains("Transparency"));
    }

    #[test]
    fn test_process_input() {
        let engine = PersonalizationEngine::new();
        let mut ctx = PersonalizationContext::new();

        engine.process_input(&mut ctx, "Tell me more about this, sounds interesting");

        assert_eq!(ctx.turn_count, 1);
        assert!(!ctx.signals.is_empty());
    }

    #[test]
    fn test_trend_analysis() {
        let engine = PersonalizationEngine::new();
        let ctx = PersonalizationContext::new()
            .with_signal(BehaviorSignal::Interest)
            .with_signal(BehaviorSignal::StrongInterest);

        let trend = engine.analyze_trend(&ctx);
        assert_eq!(trend, TrendAnalysis::Improving);
    }

    #[test]
    fn test_config_driven_features() {
        // Test config-driven segment adapter
        let mut config = SegmentAdapterConfig::default();
        config.segment_features.insert(
            "high_value".to_string(),
            vec!["relationship_manager".to_string(), "higher_limits".to_string()],
        );
        config.value_propositions.insert(
            "high_value".to_string(),
            vec!["Exclusive rates".to_string()],
        );

        let adapter = SegmentAdapter::from_config(config);
        let engine = PersonalizationEngine::with_adapter(adapter);

        let features = engine.get_feature_ids(CustomerSegment::HighValue);
        assert_eq!(features.len(), 2);
        assert!(features.contains(&"relationship_manager".to_string()));
    }

    #[test]
    fn test_record_objection() {
        let mut ctx = PersonalizationContext::new();
        assert!(!ctx.has_objection);
        assert!(ctx.current_objection_id.is_none());

        ctx.record_objection("collateral_safety");

        assert!(ctx.has_objection);
        assert_eq!(ctx.current_objection_id, Some("collateral_safety".to_string()));

        ctx.clear_objection();
        assert!(ctx.current_objection_id.is_none());
    }

    #[test]
    fn test_segment_id() {
        let ctx = PersonalizationContext::new().with_segment(CustomerSegment::HighValue);
        assert_eq!(ctx.segment_id(), Some("high_value".to_string()));

        let ctx_no_segment = PersonalizationContext::new();
        assert_eq!(ctx_no_segment.segment_id(), None);
    }

    #[test]
    fn test_handle_objection_config_driven() {
        let mut config = SegmentAdapterConfig::default();
        config.segment_features.insert("trust_seeker".to_string(), vec![]);
        config.value_propositions.insert("trust_seeker".to_string(), vec![]);
        config.objection_responses.insert(
            "safety".to_string(),
            ObjectionResponseConfig {
                segment: "trust_seeker".to_string(),
                acknowledgment: "I understand your concern.".to_string(),
                response: "Your assets are fully protected.".to_string(),
                follow_up: Some("Would you like details?".to_string()),
                highlight_feature: "security".to_string(),
            },
        );

        let adapter = SegmentAdapter::from_config(config);
        let engine = PersonalizationEngine::with_adapter(adapter);

        let ctx = PersonalizationContext::new()
            .with_segment(CustomerSegment::TrustSeeker)
            .with_customer_name("Priya");

        let response = engine.handle_objection(&ctx, "safety");
        assert!(response.is_some());
        let response = response.unwrap();
        assert!(response.contains("I understand your concern."));
        assert!(response.contains("Priya"));
    }
}
