//! Crate-Specific Domain Views
//!
//! Each crate accesses domain configuration through a "view" that provides
//! only the information that crate needs, in terminology appropriate for that crate.

use std::collections::HashMap;
use std::sync::Arc;

use super::branches::{BranchEntry, BranchesConfig};
use super::competitors::{CompetitorEntry as ExtCompetitorEntry, CompetitorsConfig};
use super::objections::{ObjectionResponse, ObjectionsConfig};
use super::prompts::PromptsConfig;
use super::scoring::{CategoryWeights, EscalationConfig, ScoringConfig};
use super::segments::{SegmentDefinition, SegmentsConfig};
use super::slots::{GoalDefinition, SlotDefinition, SlotsConfig};
use super::sms_templates::SmsTemplatesConfig;
use super::stages::{StageDefinition, StagesConfig, TransitionTrigger};
use super::tools::{ToolSchema, ToolsConfig};
use super::{
    MasterDomainConfig, MemoryCompressorConfig, CurrencyConfig,
};

/// View for the agent crate
/// Provides access to conversation stages, DST slots, scoring, objections
pub struct AgentDomainView {
    config: Arc<MasterDomainConfig>,
}

impl AgentDomainView {
    pub fn new(config: Arc<MasterDomainConfig>) -> Self {
        Self { config }
    }

    // ====== Brand Information ======

    /// P13 FIX: Get bank name for persona goal
    pub fn company_name(&self) -> &str {
        &self.config.brand.company_name
    }

    /// P13 FIX: Get agent name
    pub fn agent_name(&self) -> &str {
        &self.config.brand.agent_name
    }

    /// P13 FIX: Get agent role for persona goal (e.g., "Gold Loan Advisor")
    /// Falls back to domain display name if not set
    pub fn agent_role(&self) -> &str {
        if self.config.brand.agent_role.is_empty() {
            // Default fallback based on domain
            "Advisor"
        } else {
            &self.config.brand.agent_role
        }
    }

    /// Get helpline number
    pub fn helpline(&self) -> &str {
        &self.config.brand.helpline
    }

    // ====== Prompts Configuration ======

    /// Get the prompts configuration
    pub fn prompts_config(&self) -> &PromptsConfig {
        &self.config.prompts
    }

    // ====== DST Instructions ======

    /// P13 FIX: Get DST instruction for an action type
    /// Falls back to English if the language-specific instruction is not found
    pub fn dst_instruction(&self, action_type: &str, language: &str) -> Option<&str> {
        self.config.prompts.dst_instruction(action_type, language)
    }

    // ====== High-Value Customer Detection ======

    /// Get high-value thresholds for lead scoring
    pub fn high_value_amount_threshold(&self) -> f64 {
        self.config.high_value.amount_threshold
    }

    pub fn high_value_weight_threshold(&self) -> f64 {
        self.config.high_value.weight_threshold_grams
    }

    /// Check if signals indicate high-value customer
    pub fn is_high_value(&self, amount: Option<f64>, weight: Option<f64>) -> bool {
        self.config.is_high_value(amount, weight)
    }

    /// Get high-value features to highlight
    pub fn high_value_features(&self) -> &[String] {
        &self.config.high_value.features
    }

    /// Get competitor by name for comparison
    pub fn get_competitor_rate(&self, name: &str) -> Option<f64> {
        self.config.get_competitor(name).map(|c| c.typical_rate)
    }

    /// Get our rate for comparison
    pub fn our_rate_for_amount(&self, amount: f64) -> f64 {
        self.config.get_rate_for_amount(amount)
    }

    // ====== Slot Configuration ======

    /// Get the full slots configuration
    pub fn slots_config(&self) -> &SlotsConfig {
        &self.config.slots
    }

    /// Get a slot definition by name
    pub fn get_slot(&self, name: &str) -> Option<&SlotDefinition> {
        self.config.slots.get_slot(name)
    }

    /// P19 FIX: Get display label for a slot (e.g., "asset_quantity" -> "Gold Weight")
    pub fn get_slot_display_label(&self, slot_name: &str) -> String {
        self.config.slots.get_slot_display_label(slot_name)
    }

    /// P21 FIX: Get all slot names defined in config
    pub fn all_slot_names(&self) -> Vec<&str> {
        self.config.slots.all_slot_names()
    }

    /// P21 FIX: Get all slot display labels as a HashMap
    /// Useful for building display mappings without hardcoding slot names
    pub fn all_slot_display_labels(&self) -> std::collections::HashMap<String, String> {
        self.config.slots.all_slot_display_labels().into_iter().collect()
    }

    /// Get a goal definition by name
    pub fn get_goal(&self, name: &str) -> Option<&GoalDefinition> {
        self.config.slots.get_goal(name)
    }

    /// Map an intent to a goal
    pub fn goal_for_intent(&self, intent: &str) -> Option<&str> {
        self.config.slots.goal_for_intent(intent)
    }

    /// Get extraction patterns for a slot in a specific language
    pub fn extraction_patterns(&self, slot_name: &str, language: &str) -> Vec<&str> {
        self.config.slots.extraction_patterns(slot_name, language)
    }

    /// Get quality factor for a quality tier
    ///
    /// # Arguments
    /// * `slot_name` - The slot name (e.g., "asset_quality_tier")
    /// * `tier_id` - The tier ID (e.g., "tier_1")
    pub fn quality_factor(&self, slot_name: &str, tier_id: &str) -> f64 {
        self.config.slots.quality_factor(slot_name, tier_id)
    }

    /// Get typical rate for a lender (from slot enum values)
    pub fn lender_rate(&self, lender_id: &str) -> Option<f64> {
        self.config.slots.lender_rate(lender_id)
    }

    /// Get unit conversion factor (e.g., tola -> grams)
    pub fn unit_conversion(&self, slot_name: &str, unit: &str) -> Option<f64> {
        self.config.slots.unit_conversion(slot_name, unit)
    }

