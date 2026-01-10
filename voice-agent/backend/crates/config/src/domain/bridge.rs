//! Domain Bridge - Adapts YAML configuration to core trait implementations
//!
//! This module provides adapters that implement the core traits using
//! the loaded YAML configuration. This is the bridge between config-driven
//! domain knowledge and domain-agnostic abstractions.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_config::domain::{MasterDomainConfig, DomainBridge};
//!
//! // Load config for a domain (domain_id from DOMAIN_ID env var)
//! let config = MasterDomainConfig::load(&domain_id, "config/")?;
//! let bridge = DomainBridge::new(Arc::new(config));
//!
//! // Get trait implementations
//! let calculator = bridge.calculator();
//! let scorer = bridge.lead_scoring();
//! ```

use std::sync::Arc;

use voice_agent_core::traits::{
    // Calculator
    ConfigDrivenCalculator, DomainCalculator, QualityFactor, RateTier,
    // Scoring
    ConfigLeadScoring, LeadScoringStrategy, ScoringConfig as CoreScoringConfig,
    // Competitors
    CompetitorAnalyzer, CompetitorInfo, ConfigCompetitorAnalyzer,
    ComparisonPoint,
    // Objections (P13 FIX)
    AcreResponse, ConfigObjectionDefinition, ConfigObjectionHandler, ObjectionHandler,
    // Segments
    ConfigSegmentDefinition, ConfigSegmentDetector, SegmentDetector,
    // Goals
    ConfigGoalDefinition, ConfigGoalSchema, ConversationGoalSchema,
    // P20 FIX: Config-driven traits
    // Features
    ConfigFeatureDefinition, ConfigFeatureProvider, FeatureProvider,
    // Objections (P20 FIX - new interface)
    ConfigAcreResponse, ConfigObjectionDef, ConfigObjectionProvider, DetectionPattern,
    ObjectionProvider,
    // Tool Arguments
    ConfigToolArgumentProvider, IntentToolMapping, ToolArgumentProvider, ToolDefaults,
    // Lead Classification
    ClassificationRule, ConfigLeadClassifier, EscalationTriggerConfig, LeadClassifier,
    QualificationThreshold,
    // P23 FIX: Signal system
    SignalStore,
};

use super::MasterDomainConfig;

/// Domain bridge - creates trait implementations from config
pub struct DomainBridge {
    config: Arc<MasterDomainConfig>,
}

impl DomainBridge {
    /// Create a new bridge from config
    pub fn new(config: Arc<MasterDomainConfig>) -> Self {
        Self { config }
    }

    /// Get the underlying config
    pub fn config(&self) -> &MasterDomainConfig {
        &self.config
    }

    /// Get a domain calculator implementation from config
    ///
    /// Returns `Arc<dyn DomainCalculator>` for polymorphic usage, dependency injection,
    /// and runtime implementation swapping.
    pub fn calculator(&self) -> Arc<dyn DomainCalculator> {
        // Convert config rate tiers to core rate tiers
        let rate_tiers: Vec<RateTier> = self.config.constants.interest_rates.tiers
            .iter()
            .enumerate()
            .map(|(i, t)| RateTier {
                max_amount: t.max_amount,
                rate: t.rate,
                label: format!("Tier {}", i + 1),
            })
            .collect();

        // Convert variant factors to quality factors (e.g., purity for gold)
        let quality_factors: Vec<QualityFactor> = self.config.constants.variant_factors
            .iter()
            .map(|(id, factor)| QualityFactor {
                id: id.clone(),
                display_name: format!("{} Variant", id),
                factor: *factor,
            })
            .collect();

        Arc::new(ConfigDrivenCalculator::new(
            rate_tiers,
            quality_factors,
            self.config.constants.ltv_percent,
            self.config.constants.asset_price_per_unit,
            self.config.constants.interest_rates.base_rate,
            self.config.constants.loan_limits.min,
            self.config.constants.loan_limits.max,
            self.config.constants.processing_fee_percent,
            0.0, // foreclosure fee
        ))
    }