    /// Get required slots for a goal
    pub fn required_slots_for_goal(&self, goal_name: &str) -> Vec<&str> {
        self.config
            .slots
            .get_goal(goal_name)
            .map(|g| g.required_slots.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get optional slots for a goal
    pub fn optional_slots_for_goal(&self, goal_name: &str) -> Vec<&str> {
        self.config
            .slots
            .get_goal(goal_name)
            .map(|g| g.optional_slots.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get completion action for a goal
    pub fn completion_action_for_goal(&self, goal_name: &str) -> Option<&str> {
        self.config
            .slots
            .get_goal(goal_name)
            .and_then(|g| g.completion_action.as_deref())
    }

    // ====== Stage Configuration ======

    /// Get the full stages configuration
    pub fn stages_config(&self) -> &StagesConfig {
        &self.config.stages
    }

    /// Get a stage definition by ID
    pub fn get_stage(&self, stage_id: &str) -> Option<&StageDefinition> {
        self.config.stages.get_stage(stage_id)
    }

    /// Get the initial stage ID
    pub fn initial_stage_id(&self) -> &str {
        &self.config.stages.initial_stage
    }

    /// Get the initial stage definition
    pub fn get_initial_stage(&self) -> Option<&StageDefinition> {
        self.config.stages.get_initial_stage()
    }

    /// Get valid transitions from a stage
    pub fn stage_transitions(&self, stage_id: &str) -> Vec<&str> {
        self.config.stages.get_transitions(stage_id)
    }

    /// Check if a transition is valid
    pub fn is_valid_stage_transition(&self, from: &str, to: &str) -> bool {
        self.config.stages.is_valid_transition(from, to)
    }

    /// Get stage guidance text
    pub fn stage_guidance(&self, stage_id: &str) -> Option<&str> {
        self.config.stages.get_guidance(stage_id)
    }

    /// Get suggested questions for a stage
    pub fn stage_questions(&self, stage_id: &str) -> Vec<&str> {
        self.config.stages.get_suggested_questions(stage_id)
    }

    /// Get context budget for a stage (in tokens)
    pub fn stage_context_budget(&self, stage_id: &str) -> usize {
        self.config.stages.get_context_budget(stage_id)
    }

    /// Get RAG context fraction for a stage (0.0-1.0)
    pub fn stage_rag_fraction(&self, stage_id: &str) -> f32 {
        self.config.stages.get_rag_fraction(stage_id)
    }

    /// Get transition trigger for a stage
    pub fn stage_trigger(&self, stage_id: &str) -> Option<&TransitionTrigger> {
        self.config.stages.get_trigger(stage_id)
    }

    /// Get all stage IDs
    pub fn all_stage_ids(&self) -> Vec<&str> {
        self.config.stages.stage_ids()
    }

    /// P16 FIX: Get intent-based transition target stage
    ///
    /// Returns the target stage ID for an intent + current stage combination,
    /// if defined in config and the min_turns requirement is satisfied.
    pub fn get_intent_transition(
        &self,
        intent: &str,
        current_stage: &str,
        current_turns: usize,
    ) -> Option<&str> {
        self.config
            .stages
            .can_transition_on_intent(intent, current_stage, current_turns)
    }

    // ====== Lead Scoring Configuration ======

    /// Get the full scoring configuration
    pub fn scoring_config(&self) -> &ScoringConfig {
        &self.config.scoring
    }

    /// Get qualification level from score
    pub fn qualification_level(&self, score: u32) -> &'static str {
        self.config.scoring.qualification_level(score)
    }

    /// Get escalation configuration
    pub fn escalation_config(&self) -> &EscalationConfig {
        &self.config.scoring.escalation
    }

    /// Get max objections before escalation
    pub fn max_objections_before_escalate(&self) -> u32 {
        self.config.scoring.escalation.max_objections
    }

    /// Get max stalled turns before escalation
    pub fn max_stalled_turns(&self) -> u32 {
        self.config.scoring.escalation.max_stalled_turns
    }

    /// Get high-value loan threshold
    pub fn high_value_loan_threshold(&self) -> f64 {
        self.config.scoring.escalation.high_value_threshold
    }

    /// Get category weights for scoring
    pub fn scoring_weights(&self) -> &CategoryWeights {
        &self.config.scoring.weights
    }

    /// Get urgency keywords for a language
    pub fn urgency_keywords(&self, language: &str) -> Vec<&str> {
        self.config.scoring.urgency_keywords(language)
    }

    /// Get trust score for a level
    pub fn trust_score(&self, level: &str) -> u32 {
        self.config.scoring.trust_score(level)
    }

    /// Get qualification thresholds
    pub fn qualification_thresholds(&self) -> (u32, u32, u32, u32) {
        let t = &self.config.scoring.qualification_thresholds;
        (t.cold, t.warm, t.hot, t.qualified)
    }

    // ====== Objection Handling Configuration ======

    /// Get the full objections configuration
    pub fn objections_config(&self) -> &ObjectionsConfig {
        &self.config.objections
    }

    /// Detect objection type from text
    pub fn detect_objection(&self, text: &str, language: &str) -> Option<&str> {
        self.config.objections.detect_objection(text, language)
    }

    /// Get objection response for a type and language
    pub fn objection_response(&self, objection_type: &str, language: &str) -> Option<&ObjectionResponse> {
        self.config.objections.get_response(objection_type, language)
    }

    /// Get default objection response for unrecognized concerns
    pub fn default_objection_response(&self, language: &str) -> Option<&ObjectionResponse> {
        self.config.objections.get_default_response(language)
    }

    /// Get all objection type names
    pub fn objection_types(&self) -> Vec<&str> {
        self.config.objections.objection_types()
    }

    /// Build full response text for an objection
    pub fn build_objection_response(&self, objection_type: &str, language: &str) -> Option<String> {
        self.config.objections.build_full_response(objection_type, language)
    }

    // ====== Feature Configuration ======

    /// Get display name for a feature in a specific language
    pub fn feature_display_name(&self, feature_id: &str, language: &str) -> Option<&str> {
        self.config.features.display_name(feature_id, language)
    }

    /// Get features for a segment (from features config)
    pub fn features_for_segment(&self, segment_id: &str) -> Vec<&str> {
        self.config.features.features_for_segment(segment_id)
    }

    /// Get top N features for a segment
    pub fn top_features_for_segment(&self, segment_id: &str, n: usize) -> Vec<&str> {
        self.config.features.top_features_for_segment(segment_id, n)
    }

    /// Get value propositions for a segment (from features config)
    pub fn value_propositions_for_segment(&self, segment_id: &str) -> Vec<&str> {
        self.config.features.value_propositions_for_segment(segment_id)
    }

    /// Get value propositions with rate substitution
    pub fn value_propositions_with_rate(&self, segment_id: &str, rate: f64) -> Vec<String> {
        self.config.features.value_propositions_with_rate(segment_id, rate)
    }

    /// Check if a feature exists
    pub fn has_feature(&self, feature_id: &str) -> bool {
        self.config.features.has_feature(feature_id)
    }

    /// Get all feature IDs
    pub fn all_feature_ids(&self) -> Vec<&str> {
        self.config.features.feature_ids()
    }

    // ====== Customer Segment Configuration ======

    /// Get the full segments configuration
    pub fn segments_config(&self) -> &SegmentsConfig {
        &self.config.segments
    }

    /// Get a segment definition by ID
    pub fn get_segment(&self, segment_id: &str) -> Option<&SegmentDefinition> {
        self.config.segments.get_segment(segment_id)
    }

    /// Detect customer segments from text and signals
    pub fn detect_segments(
        &self,
        text: &str,
        language: &str,
        numeric_values: &HashMap<String, f64>,
        text_values: &HashMap<String, String>,
    ) -> Vec<&str> {
        self.config.segments.detect_segments(text, language, numeric_values, text_values)
    }

    /// Get value propositions for a segment
    pub fn segment_value_props(&self, segment_id: &str, language: &str) -> Vec<&str> {
        self.config.segments.get_value_props(segment_id, language)
    }

    /// Get features to highlight for a segment
    pub fn segment_features(&self, segment_id: &str) -> Vec<&str> {
        self.config.segments.get_features(segment_id)
    }

    /// Get the default segment ID
    pub fn default_segment(&self) -> &str {
        &self.config.segments.default_segment
    }

    /// P25 FIX: Get persona config for a segment
    ///
    /// Returns the embedded SegmentPersonaConfig for config-driven persona creation.
    /// This replaces the hardcoded Persona::for_segment() match statement.
    pub fn persona_config_for_segment(&self, segment_id: &str) -> Option<voice_agent_core::traits::PersonaConfig> {
        self.config.segments.get_persona_config(segment_id).map(|seg_persona| {
            voice_agent_core::traits::PersonaConfig {
                name: seg_persona.name.clone(),
                tone: seg_persona.tone.clone(),
                warmth: seg_persona.warmth,
                empathy: seg_persona.empathy,
                language_complexity: seg_persona.language_complexity.clone(),
                urgency: seg_persona.urgency.clone(),
                use_customer_name: seg_persona.use_customer_name,
                acknowledge_emotions: seg_persona.acknowledge_emotions,
                use_hinglish: seg_persona.use_hinglish,
                max_response_words: seg_persona.max_response_words,
            }
        })
    }

    // ====== P18 FIX: SegmentAdapter Config Builder ======

    /// Build SegmentAdapterConfig from domain configuration
    ///
    /// This bridges voice_agent_config to voice_agent_core::SegmentAdapterConfig,
    /// loading all domain-specific content from YAML files.
    ///
    /// ```ignore
    /// let adapter_config = view.build_segment_adapter_config();
    /// let adapter = SegmentAdapter::from_config(adapter_config);
    /// ```
    pub fn build_segment_adapter_config(&self) -> voice_agent_core::SegmentAdapterConfig {
        use voice_agent_core::{ObjectionResponseConfig, SegmentAdapterConfig};

        let mut config = SegmentAdapterConfig::default();

        // Load segment features from features.yaml
        for (segment_id, feature_ids) in &self.config.features.segment_features {
            config.segment_features.insert(segment_id.clone(), feature_ids.clone());
        }

        // Load value propositions from features.yaml
        for (segment_id, propositions) in &self.config.features.value_propositions {
            config.value_propositions.insert(segment_id.clone(), propositions.clone());
        }

        // Load objection responses from objections.yaml
        // Maps config ObjectionResponse to core ObjectionResponseConfig
        for (objection_id, definition) in &self.config.objections.objections {
            // Get English response as default
            if let Some(response) = definition.responses.get("en") {
                // Map config fields to core fields
                // Config: acknowledge, reframe, evidence, call_to_action
                // Core: acknowledgment, response, follow_up, highlight_feature
                config.objection_responses.insert(
                    objection_id.clone(),
                    ObjectionResponseConfig {
                        segment: "trust_seeker".to_string(), // Default segment
                        acknowledgment: response.acknowledge.clone(),
                        response: format!("{} {}", response.reframe, response.evidence),
                        follow_up: if response.call_to_action.is_empty() {
                            None
                        } else {
                            Some(response.call_to_action.clone())
                        },
                        highlight_feature: "security".to_string(), // Default feature
                    },
                );
            }
        }

        config
    }

    // ====== P16 FIX: Additional Agent Methods ======

    /// Get product name from brand config
    pub fn product_name(&self) -> &str {
        &self.config.brand.product_name
    }

    /// Get competitors configuration
    pub fn competitors_config(&self) -> &CompetitorsConfig {
        &self.config.competitors_config
    }

    /// Get stage fallback response with brand substitution
    pub fn stage_fallback_response(&self, stage_name: &str, language: &str) -> Option<String> {
        self.config.prompts.get_stage_fallback(stage_name, language)
            .map(|r| self.substitute_brand_placeholders(r))
    }

    /// Get greeting text for a language
    pub fn greeting(&self, language: &str) -> String {
        let template = self.config.prompts.get_greeting(language);
        self.substitute_brand_placeholders(template)
    }

    /// Get farewell text for a language
    pub fn farewell(&self, language: &str) -> String {
        let template = self.config.prompts.get_farewell(language);
        self.substitute_brand_placeholders(template)
    }

    /// Substitute brand placeholders in text
    /// P16 FIX: Supports both new ({company_name}) and legacy ({bank_name}) placeholders
    fn substitute_brand_placeholders(&self, text: &str) -> String {
        text.replace("{company_name}", &self.config.brand.company_name)
            .replace("{bank_name}", &self.config.brand.company_name) // Legacy support
            .replace("{brand.company_name}", &self.config.brand.company_name)
            .replace("{brand.bank_name}", &self.config.brand.company_name) // Legacy support
            .replace("{agent_name}", &self.config.brand.agent_name)
            .replace("{brand.agent_name}", &self.config.brand.agent_name)
            .replace("{product_name}", &self.config.brand.product_name)
            .replace("{brand.product_name}", &self.config.brand.product_name)
            .replace("{helpline}", &self.config.brand.helpline)
            .replace("{brand.helpline}", &self.config.brand.helpline)
    }

    // ====== P16 FIX: Slot Alias Resolution for Fact Storage ======

    /// Resolve a slot name to its canonical fact key using config aliases
    /// Returns Some(canonical_key) if an alias exists, None otherwise
    pub fn resolve_slot_alias(&self, slot_name: &str) -> Option<&str> {
        self.config.slots.resolve_slot_alias(slot_name)
    }

    /// Check if a slot name should trigger customer name update (not fact storage)
    pub fn is_customer_name_slot(&self, slot_name: &str) -> bool {
        self.config.slots.is_customer_name_slot(slot_name)
    }

    /// Get the canonical fact key for a slot, using config aliases
    /// Returns the alias if one exists, otherwise returns the original slot name
    pub fn canonical_fact_key<'a>(&'a self, slot_name: &'a str) -> &'a str {
        self.config.slots.canonical_fact_key(slot_name)
    }

    /// Check if slot aliases are configured
    pub fn has_slot_aliases(&self) -> bool {
        self.config.slots.has_slot_aliases()
    }

    // ====== P16 FIX: Intent to Tool Resolution ======

    /// Resolve which tool to call for an intent, given the available slots
    /// Returns Some(tool_name) if a tool should be called, None otherwise
    pub fn resolve_tool_for_intent(&self, intent: &str, available_slots: &[&str]) -> Option<&str> {
        self.config.tools.resolve_tool_for_intent(intent, available_slots)
    }

    /// Get the intent-to-tool mapping for an intent
    pub fn get_intent_mapping(&self, intent: &str) -> Option<&super::tools::IntentToolMapping> {
        self.config.tools.get_intent_mapping(intent)
    }

    /// Check if intent-to-tool mappings are configured
    pub fn has_intent_mappings(&self) -> bool {
        self.config.tools.has_intent_mappings()
    }

    /// Get default arguments for a tool
    /// Returns a HashMap of argument_name -> default_value
    pub fn get_tool_defaults(&self, tool: &str) -> Option<&std::collections::HashMap<String, serde_json::Value>> {
        self.config.tools.get_tool_defaults(tool)
    }

    /// Get argument name mapping for a tool
    /// Returns a HashMap of slot_name -> tool_argument_name
    pub fn get_argument_mapping(&self, tool: &str) -> Option<&std::collections::HashMap<String, String>> {
        self.config.tools.get_argument_mapping(tool)
    }

    /// P20 FIX: Get common argument mappings that apply to all tools
    pub fn get_common_argument_mappings(&self) -> &std::collections::HashMap<String, String> {
        self.config.tools.get_common_argument_mappings()
    }

    /// Map a slot name to the corresponding tool argument name
    pub fn map_slot_to_argument<'a>(&'a self, tool: &str, slot: &'a str) -> &'a str {
        self.config.tools.get_argument_mapping(tool)
            .and_then(|m| m.get(slot).map(|s| s.as_str()))
            .unwrap_or(slot)
    }

    // ====== Domain Context / Vocabulary ======

    /// Get vocabulary terms for text processing
    pub fn vocabulary_terms(&self) -> &[String] {
        &self.config.vocabulary.terms
    }

    /// Get phrases for text processing
    pub fn vocabulary_phrases(&self) -> &[String] {
        &self.config.vocabulary.phrases
    }

    /// Get abbreviations as (short, full) pairs
    pub fn vocabulary_abbreviations(&self) -> Vec<(String, String)> {
        self.config.vocabulary.abbreviations
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Get entity types to preserve
    pub fn vocabulary_entities(&self) -> &[String] {
        &self.config.vocabulary.preserve_entities
    }

    /// Get all competitor names for text processing
    pub fn all_competitor_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        // From extended competitors config
        for (_, entry) in &self.config.competitors_config.competitors {
            names.push(entry.display_name.clone());
            names.extend(entry.aliases.clone());
        }
        // From basic competitors in domain.yaml
        for (_, entry) in &self.config.competitors {
            names.push(entry.display_name.clone());
            names.extend(entry.aliases.clone());
        }
        names.sort();
        names.dedup();
        names
    }

    /// Create DomainContext from config for text processing
    /// Returns a DomainContext populated from vocabulary config
    pub fn create_domain_context(&self) -> voice_agent_core::DomainContext {
        voice_agent_core::DomainContext::from_config(
            &self.config.domain_id,
            self.vocabulary_terms().to_vec(),
            self.vocabulary_phrases().to_vec(),
            self.vocabulary_abbreviations(),
            self.vocabulary_entities().to_vec(),
            self.all_competitor_names(),
        )
    }

    // ====== P18 FIX: Memory Compressor Configuration (Domain-Agnostic) ======

    /// Get the memory compressor configuration
    /// Used by ExtractiveCompressor for config-driven sentence scoring
    pub fn memory_compressor_config(&self) -> &MemoryCompressorConfig {
        &self.config.memory_compressor
    }

    // ====== P18 FIX: Currency Configuration (Domain-Agnostic) ======

    /// Get the full currency configuration
    pub fn currency_config(&self) -> &CurrencyConfig {
        &self.config.currency
    }

    /// Get the savings unit amount (e.g., 100000 for "lakh")
    /// Used for calculating per-unit savings in comparisons
    pub fn savings_unit_amount(&self) -> f64 {
        self.config.currency.display_units.savings_unit.amount
    }

    /// Get the savings unit name (e.g., "lakh")
    pub fn savings_unit_name(&self) -> &str {
        &self.config.currency.display_units.savings_unit.name
    }

    /// Get the currency symbol (e.g., "₹")
    pub fn currency_symbol(&self) -> &str {
        &self.config.currency.symbol
    }

    /// Get the currency code (e.g., "INR")
    pub fn currency_code(&self) -> &str {
        &self.config.currency.code
    }

    /// P2.6 FIX: Get the currency field suffix (e.g., "inr" for "_inr" suffixes)
    pub fn currency_field_suffix(&self) -> &str {
        &self.config.currency.field_suffix
    }

    // ====== P18 FIX: Quality Tier Configuration (Domain-Agnostic) ======

    /// Parse free text to quality tier ID using config patterns
    /// Replaces hardcoded parse_purity_id() with config-driven parsing
    pub fn parse_quality_tier(&self, slot_name: &str, text: &str) -> Option<String> {
        self.config.slots.parse_quality_tier(slot_name, text)
    }

    /// Get display string for a quality tier ID
    /// Replaces hardcoded format_purity_display() with config-driven lookup
    pub fn format_quality_display(&self, slot_name: &str, tier_id: &str) -> Option<&str> {
        self.config.slots.format_quality_display(slot_name, tier_id)
    }

    /// Get all tier IDs for a slot
    pub fn quality_tier_ids(&self, slot_name: &str) -> Vec<&str> {
        self.config.slots.quality_tier_ids(slot_name)
    }

    // ====== P18 FIX: Intent Detector Configuration (Domain-Agnostic) ======

    /// Get competitor patterns for IntentDetector::add_competitor_patterns()
    ///
    /// Returns tuples of (id, display_name, regex_pattern) for each competitor.
    /// This method converts owned strings to references for the IntentDetector API.
    pub fn competitor_intent_patterns(&self) -> Vec<(&str, &str, String)> {
        self.config.competitors
            .iter()
            .map(|(id, entry)| {
                // Build pattern from ID and aliases
                let mut all_names = vec![id.as_str()];
                all_names.extend(entry.aliases.iter().map(|s| s.as_str()));

                // Create regex pattern that matches any of the names
                let alternatives = all_names
                    .iter()
                    .map(|name| regex::escape(name))
                    .collect::<Vec<_>>()
                    .join("|");

                let pattern = format!(r"(?i)\b({})\b", alternatives);

                (id.as_str(), entry.display_name.as_str(), pattern)
            })
            .collect()
    }

    /// P2.1 FIX: Get quality tier patterns for IntentDetector::add_variant_patterns()
    ///
    /// Returns tuples of (tier_value, regex_pattern) for each quality tier.
    /// This allows domain-specific quality tiers (gold purity, car condition, etc.)
    /// to be detected from user utterances.
    pub fn quality_tier_intent_patterns(&self) -> Vec<(String, String)> {
        self.config
            .extraction_patterns
            .asset_quality
            .tiers
            .iter()
            .flat_map(|tier| {
                // Collect all patterns from all languages
                tier.patterns
                    .values()
                    .flatten()
                    .map(|pattern| (tier.value.clone(), pattern.clone()))
            })
            .collect()
    }

    /// P2.1 FIX: Get location patterns for IntentDetector
    ///
    /// Returns a combined regex pattern for all configured cities.
    /// This replaces the hardcoded 9-city list with config-driven cities.
    pub fn location_intent_pattern(&self) -> String {
        let city_names: Vec<String> = self
            .config
            .extraction_patterns
            .locations
            .cities
            .iter()
            .flat_map(|city| {
                let mut names = vec![regex::escape(&city.name)];
                names.extend(city.aliases.iter().map(|a| regex::escape(a)));
                // Also include the pattern_en if it's a different simple name
                if !city.pattern_en.is_empty() && !city.pattern_en.contains('|') {
                    names.push(regex::escape(&city.pattern_en));
                }
                names
            })
            .collect();

        if city_names.is_empty() {
            // Fallback to a minimal pattern if no cities configured
            return r"(?i)\b(mumbai|delhi|bangalore)\b".to_string();
        }

        format!(r"(?i)\b({})\b", city_names.join("|"))
    }

    // ====== P18 FIX: RAG Configuration (Domain-Agnostic) ======

    /// Get the RAG collection name for this domain.
    /// Returns configured name or derives from domain_id (e.g., "my_domain" -> "my_domain_knowledge")
    pub fn rag_collection_name(&self) -> String {
        self.config
            .rag_collection_name
            .clone()
            .unwrap_or_else(|| format!("{}_knowledge", self.config.domain_id))
    }

    // ====== P20 FIX: Config-Driven Trait Providers ======
    //
    // These methods provide access to config-driven trait implementations
    // that replace hardcoded enums and patterns throughout the codebase.
    // Each provider is created from YAML config through DomainBridge.

    /// Get a feature provider for config-driven feature management
    ///
    /// Replaces hardcoded Feature enum with config-driven features.
    /// Use for personalization, segment-specific feature highlighting, etc.
    ///
    /// ```ignore
    /// let provider = view.feature_provider();
    /// let features = provider.features_for_segment("high_value");
    /// ```
    pub fn feature_provider(
        &self,
    ) -> Arc<dyn voice_agent_core::traits::FeatureProvider> {
        let bridge = super::DomainBridge::new(self.config.clone());
        bridge.feature_provider()
    }

    /// Get an objection provider for config-driven objection handling
    ///
    /// Replaces hardcoded Objection enum with config-driven objections.
    /// Use for objection detection, ACRE responses, etc.
    ///
    /// ```ignore
    /// let provider = view.objection_provider();
    /// if let Some((id, confidence)) = provider.detect_objection(text, "en", &[]) {
    ///     let response = provider.get_acre_response(&id, "en", &vars);
    /// }
    /// ```
    pub fn objection_provider(
        &self,
    ) -> Arc<dyn voice_agent_core::traits::ObjectionProvider> {
        let bridge = super::DomainBridge::new(self.config.clone());
        bridge.objection_provider()
    }

    /// Get a tool argument provider for config-driven tool defaults
    ///
    /// Replaces hardcoded tool defaults and intent-to-tool mappings.
    /// Use for tool resolution and argument preparation.
    ///
    /// ```ignore
    /// let provider = view.tool_argument_provider();
    /// if let Some(tool) = provider.resolve_tool_for_intent("check_eligibility", &["weight"]) {
    ///     let defaults = provider.get_tool_defaults(&tool);
    /// }
    /// ```
    pub fn tool_argument_provider(
        &self,
    ) -> Arc<dyn voice_agent_core::traits::ToolArgumentProvider> {
        let bridge = super::DomainBridge::new(self.config.clone());
        bridge.tool_argument_provider()
    }

    /// Get a lead classifier for config-driven MQL/SQL classification
    ///
    /// Replaces hardcoded lead classification rules with config-driven rules.
    /// Use for lead scoring and qualification.
    ///
    /// ```ignore
    /// let classifier = view.lead_classifier();
    /// let classification = classifier.classify(&signals);
    /// let qualification = classifier.qualification_level(score);
    /// ```
    pub fn lead_classifier(&self) -> Arc<dyn voice_agent_core::traits::LeadClassifier> {
        let bridge = super::DomainBridge::new(self.config.clone());
        bridge.lead_classifier()
    }

    /// Get the underlying config for advanced use cases
    ///
    /// Prefer using specific methods over direct config access.
    pub fn config(&self) -> &MasterDomainConfig {
        &self.config
    }

    /// Get the domain ID
    pub fn domain_id(&self) -> &str {
        &self.config.domain_id
    }

    // ====== P16 FIX: Compliance Configuration ======

    /// Get AI disclosure message for a language (RBI compliance)
    ///
    /// Returns the localized AI disclosure message that must be played
    /// at the start of conversations per regulatory requirements.
    /// Falls back to English if requested language is not available.
    ///
    /// ```ignore
    /// let disclosure = view.ai_disclosure("hi");
    /// // Returns Hindi disclosure message
    /// ```
    pub fn ai_disclosure(&self, language: &str) -> &str {
        self.config.compliance.get_ai_disclosure(language)
    }

    /// Check if a phrase is forbidden by compliance rules
    pub fn is_forbidden_phrase(&self, text: &str) -> bool {
        self.config.compliance.is_forbidden(text)
    }

    /// Check if an interest rate is within allowed bounds
    pub fn is_rate_compliant(&self, rate: f64) -> bool {
        self.config.compliance.is_rate_valid(rate)
    }

    // ====== P22 FIX: Intent Configuration ======

    /// Get the full intents configuration
    pub fn intents_config(&self) -> &super::IntentsConfig {
        &self.config.intents
    }

    /// Get an intent definition by name
    pub fn get_intent(&self, name: &str) -> Option<&super::IntentDefinition> {
        self.config.intents.get_intent(name)
    }

    /// Get all intent names
    pub fn intent_names(&self) -> Vec<&str> {
        self.config.intents.intent_names()
    }

    /// Get required slots for an intent
    pub fn required_slots_for_intent(&self, intent: &str) -> Vec<&str> {
        self.config
            .intents
            .get_intent(intent)
            .map(|i| i.required_slots.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get the default intent when none matches
    pub fn default_intent(&self) -> &str {
        &self.config.intents.default_intent
    }

    /// Get minimum confidence threshold for intent detection
    pub fn intent_min_confidence(&self) -> f32 {
        self.config.intents.min_confidence
    }

    // ====== P22 FIX: Full Vocabulary Configuration ======

    /// Get the full vocabulary configuration (ASR boost, phonetic corrections)
    pub fn vocabulary_full(&self) -> &super::FullVocabularyConfig {
        &self.config.vocabulary_full
    }

    /// Get phonetic correction for a word
    pub fn phonetic_correction(&self, word: &str) -> Option<&str> {
        self.config.vocabulary_full.phonetic_correction(word)
    }

    /// Get term boost for ASR
    pub fn term_boost(&self, term: &str) -> f64 {
        self.config.vocabulary_full.term_boost(term)
    }

    /// Convert Hindi number word to numeric value
    pub fn hindi_to_number(&self, word: &str) -> Option<i64> {
        self.config.vocabulary_full.hindi_to_number(word)
    }

    /// Expand abbreviation to full form
    pub fn expand_abbreviation(&self, abbrev: &str) -> Option<&str> {
        self.config.vocabulary_full.expand_abbreviation(abbrev)
    }

    // ====== P22 FIX: Entity Configuration ======

    /// Get the full entities configuration
    pub fn entities_config(&self) -> &super::EntitiesConfig {
        &self.config.entities
    }

    /// Get display name for an entity type
    pub fn entity_display_name(&self, entity_type: &str, language: &str) -> Option<&str> {
        self.config.entities.display_name(entity_type, language)
    }

    /// Get unit for an entity type
    pub fn entity_unit(&self, entity_type: &str, language: &str) -> Option<&str> {
        self.config.entities.unit(entity_type, language)
    }

    /// Get extraction priority order for entity type
    pub fn entity_extraction_order(&self, entity_type: &str) -> usize {
        self.config.entities.extraction_order(entity_type)
    }

    /// Resolve entity alias to canonical name
    pub fn resolve_entity_alias<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        self.config.entities.resolve_alias(name)
    }

    /// Format entity value for display
    pub fn format_entity_value(&self, entity_type: &str, value: &str, language: &str) -> String {
        self.config.entities.format_value(entity_type, value, language)
    }
}

/// View for the llm crate
/// Provides access to prompts, tool schemas, brand info
pub struct LlmDomainView {
    config: Arc<MasterDomainConfig>,
}

impl LlmDomainView {
    pub fn new(config: Arc<MasterDomainConfig>) -> Self {
        Self { config }
    }

    /// Get domain display name for prompts
    pub fn domain_name(&self) -> &str {
        &self.config.display_name
    }

    /// Get brand information for prompts
    pub fn company_name(&self) -> &str {
        &self.config.brand.company_name
    }

    pub fn agent_name(&self) -> &str {
        &self.config.brand.agent_name
    }

    pub fn helpline(&self) -> &str {
        &self.config.brand.helpline
    }

    /// Get key facts for system prompt
    pub fn key_facts(&self) -> Vec<String> {
        let mut facts = Vec::new();

        // Best interest rate
        if let Some(best_tier) = self.config.constants.interest_rates.tiers.last() {
            facts.push(format!("Interest rates: Starting from {}% p.a.", best_tier.rate));
        }

        // LTV
        facts.push(format!("LTV: Up to {}% of gold value", self.config.constants.ltv_percent));

        // Loan range
        let min = self.config.constants.loan_limits.min;
        let max = self.config.constants.loan_limits.max;
        facts.push(format!("Loan range: ₹{} to ₹{}", format_amount(min), format_amount(max)));

        facts
    }

    /// Get product variants for tool responses
    pub fn product_names(&self) -> Vec<&str> {
        self.config.products.values().map(|p| p.name.as_str()).collect()
    }

    // ====== Tool Schema Configuration ======

    /// Get the full tools configuration
    pub fn tools_config(&self) -> &ToolsConfig {
        &self.config.tools
    }

    /// Get a tool schema by name
    pub fn get_tool(&self, name: &str) -> Option<&ToolSchema> {
        self.config.tools.get_tool(name)
    }

    /// Get all tool names
    pub fn tool_names(&self) -> Vec<&str> {
        self.config.tools.tool_names()
    }

    /// Get enabled tool names
    pub fn enabled_tool_names(&self) -> Vec<&str> {
        self.config.tools.enabled_tool_names()
    }

    /// Get all tool schemas as JSON for LLM consumption
    pub fn tool_schemas_json(&self) -> Vec<serde_json::Value> {
        self.config.tools.to_json_schemas()
    }

    /// Get tool usage guideline
    pub fn tool_guideline(&self, key: &str) -> Option<&str> {
        self.config.tools.get_guideline(key)
    }

    /// Get general tool usage guideline
    pub fn general_tool_guideline(&self) -> Option<&str> {
        self.config.tools.get_guideline("general")
    }

    // ====== Prompts Configuration ======

    /// Get the full prompts configuration
    pub fn prompts_config(&self) -> &PromptsConfig {
        &self.config.prompts
    }

    /// Get system prompt template
    pub fn system_prompt_template(&self) -> &str {
        &self.config.prompts.system_prompt
    }

    /// Get language style description
    pub fn language_style(&self, language: &str) -> &str {
        self.config.prompts.language_style(language)
    }

    /// Build persona traits string
    pub fn build_persona_traits(&self, warmth: f32, empathy: f32, formality: f32, urgency: f32) -> String {
        self.config.prompts.build_persona_traits(warmth, empathy, formality, urgency)
    }

    /// Build system prompt with brand and persona
    pub fn build_system_prompt(
        &self,
        persona_traits: &str,
        language: &str,
        key_facts: &str,
    ) -> String {
        self.config.prompts.build_system_prompt(
            &self.config.brand.agent_name,
            &self.config.brand.company_name,
            persona_traits,
            language,
            key_facts,
            &self.config.brand.helpline,
        )
    }

    /// Build RAG context message
    pub fn build_rag_context(&self, context: &str) -> String {
        self.config.prompts.build_rag_context(context)
    }

    /// Build stage guidance message
    pub fn build_stage_guidance(&self, guidance: &str) -> String {
        self.config.prompts.build_stage_guidance(guidance)
    }

    /// Get response template for scenario and language
    pub fn response_template(&self, scenario: &str, language: &str) -> Option<&str> {
        self.config.prompts.response_template(scenario, language)
    }

    /// Get error template for scenario and language
    pub fn error_template(&self, scenario: &str, language: &str) -> Option<&str> {
        self.config.prompts.error_template(scenario, language)
    }

    // ====== P7 FIX: Methods migrated from DomainConfigManager ======

    /// Get greeting for language (from response templates)
    /// Falls back to English if language not found
    pub fn get_greeting(&self, language: &str) -> String {
        self.config.prompts.response_template("greeting", language)
            .or_else(|| self.config.prompts.response_template("greeting", "en"))
            .map(|template| {
                template
                    .replace("{agent_name}", &self.config.brand.agent_name)
                    .replace("{company_name}", &self.config.brand.company_name)
            })
            .unwrap_or_else(|| {
                format!(
                    "Hello! I'm {} from {}. How can I help you today?",
                    self.config.brand.agent_name,
                    self.config.brand.company_name
                )
            })
    }

    /// Get greeting with time-based prefix (morning/afternoon/evening)
    pub fn get_greeting_with_time(&self, language: &str, hour: u32) -> String {
        let time_greeting = match hour {
            5..=11 => "Good morning",
            12..=16 => "Good afternoon",
            17..=20 => "Good evening",
            _ => "Hello",
        };

        let base_greeting = self.get_greeting(language);
        format!("{}! {}", time_greeting, base_greeting.trim_start_matches("Hello! "))
    }

    /// Get farewell message for language
    pub fn get_farewell(&self, language: &str) -> String {
        self.config.prompts.response_template("farewell", language)
            .or_else(|| self.config.prompts.response_template("farewell", "en"))
            .map(|template| {
                template
                    .replace("{helpline}", &self.config.brand.helpline)
            })
            .unwrap_or_else(|| {
                format!(
                    "Thank you for speaking with me! Call our helpline at {} if you have any questions.",
                    self.config.brand.helpline
                )
            })
    }
}

/// View for the tools crate
/// Provides access to tool configs, branch data, SMS templates, constants
pub struct ToolsDomainView {
    config: Arc<MasterDomainConfig>,
}

impl ToolsDomainView {
    pub fn new(config: Arc<MasterDomainConfig>) -> Self {
        Self { config }
    }

    /// Get interest rate for eligibility calculations
    pub fn get_rate_for_amount(&self, amount: f64) -> f64 {
        self.config.get_rate_for_amount(amount)
    }

    /// Get LTV percentage
    pub fn ltv_percent(&self) -> f64 {
        self.config.constants.ltv_percent
    }

    /// Get variant/purity factor (e.g., K24=1.0, K22=0.916 for gold)
    pub fn purity_factor(&self, variant: &str) -> f64 {
        self.config.constants.variant_factors
            .get(variant)
            .copied()
            .unwrap_or(1.0)
    }

    /// Get asset price per unit (e.g., gold price per gram)
    pub fn asset_price_per_unit(&self) -> f64 {
        self.config.constants.asset_price_per_unit
    }

    /// Legacy alias for gold_price_per_gram
    pub fn gold_price_per_gram(&self) -> f64 {
        self.asset_price_per_unit()
    }

    /// P20 FIX: Get asset unit from config (e.g., "gram" for gold, "unit" for others)
    pub fn asset_unit(&self) -> &str {
        self.config.slots.asset_unit()
    }

    /// Get loan limits
    pub fn min_loan_amount(&self) -> f64 {
        self.config.constants.loan_limits.min
    }

    pub fn max_loan_amount(&self) -> f64 {
        self.config.constants.loan_limits.max
    }

    /// Get processing fee percentage
    pub fn processing_fee_percent(&self) -> f64 {
        self.config.constants.processing_fee_percent
    }

    /// Get competitor info for savings calculations
    pub fn get_competitor(&self, name: &str) -> Option<CompetitorInfo> {
        self.config.get_competitor(name).map(|c| CompetitorInfo {
            name: c.display_name.clone(),
            rate: c.typical_rate,
            ltv: c.ltv_percent,
        })
    }

    /// Get brand info for SMS/responses
    pub fn company_name(&self) -> &str {
        &self.config.brand.company_name
    }

    pub fn helpline(&self) -> &str {
        &self.config.brand.helpline
    }

    // ====== Branch Configuration ======

    /// Get the full branches configuration
    pub fn branches_config(&self) -> &BranchesConfig {
        &self.config.branches
    }

    /// Get all branches
    pub fn all_branches(&self) -> &[BranchEntry] {
        &self.config.branches.branches
    }

    /// Find branches by city
    pub fn find_branches_by_city(&self, city: &str) -> Vec<&BranchEntry> {
        self.config.branches.find_by_city(city)
    }

    /// Find branches by pincode
    pub fn find_branches_by_pincode(&self, pincode: &str) -> Vec<&BranchEntry> {
        self.config.branches.find_by_pincode(pincode)
    }

    /// Get branch by ID
    pub fn get_branch(&self, branch_id: &str) -> Option<&BranchEntry> {
        self.config.branches.get_branch(branch_id)
    }

    /// Get branches with the primary service (domain-agnostic)
    pub fn service_branches(&self) -> Vec<&BranchEntry> {
        self.config.branches.service_locations()
    }

    /// Legacy alias for backward compatibility
    /// P21 FIX: Deprecated - use the domain-agnostic service_branches() method
    #[deprecated(since = "0.20.0", note = "Use service_branches() for domain-agnostic access")]
    pub fn legacy_service_branches(&self) -> Vec<&BranchEntry> {
        self.service_branches()
    }

    /// Get default max results for branch search
    pub fn branch_search_max_results(&self) -> usize {
        self.config.branches.defaults.max_results
    }

    // ====== SMS Templates Configuration ======

    /// Get the full SMS templates configuration
    pub fn sms_templates_config(&self) -> &SmsTemplatesConfig {
        &self.config.sms_templates
    }

    /// Get SMS template by type and language
    pub fn sms_template(&self, template_type: &str, language: &str) -> Option<&str> {
        self.config.sms_templates.get_template(template_type, language)
    }

    /// Build SMS message from template with placeholders
    pub fn build_sms_message(
        &self,
        template_type: &str,
        language: &str,
        placeholders: &HashMap<String, String>,
    ) -> Option<String> {
        self.config.sms_templates.build_message(template_type, language, placeholders)
    }

    /// Get all SMS template types
    pub fn sms_template_types(&self) -> Vec<&str> {
        self.config.sms_templates.template_types()
    }

    /// Check if SMS type is transactional
    pub fn is_transactional_sms(&self, template_type: &str) -> bool {
        self.config.sms_templates.is_transactional(template_type)
    }

    // ====== Extended Competitors Configuration ======

    /// Get the full competitors configuration
    pub fn competitors_config(&self) -> &CompetitorsConfig {
        &self.config.competitors_config
    }

    /// Get extended competitor info by ID
    pub fn get_competitor_extended(&self, id: &str) -> Option<&ExtCompetitorEntry> {
        self.config.competitors_config.get_competitor(id)
    }

    /// Find competitor by name or alias
    pub fn find_competitor_by_name(&self, name: &str) -> Option<(&str, &ExtCompetitorEntry)> {
        self.config.competitors_config.find_by_name(name)
    }

    /// Get all NBFCs
    pub fn nbfc_competitors(&self) -> Vec<(&str, &ExtCompetitorEntry)> {
        self.config.competitors_config.nbfcs()
    }

    /// Get all bank competitors
    pub fn bank_competitors(&self) -> Vec<(&str, &ExtCompetitorEntry)> {
        self.config.competitors_config.banks()
    }

    /// Get default rate for competitor type
    pub fn default_competitor_rate(&self, competitor_type: &str) -> f64 {
        self.config.competitors_config.default_rate_for_type(competitor_type)
    }

    /// Get highlighted comparison points
    pub fn highlighted_comparison_points(&self) -> Vec<(&str, &str)> {
        self.config.competitors_config.highlighted_points()
            .into_iter()
            .map(|p| (p.category.as_str(), p.our_advantage.as_str()))
            .collect()
    }

    /// P14 FIX: Get our features for comparison
    pub fn our_features(&self) -> &[String] {
        self.config.competitors_config.our_features()
    }

    /// P14 FIX: Get all competitor IDs
    pub fn all_competitor_ids(&self) -> Vec<&str> {
        self.config.competitors_config.competitor_ids()
    }

    /// P14 FIX: Get competitor data tuple (id, display_name, rate, ltv, strengths)
    /// Returns all competitors as tuples for comparison tool
    pub fn all_competitors_data(&self) -> Vec<(&str, &str, f64, f64, Vec<&str>)> {
        self.config.competitors_config.competitors.iter()
            .map(|(id, entry)| {
                (
                    id.as_str(),
                    entry.display_name.as_str(),
                    entry.typical_rate,
                    entry.ltv_percent,
                    entry.strengths.iter().map(|s| s.as_str()).collect(),
                )
            })
            .collect()
    }

    // ====== P7 FIX: Methods migrated from DomainConfigManager ======

    /// Check if doorstep service is available in a city
    pub fn doorstep_available(&self, city: &str) -> bool {
        self.config.branches.doorstep_available(city)
    }

    /// Calculate monthly savings vs competitor
    /// Returns savings info, or None if competitor not found
    pub fn calculate_competitor_savings(
        &self,
        competitor: &str,
        loan_amount: f64,
    ) -> Option<MonthlySavings> {
        // Try to get competitor rate from extended config first, then basic config
        let their_rate = self.config.competitors_config.find_by_name(competitor)
            .map(|(_, entry)| entry.typical_rate)
            .or_else(|| {
                // Fallback to basic competitor list in domain.yaml
                self.config.get_competitor(competitor).map(|c| c.typical_rate)
            })?;

        let our_rate = self.config.get_rate_for_amount(loan_amount);

        if their_rate <= our_rate {
            // No savings if competitor is cheaper
            return Some(MonthlySavings {
                monthly: 0.0,
                annual: 0.0,
                our_rate,
                their_rate,
            });
        }

        // Monthly savings = loan_amount * (rate_diff / 12 / 100)
        let rate_diff = their_rate - our_rate;
        let monthly_savings = loan_amount * rate_diff / 12.0 / 100.0;
        let annual_savings = monthly_savings * 12.0;

        Some(MonthlySavings {
            monthly: monthly_savings,
            annual: annual_savings,
            our_rate,
            their_rate,
        })
    }

    // ====== P12: Methods replacing GoldLoanConfig ======

    /// Calculate asset value given weight/quantity and variant/purity
    pub fn calculate_asset_value(&self, weight: f64, variant: &str) -> f64 {
        let variant_factor = self.purity_factor(variant);
        weight * self.asset_price_per_unit() * variant_factor
    }

    /// Legacy alias for backward compatibility
    #[deprecated(since = "0.20.0", note = "Use calculate_asset_value() instead")]
    pub fn calculate_gold_value(&self, weight_grams: f64, purity: &str) -> f64 {
        self.calculate_asset_value(weight_grams, purity)
    }

    /// Calculate maximum loan amount based on asset value
    pub fn calculate_max_loan(&self, asset_value: f64) -> f64 {
        let max_from_ltv = asset_value * (self.ltv_percent() / 100.0);
        max_from_ltv.min(self.max_loan_amount())
    }

    /// Get competitor rate by name (convenience method)
    /// Falls back to default NBFC rate if competitor not found
    pub fn get_competitor_rate(&self, lender: &str) -> f64 {
        self.config.competitors_config.find_by_name(lender)
            .map(|(_, entry)| entry.typical_rate)
            .or_else(|| {
                self.config.get_competitor(lender).map(|c| c.typical_rate)
            })
            .unwrap_or_else(|| self.default_competitor_rate("nbfc"))
    }

    /// Calculate monthly savings when switching from competitor
    /// Uses the competitor's rate and our tiered rate
    pub fn calculate_monthly_savings(&self, loan_amount: f64, current_rate: f64) -> f64 {
        let our_rate = self.get_rate_for_amount(loan_amount);
        let current_monthly = loan_amount * (current_rate / 100.0 / 12.0);
        let our_monthly = loan_amount * (our_rate / 100.0 / 12.0);
        current_monthly - our_monthly
    }

    /// Get the base/headline interest rate (for marketing purposes)
    pub fn base_interest_rate(&self) -> f64 {
        self.config.constants.interest_rates.base_rate
    }

    /// P15 FIX: Get rate tier name for an amount
    /// Returns the tier name (e.g., "Standard", "Premium", "Elite")
    /// If tier has no name, derives from tier index
    pub fn get_rate_tier_name(&self, amount: f64) -> &str {
        for (idx, tier) in self.config.constants.interest_rates.tiers.iter().enumerate() {
            let threshold = tier.max_amount.unwrap_or(f64::MAX);
            if amount <= threshold {
                // Return tier name if set, otherwise derive from index
                if !tier.name.is_empty() {
                    return &tier.name;
                }
                // Derive tier name from index
                return match idx {
                    0 => "Standard",
                    1 => "Premium",
                    2 => "Elite",
                    _ => "Special",
                };
            }
        }
        // Return the last tier name for amounts above all thresholds
        self.config.constants.interest_rates.tiers
            .last()
            .and_then(|t| {
                if !t.name.is_empty() {
                    Some(t.name.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("Elite")
    }

    /// P15 FIX: Get competitor IDs for building dynamic schema enums
    /// Returns owned strings for use in tool schemas
    pub fn competitor_ids(&self) -> Vec<String> {
        self.config.competitors_config.competitor_ids()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    // ====== P15 FIX: Domain Context / Vocabulary ======

    /// Get vocabulary terms for text processing
    pub fn vocabulary_terms(&self) -> &[String] {
        &self.config.vocabulary.terms
    }

    /// Get phrases for text processing
    pub fn vocabulary_phrases(&self) -> &[String] {
        &self.config.vocabulary.phrases
    }

    /// Get abbreviations as (short, full) pairs
    pub fn vocabulary_abbreviations(&self) -> Vec<(String, String)> {
        self.config.vocabulary.abbreviations
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Get entity types to preserve
    pub fn vocabulary_entities(&self) -> &[String] {
        &self.config.vocabulary.preserve_entities
    }

    /// Get all competitor names for text processing
    pub fn all_competitor_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        // From extended competitors config
        for (_, entry) in &self.config.competitors_config.competitors {
            names.push(entry.display_name.clone());
            names.extend(entry.aliases.clone());
        }
        // From basic competitors in domain.yaml
        for (_, entry) in &self.config.competitors {
            names.push(entry.display_name.clone());
            names.extend(entry.aliases.clone());
        }
        names.sort();
        names.dedup();
        names
    }

    /// P15 FIX: Create DomainContext from config
    /// Returns a DomainContext populated from vocabulary config
    pub fn create_domain_context(&self) -> voice_agent_core::DomainContext {
        voice_agent_core::DomainContext::from_config(
            &self.config.domain_id,
            self.vocabulary_terms().to_vec(),
            self.vocabulary_phrases().to_vec(),
            self.vocabulary_abbreviations(),
            self.vocabulary_entities().to_vec(),
            self.all_competitor_names(),
        )
    }

    // ====== P16 FIX: Tool Schema Configuration ======

    /// Get the full tools configuration
    /// Tools should use this to read their schemas from config
    pub fn tools_config(&self) -> &ToolsConfig {
        &self.config.tools
    }

    /// Get a tool schema by name
    pub fn get_tool(&self, name: &str) -> Option<&ToolSchema> {
        self.config.tools.get_tool(name)
    }

    /// Get core schema for a tool by name
    /// Returns the schema in the format expected by the Tool trait
    pub fn get_tool_core_schema(
        &self,
        name: &str,
    ) -> Option<voice_agent_core::traits::ToolSchema> {
        self.config.tools.get_core_schema(name)
    }

    // ====== P16 FIX: Document Requirements Configuration ======

    /// Get the full documents configuration
    pub fn documents_config(&self) -> &super::documents::DocumentsConfig {
        &self.config.documents
    }

    /// Get mandatory documents for all applications
    pub fn mandatory_documents(&self) -> &[super::documents::DocumentEntry] {
        &self.config.documents.mandatory_documents
    }

    /// Get domain-specific documents (e.g., items to bring for valuation)
    pub fn domain_specific_documents(&self) -> &[super::documents::DocumentEntry] {
        &self.config.documents.domain_specific_documents
    }

    /// Get additional documents for a service type
    pub fn documents_for_service_type(&self, service_type: &str) -> &[super::documents::DocumentEntry] {
        self.config.documents.documents_for_service_type(service_type)
    }

    /// Get additional documents for a customer type
    pub fn documents_for_customer_type(&self, customer_type: &str) -> &[super::documents::DocumentEntry] {
        self.config.documents.documents_for_customer_type(customer_type)
    }

    /// Get service type IDs for tool schema enum
    pub fn document_service_types(&self) -> Vec<&str> {
        self.config.documents.service_type_ids()
    }

    /// Get customer type IDs for tool schema enum
    pub fn document_customer_types(&self) -> Vec<&str> {
        self.config.documents.customer_type_ids()
    }

    /// Get existing customer note
    pub fn existing_customer_note(&self) -> &str {
        self.config.documents.existing_customer_note()
    }

    /// Get new customer note
    pub fn new_customer_note(&self) -> &str {
        self.config.documents.new_customer_note()
    }

    /// Get general notes
    pub fn document_general_notes(&self) -> &[String] {
        self.config.documents.general_notes()
    }

    /// Get document tool description from config
    pub fn document_tool_description(&self) -> &str {
        self.config.documents.tool_description()
    }

    /// Check if document config has any document definitions
    pub fn has_document_config(&self) -> bool {
        self.config.documents.has_documents()
    }

    // ====== Domain ID ======

    /// Get the domain ID
    pub fn domain_id(&self) -> &str {
        &self.config.domain_id
    }

    /// Get the domain display name
    pub fn domain_display_name(&self) -> &str {
        &self.config.display_name
    }

    /// Get product name from brand config
    pub fn product_name(&self) -> &str {
        &self.config.brand.product_name
    }

    // ====== P23 FIX: Currency Configuration ======

    /// Get the currency symbol (e.g., "₹")
    pub fn currency_symbol(&self) -> &str {
        &self.config.currency.symbol
    }

    /// Get the currency code (e.g., "INR")
    pub fn currency_code(&self) -> &str {
        &self.config.currency.code
    }

    /// P2.6 FIX: Get the currency field suffix (e.g., "inr" for "_inr" suffixes)
    pub fn currency_field_suffix(&self) -> &str {
        &self.config.currency.field_suffix
    }

    // ====== P16 FIX: Tool Response Templates ======

    /// Get a response template for a tool and scenario
    pub fn get_response_template(&self, tool: &str, scenario: &str, language: &str) -> Option<&str> {
        self.config.tool_responses.get_template(tool, scenario, language)
    }

    /// Render a response template with variable substitution
    pub fn render_response(
        &self,
        tool: &str,
        scenario: &str,
        language: &str,
        vars: &HashMap<String, String>,
    ) -> Option<String> {
        self.config.tool_responses.render_template(tool, scenario, language, vars)
    }

    /// Check if response templates are configured for a tool
    pub fn has_response_templates(&self, tool: &str) -> bool {
        self.config.tool_responses.has_tool(tool)
    }

    /// Get rate description for a tier (e.g., "premium", "competitive")
    pub fn get_rate_description(&self, tier: &str) -> &str {
        self.config.tool_responses.get_rate_description(tier)
    }

    /// Build default template variables from brand config
    pub fn default_template_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("company_name".to_string(), self.config.brand.company_name.clone());
        vars.insert("product_name".to_string(), self.config.brand.product_name.clone());
        vars.insert("helpline".to_string(), self.config.brand.helpline.clone());
        vars.insert("agent_name".to_string(), self.config.brand.agent_name.clone());
        // P23 FIX: Use config-driven currency symbol instead of hardcoded "₹"
        vars.insert("currency".to_string(), self.config.currency.symbol.clone());
        vars
    }

    // ====== P16 FIX: Intent to Tool Resolution ======

    /// Resolve which tool to call for an intent, given the available slots
    /// Returns Some(tool_name) if a tool should be called, None otherwise
    pub fn resolve_tool_for_intent(&self, intent: &str, available_slots: &[&str]) -> Option<&str> {
        self.config.tools.resolve_tool_for_intent(intent, available_slots)
    }

    /// Get the intent-to-tool mapping for an intent
    pub fn get_intent_mapping(&self, intent: &str) -> Option<&super::tools::IntentToolMapping> {
        self.config.tools.get_intent_mapping(intent)
    }

    /// Check if intent-to-tool mappings are configured
    pub fn has_intent_mappings(&self) -> bool {
        self.config.tools.has_intent_mappings()
    }

    /// Get all configured intent names
    pub fn mapped_intents(&self) -> Vec<&str> {
        self.config.tools.mapped_intents()
    }

    // ====== P20 FIX: Quality Tier Configuration ======

    /// Get the default quality tier display value
    ///
    /// Reads from slots.yaml asset_quality_tier.default or parsing.default_id.
    /// Returns the display value, not the tier ID.
    ///
    /// Falls back to generic "tier_2" if not configured.
    pub fn default_quality_tier_display(&self) -> String {
        let slot = self.config.slots.get_slot("asset_quality_tier");
        if let Some(slot_def) = slot {
            // Get the default tier ID
            let default_id = slot_def
                .default
                .as_deref()
                .or_else(|| slot_def.parsing.as_ref().map(|p| p.default_id.as_str()))
                .unwrap_or("tier_2");

            // Look up the display value for this tier ID
            if let Some(values) = &slot_def.values {
                for value in values {
                    if value.id == default_id {
                        return value.display.clone();
                    }
                }
            }

            // Fallback to ID if display not found
            return default_id.to_string();
        }

        // P23 FIX: Use generic fallback - config should define domain-specific tiers
        tracing::warn!("asset_quality_tier slot not found in config - using generic fallback");
        "tier_2".to_string()
    }

    /// Get all quality tier display values
    ///
    /// Reads from slots.yaml asset_quality_tier.values[].display
    pub fn quality_tier_displays(&self) -> Vec<String> {
        let slot = self.config.slots.get_slot("asset_quality_tier");
        if let Some(slot_def) = slot {
            if let Some(values) = &slot_def.values {
                return values.iter().map(|v| v.display.clone()).collect();
            }
        }

        // P23 FIX: Use generic fallback - config should define domain-specific tiers
        tracing::warn!("asset_quality_tier values not found in config - using generic fallback");
        vec!["tier_1".to_string(), "tier_2".to_string(), "tier_3".to_string()]
    }

    /// P20 FIX: Get all quality tiers with full information
    ///
    /// Returns tuples of (short_code, factor, description) for all tiers.
    /// Used by price tool for dynamic tier display.
    pub fn quality_tiers_full(&self) -> Vec<(String, f64, String)> {
        let slot = self.config.slots.get_slot("asset_quality_tier");
        if let Some(slot_def) = slot {
            if let Some(values) = &slot_def.values {
                return values
                    .iter()
                    .map(|v| {
                        (
                            v.short_code().to_string(),
                            v.quality_factor.unwrap_or(1.0),
                            v.description().to_string(),
                        )
                    })
                    .collect();
            }
        }

        // P23 FIX: Use generic fallback - config should define domain-specific tiers
        tracing::warn!("asset_quality_tier values not found in config - using generic fallback");
        vec![
            ("tier_1".to_string(), 1.0, "Highest quality".to_string()),
            ("tier_2".to_string(), 0.916, "Standard quality".to_string()),
            ("tier_3".to_string(), 0.75, "Economy quality".to_string()),
        ]
    }

    /// P20 FIX: Get tier short codes only
    pub fn quality_tier_short_codes(&self) -> Vec<String> {
        let slot = self.config.slots.get_slot("asset_quality_tier");
        if let Some(slot_def) = slot {
            if let Some(values) = &slot_def.values {
                return values.iter().map(|v| v.short_code().to_string()).collect();
            }
        }

        // P23 FIX: Use generic fallback - config should define domain-specific tiers
        tracing::warn!("asset_quality_tier values not found in config - using generic fallback");
        vec!["tier_1".to_string(), "tier_2".to_string(), "tier_3".to_string()]
    }

    /// P20 FIX: Get tier description by short code
    pub fn tier_description(&self, short_code: &str) -> String {
        let slot = self.config.slots.get_slot("asset_quality_tier");
        if let Some(slot_def) = slot {
            if let Some(values) = &slot_def.values {
                for v in values {
                    if v.short_code().eq_ignore_ascii_case(short_code) {
                        return v.description().to_string();
                    }
                }
            }
        }

        format!("{} quality tier", short_code)
    }

    /// Get quality factor for a display value (e.g., "22K" -> 0.916)
    ///
    /// Supports both tier IDs (tier_1, tier_2) and display values (24K, 22K)
    pub fn quality_factor_by_display(&self, display_or_id: &str) -> f64 {
        // First try the variant_factors from constants (for backward compat)
        if let Some(factor) = self.config.constants.variant_factors.get(display_or_id) {
            return *factor;
        }

        // Then try slot-based lookup
        let slot = self.config.slots.get_slot("asset_quality_tier");
        if let Some(slot_def) = slot {
            if let Some(values) = &slot_def.values {
                for value in values {
                    // Match by display or ID
                    if value.display.eq_ignore_ascii_case(display_or_id)
                        || value.id.eq_ignore_ascii_case(display_or_id)
                    {
                        // quality_factor is optional in the struct
                        return value.quality_factor.unwrap_or(1.0);
                    }
                }
            }
        }

        // Default to 1.0 if not found
        1.0
    }
}

/// Simplified competitor info for tools
#[derive(Debug, Clone)]
pub struct CompetitorInfo {
    pub name: String,
    pub rate: f64,
    pub ltv: f64,
}

/// Monthly savings calculation result
#[derive(Debug, Clone)]
pub struct MonthlySavings {
    pub monthly: f64,
    pub annual: f64,
    pub our_rate: f64,
    pub their_rate: f64,
}

/// Format amount in Indian style (lakhs/crores)
fn format_amount(amount: f64) -> String {
    if amount >= 10_000_000.0 {
        format!("{:.1} Cr", amount / 10_000_000.0)
    } else if amount >= 100_000.0 {
        format!("{:.1} L", amount / 100_000.0)
    } else {
        format!("{:.0}", amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_amount() {
        assert_eq!(format_amount(10000.0), "10000");
        assert_eq!(format_amount(100000.0), "1.0 L");
        assert_eq!(format_amount(2500000.0), "25.0 L");
        assert_eq!(format_amount(10000000.0), "1.0 Cr");
        assert_eq!(format_amount(25000000.0), "2.5 Cr");
    }
}