    /// Get a lead scoring strategy implementation from config
    ///
    /// Returns `Arc<dyn LeadScoringStrategy>` for polymorphic usage.
    pub fn lead_scoring(&self) -> Arc<dyn LeadScoringStrategy> {
        let scoring = &self.config.scoring;
        let thresholds = &scoring.qualification_thresholds;

        // Note: config thresholds use >= semantics (score >= 30 is warm)
        // Core thresholds use <= semantics for upper bounds (score <= 29 is cold)
        Arc::new(ConfigLeadScoring::new(CoreScoringConfig {
            // Convert >= thresholds to <= upper bounds
            cold_max: thresholds.warm.saturating_sub(1),
            warm_max: thresholds.hot.saturating_sub(1),
            hot_max: thresholds.qualified.saturating_sub(1),
            qualified_min: thresholds.qualified,

            // Urgency scores
            urgency_has_signal_score: scoring.urgency.has_signal_score,
            urgency_keyword_score: scoring.urgency.per_keyword_score,
            urgency_max_keywords: scoring.urgency.max_keywords,
            urgency_max_score: scoring.urgency.max_score,

            // Engagement scores
            engagement_per_turn_score: scoring.engagement.per_turn_score,
            engagement_max_turns: scoring.engagement.max_turns,
            engagement_per_question_score: scoring.engagement.per_question_score,
            engagement_max_questions: scoring.engagement.max_questions,
            engagement_rates_inquiry_score: scoring.engagement.rates_inquiry_score,
            engagement_comparison_score: scoring.engagement.comparison_score,
            engagement_max_score: scoring.engagement.max_score,

            // Information scores (P21 FIX: now asset_details_score for domain-agnosticism)
            info_contact_score: scoring.information.contact_info_score,
            info_asset_details_score: scoring.information.asset_details_score,
            info_loan_amount_score: scoring.information.loan_amount_score,
            info_specific_requirements_score: scoring.information.specific_requirements_score,
            info_max_score: scoring.information.max_score,

            // Intent scores
            intent_to_proceed_score: scoring.intent.intent_to_proceed_score,
            intent_callback_score: scoring.intent.callback_request_score,
            intent_branch_visit_score: scoring.intent.branch_visit_score,
            intent_max_score: scoring.intent.max_score,

            // Penalties
            penalty_disinterest: scoring.penalties.disinterest,
            penalty_competitor_preference: scoring.penalties.competitor_preference,
            penalty_human_request: scoring.penalties.human_request,
            penalty_per_unresolved_objection: scoring.penalties.per_unresolved_objection,

            // Escalation thresholds
            escalation_max_objections: scoring.escalation.max_objections,
            escalation_max_stalled_turns: scoring.escalation.max_stalled_turns,
            escalation_high_value_threshold: scoring.escalation.high_value_threshold,

            // MQL criteria
            mql_min_engagement_turns: 3,

            // Urgency keywords from config
            urgency_keywords_en: scoring.urgency.keywords.get("en")
                .cloned()
                .unwrap_or_default(),
            urgency_keywords_hi: scoring.urgency.keywords.get("hi")
                .cloned()
                .unwrap_or_default(),
        }))
    }

    /// Get a competitor analyzer implementation from config
    ///
    /// Returns `Arc<dyn CompetitorAnalyzer>` for polymorphic usage.
    pub fn competitor_analyzer(&self) -> Arc<dyn CompetitorAnalyzer> {
        let competitors: Vec<CompetitorInfo> = self.config.competitors_config.competitors
            .iter()
            .map(|(id, entry)| {
                // Pass the string type ID directly from config
                let comp_type_id = entry.competitor_type.to_lowercase();

                let mut info = CompetitorInfo::new(
                    id.clone(),
                    entry.display_name.clone(),
                    entry.typical_rate,
                    comp_type_id,
                );

                // Add rate range if available
                if let Some(ref range) = entry.rate_range {
                    info = info.with_rate_range(range.min, range.max);
                }

                info = info.with_ltv(entry.ltv_percent)
                    .with_aliases(entry.aliases.clone())
                    .with_strengths(entry.strengths.clone())
                    .with_weaknesses(entry.weaknesses.clone());

                info
            })
            .collect();

        let comparison_points: Vec<ComparisonPoint> = self.config.competitors_config
            .comparison_points
            .iter()
            .map(|p| ComparisonPoint {
                category: p.category.clone(),
                our_advantage: p.our_advantage.clone(),
                highlight: p.highlight,
            })
            .collect();

        // Build rate tiers for our rates
        let rate_tiers: Vec<(f64, f64)> = self.config.constants.interest_rates.tiers
            .iter()
            .map(|t| (t.max_amount.unwrap_or(f64::MAX), t.rate))
            .collect();

        // Get default unknown competitor rate from config
        let default_unknown_rate = self.config.competitors_config.defaults.nbfc_rate;

        // P23 FIX: Build type default rates from entity_types.yaml if available,
        // falling back to competitors.yaml defaults for backward compatibility
        let type_default_rates = if !self.config.entities.competitor_types.is_empty() {
            // Use config-driven competitor types from entity_types.yaml
            self.config.entities.competitor_type_default_rates()
        } else {
            // Fallback to competitors.yaml defaults
            let mut rates = std::collections::HashMap::new();
            rates.insert("bank".to_string(), self.config.competitors_config.defaults.bank_rate);
            rates.insert("nbfc".to_string(), self.config.competitors_config.defaults.nbfc_rate);
            rates.insert("informal".to_string(), self.config.competitors_config.defaults.local_lender_rate);
            rates
        };

        // Build comparison templates from config (with fallback)
        let comparison_templates = self.config.competitors_config.comparison_message_templates.clone();

        // Get currency symbol from config
        let currency_symbol = self.config.currency.symbol.clone();

        Arc::new(ConfigCompetitorAnalyzer::new(
            competitors,
            comparison_points,
            self.config.constants.interest_rates.base_rate,
            rate_tiers,
            default_unknown_rate,
            type_default_rates,
            comparison_templates,
            currency_symbol,
        ))
    }

    /// Get an objection handler implementation from config
    ///
    /// Returns `Arc<dyn ObjectionHandler>` for polymorphic usage.
    pub fn objection_handler(&self) -> Arc<dyn ObjectionHandler> {
        let objections: Vec<ConfigObjectionDefinition> = self.config.objections.objections
            .iter()
            .map(|(id, entry)| {
                let mut def = ConfigObjectionDefinition::new(
                    id.clone(),
                    entry.display_name.clone(),
                    entry.description.clone(),
                );

                // Add patterns for each language
                for (lang, patterns) in &entry.patterns {
                    def = def.with_patterns(lang, patterns.clone());
                }

                // Add ACRE responses for each language
                for (lang, response) in &entry.responses {
                    def = def.with_response(
                        lang,
                        AcreResponse::new(
                            response.acknowledge.clone(),
                            response.reframe.clone(),
                            response.evidence.clone(),
                            response.call_to_action.clone(),
                        ),
                    );
                }

                def
            })
            .collect();

        Arc::new(ConfigObjectionHandler::new(objections))
    }

    /// Get a segment detector implementation from config
    ///
    /// Returns `Arc<dyn SegmentDetector>` for polymorphic usage.
    pub fn segment_detector(&self) -> Arc<dyn SegmentDetector> {
        let segments: Vec<ConfigSegmentDefinition> = self.config.segments.segments
            .iter()
            .map(|(id, entry)| {
                let priority = entry.priority.try_into().unwrap_or(5);
                let mut def = ConfigSegmentDefinition::new(
                    id.clone(),
                    entry.display_name.clone(),
                    entry.description.clone(),
                    priority,
                );

                // Add text patterns from detection config
                if let Some(ref patterns) = entry.detection.text_patterns {
                    for (lang, pats) in patterns {
                        def = def.with_text_patterns(lang, pats.clone());
                    }
                }

                // Add numeric thresholds
                if let Some(ref thresholds) = entry.detection.numeric_thresholds {
                    let thresh_map: std::collections::HashMap<String, f64> = thresholds
                        .iter()
                        .filter_map(|(k, v)| v.min.map(|min| (k.clone(), min)))
                        .collect();
                    def = def.with_numeric_thresholds(thresh_map);
                }

                // Add features
                def = def.with_features(entry.features.clone());

                def
            })
            .collect();

        Arc::new(ConfigSegmentDetector::new(
            segments,
            self.config.segments.default_segment.clone(),
        ))
    }

    /// Get a conversation goal schema implementation from config
    ///
    /// Returns `Arc<dyn ConversationGoalSchema>` for polymorphic usage.
    pub fn goal_schema(&self) -> Arc<dyn ConversationGoalSchema> {
        // Use the new GoalsConfig from goals.yaml
        let goals: Vec<ConfigGoalDefinition> = self.config.goals.goals
            .iter()
            .map(|(id, entry)| {
                let mut def = ConfigGoalDefinition::new(
                    id.clone(),
                    entry.display_name.clone(),
                    entry.description.clone(),
                )
                .with_required_slots(entry.required_slots.clone())
                .with_optional_slots(entry.optional_slots.clone())
                .with_priority(entry.priority);

                if let Some(ref tool) = entry.completion_tool {
                    def = def.with_completion_tool(tool.clone());
                }

                // Add slot prompts (use English prompts for now)
                if let Some(ref prompts) = entry.slot_prompts {
                    for (slot, lang_prompts) in prompts {
                        if let Some(prompt) = lang_prompts.get("en") {
                            def = def.with_slot_prompt(slot.clone(), prompt.clone());
                        }
                    }
                }

                def
            })
            .collect();

        // Intent mappings are already intent->goal in goals.yaml
        let intent_mapping = self.config.goals.intent_mappings.clone();

        Arc::new(ConfigGoalSchema::new(goals, intent_mapping))
    }

    // =========================================================================
    // P20 FIX: New Config-Driven Trait Factory Methods
    // =========================================================================

    /// Get a feature provider implementation from config
    ///
    /// Returns `Arc<dyn FeatureProvider>` for config-driven feature management.
    /// Replaces hardcoded Feature enum in personalization/adaptation.rs.
    pub fn feature_provider(&self) -> Arc<dyn FeatureProvider> {
        let features: Vec<ConfigFeatureDefinition> = self
            .config
            .features
            .features
            .iter()
            .map(|(id, entry)| {
                let mut display_name = entry.display_name.clone();
                // Convert single description to description by language
                let mut description = std::collections::HashMap::new();
                if !entry.description.is_empty() {
                    description.insert("en".to_string(), entry.description.clone());
                }

                ConfigFeatureDefinition {
                    id: id.clone(),
                    display_name,
                    description,
                    enabled: true,
                    icon: None,
                    badge: None,
                    segment_overrides: std::collections::HashMap::new(),
                }
            })
            .collect();

        // Convert value_propositions from Vec<String> to HashMap<String, Vec<String>>
        let value_propositions: std::collections::HashMap<String, std::collections::HashMap<String, Vec<String>>> =
            self.config.features.value_propositions
                .iter()
                .map(|(segment, props)| {
                    let mut by_lang = std::collections::HashMap::new();
                    by_lang.insert("en".to_string(), props.clone());
                    (segment.clone(), by_lang)
                })
                .collect();

        Arc::new(ConfigFeatureProvider::new(
            features,
            self.config.features.segment_features.clone(),
            value_propositions,
        ))
    }

    /// Get an objection provider implementation from config (P20 FIX)
    ///
    /// Returns `Arc<dyn ObjectionProvider>` for config-driven objection handling.
    /// Replaces hardcoded Objection enum in personalization/adaptation.rs.
    pub fn objection_provider(&self) -> Arc<dyn ObjectionProvider> {
        let objections: Vec<ConfigObjectionDef> = self
            .config
            .objections
            .objections
            .iter()
            .map(|(id, entry)| {
                // Convert detection patterns
                let detection: std::collections::HashMap<String, DetectionPattern> = entry
                    .patterns
                    .iter()
                    .map(|(lang, patterns)| {
                        (
                            lang.clone(),
                            DetectionPattern {
                                patterns: patterns.clone(),
                                regex_patterns: vec![],
                                confidence_boosts: vec![],
                            },
                        )
                    })
                    .collect();

                // Convert responses to ACRE format
                let responses: std::collections::HashMap<String, ConfigAcreResponse> = entry
                    .responses
                    .iter()
                    .map(|(lang, response)| {
                        (
                            lang.clone(),
                            ConfigAcreResponse {
                                acknowledge: response.acknowledge.clone(),
                                clarify: Some(response.reframe.clone()),
                                respond: response.evidence.clone(),
                                engage: Some(response.call_to_action.clone()),
                            },
                        )
                    })
                    .collect();

                // Create display_name map
                let mut display_name = std::collections::HashMap::new();
                display_name.insert("en".to_string(), entry.display_name.clone());

                ConfigObjectionDef {
                    id: id.clone(),
                    aliases: vec![],
                    display_name,
                    detection,
                    responses,
                    segment_responses: std::collections::HashMap::new(),
                    highlight_feature: None,
                    enabled: true,
                }
            })
            .collect();

        Arc::new(ConfigObjectionProvider::new(objections))
    }

    /// Get a tool argument provider implementation from config
    ///
    /// Returns `Arc<dyn ToolArgumentProvider>` for config-driven tool defaults.
    /// Replaces hardcoded fallbacks in agent/tools.rs.
    pub fn tool_argument_provider(&self) -> Arc<dyn ToolArgumentProvider> {
        // Convert intent-to-tool mappings from config
        let mappings: Vec<IntentToolMapping> = self
            .config
            .tools
            .intent_to_tool
            .iter()
            .map(|(intent, mapping)| IntentToolMapping {
                intent: intent.clone(),
                aliases: mapping.aliases.clone(),
                tool: mapping.tool.clone(),
                required_slots: mapping.required_slots.clone(),
                optional_slots: vec![], // Not in config yet - will be added if needed
                enabled: true,          // All mappings in config are enabled
            })
            .collect();

        // Convert tool defaults from config
        let defaults: Vec<ToolDefaults> = self
            .config
            .tools
            .tool_defaults
            .iter()
            .map(|(tool_name, default_values)| {
                let defaults_map: std::collections::HashMap<String, serde_json::Value> =
                    default_values
                        .iter()
                        .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                        .collect();

                // Get argument mapping for this tool if available
                let argument_mapping = self
                    .config
                    .tools
                    .argument_mappings
                    .get(tool_name)
                    .cloned()
                    .unwrap_or_default();

                ToolDefaults {
                    tool: tool_name.clone(),
                    defaults: defaults_map,
                    argument_mapping,
                }
            })
            .collect();

        Arc::new(ConfigToolArgumentProvider::new(mappings, defaults))
    }

    /// Get a lead classifier implementation from config
    ///
    /// Returns `Arc<dyn LeadClassifier>` for config-driven lead classification.
    /// Replaces hardcoded MQL/SQL rules in lead_scoring.rs.
    pub fn lead_classifier(&self) -> Arc<dyn LeadClassifier> {
        // Build SQL classification rule from config
        // SQL typically requires: urgency + contact info + specific requirements
        let sql_rule = ClassificationRule {
            required_flags: vec![
                "has_urgency_signal".to_string(),
                "provided_contact_info".to_string(),
                "has_specific_requirements".to_string(),
            ],
            any_of_flags: vec![],
            thresholds: std::collections::HashMap::new(),
        };

        // Build MQL classification rule from config
        // MQL typically requires: engagement turns >= 3 AND (asked about rates OR comparison)
        let mut mql_thresholds = std::collections::HashMap::new();
        mql_thresholds.insert(
            "engagement_turns".to_string(),
            self.config.scoring.engagement.max_turns.min(3) as u32,
        );

        let mql_rule = ClassificationRule {
            required_flags: vec![],
            any_of_flags: vec![
                "asked_about_rates".to_string(),
                "asked_for_comparison".to_string(),
            ],
            thresholds: mql_thresholds,
        };

        // Build qualification thresholds from config
        let thresholds = &self.config.scoring.qualification_thresholds;
        let mut qualification_thresholds = std::collections::HashMap::new();

        qualification_thresholds.insert(
            "cold".to_string(),
            QualificationThreshold {
                min_score: 0,
                max_score: thresholds.warm.saturating_sub(1),
            },
        );
        qualification_thresholds.insert(
            "warm".to_string(),
            QualificationThreshold {
                min_score: thresholds.warm,
                max_score: thresholds.hot.saturating_sub(1),
            },
        );
        qualification_thresholds.insert(
            "hot".to_string(),
            QualificationThreshold {
                min_score: thresholds.hot,
                max_score: thresholds.qualified.saturating_sub(1),
            },
        );
        qualification_thresholds.insert(
            "qualified".to_string(),
            QualificationThreshold {
                min_score: thresholds.qualified,
                max_score: 100,
            },
        );

        // Build escalation triggers from config
        let escalation_triggers = vec![
            EscalationTriggerConfig {
                id: "high_objections".to_string(),
                signal: "unresolved_objection_count".to_string(),
                threshold: Some(self.config.scoring.escalation.max_objections),
                priority: 1,
                message: "Too many unresolved objections".to_string(),
            },
            EscalationTriggerConfig {
                id: "stalled".to_string(),
                signal: "stalled_turns".to_string(),
                threshold: Some(self.config.scoring.escalation.max_stalled_turns),
                priority: 2,
                message: "Conversation stalled".to_string(),
            },
            EscalationTriggerConfig {
                id: "human_request".to_string(),
                signal: "human_request".to_string(),
                threshold: None,
                priority: 0,
                message: "Customer requested human agent".to_string(),
            },
        ];

        Arc::new(ConfigLeadClassifier::new(
            sql_rule,
            mql_rule,
            qualification_thresholds,
            escalation_triggers,
        ))
    }

    /// P23 FIX: Get a signal store pre-populated with definitions from config
    ///
    /// Returns a `SignalStore` initialized with signal definitions from signals.yaml.
    /// This enables config-driven signal validation and max value capping.
    ///
    /// # Usage
    ///
    /// ```ignore
    /// let signal_store = bridge.signal_store();
    /// signal_store.set_boolean("has_urgency", true);
    /// signal_store.increment_counter("engagement_turns");
    /// ```
    pub fn signal_store(&self) -> SignalStore {
        let definitions = self.config.signals.to_core_definitions();
        SignalStore::with_definitions(definitions)
    }

    /// P23 FIX: Get the signals config directly for advanced use cases
    ///
    /// Use this when you need access to signal weights, categories, or
    /// scoring thresholds for custom scoring logic.
    pub fn signals_config(&self) -> &super::signals::SignalsConfig {
        &self.config.signals
    }

    /// P23 FIX: Get the scoring config directly
    ///
    /// Returns the ScoringConfig for use with LeadScoringEngine.set_scoring_config()
    pub fn scoring_config(&self) -> Arc<super::ScoringConfig> {
        Arc::new(self.config.scoring.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_creation() {
        let config = MasterDomainConfig::default();
        let bridge = DomainBridge::new(Arc::new(config));

        // Verify calculator has defaults
        let calc = bridge.calculator();
        assert!(calc.ltv_percent() >= 0.0);
    }

    #[test]
    fn test_bridge_lead_scoring() {
        let config = MasterDomainConfig::default();
        let bridge = DomainBridge::new(Arc::new(config));

        let scorer = bridge.lead_scoring();
        let (cold_max, warm_max, hot_max, qualified_min) = scorer.thresholds();

        // Verify thresholds make sense
        assert!(cold_max < warm_max);
        assert!(warm_max < hot_max);
        assert!(hot_max < qualified_min);
    }
}
