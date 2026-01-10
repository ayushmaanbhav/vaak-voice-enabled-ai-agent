//! Master Domain Configuration
//!
//! Loads and merges hierarchical YAML configuration:
//! - Base defaults (config/base/defaults.yaml)
//! - Domain-specific (config/domains/{domain}/domain.yaml)

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;

use crate::ConfigError;
use super::branches::BranchesConfig;
use super::competitors::CompetitorsConfig;
use super::documents::DocumentsConfig;
use super::entities::EntitiesConfig;
use super::features::FeaturesConfig;
use super::goals::GoalsConfig;
use super::intents::IntentsConfig;
use super::objections::ObjectionsConfig;
use super::personas::PersonasConfig;
use super::prompts::PromptsConfig;
use super::scoring::ScoringConfig;
use super::segments::SegmentsConfig;
use super::signals::SignalsConfig;
use super::slots::SlotsConfig;
use super::sms_templates::SmsTemplatesConfig;
use super::stages::StagesConfig;
use super::tool_responses::ToolResponsesConfig;
use super::tools::ToolsConfig;
use super::vocabulary::FullVocabularyConfig;

/// Brand configuration - domain-agnostic
///
/// P16 FIX: Renamed bank_name to company_name for domain-agnostic design.
/// The company_name field is the organization offering the service.
/// The product_name field is the specific product/service being offered.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrandConfig {
    /// Company/organization name (e.g., "Kotak Mahindra Bank", "ABC Corp")
    #[serde(alias = "bank_name")]  // Backwards compatibility
    pub company_name: String,
    /// Product/service name (e.g., "Gold Loan", "Insurance")
    #[serde(default)]
    pub product_name: String,
    /// AI agent name
    pub agent_name: String,
    /// Agent role/title for persona (e.g., "Advisor", "Assistant")
    #[serde(default)]
    pub agent_role: String,
    /// Contact helpline number
    pub helpline: String,
    /// Website URL
    #[serde(default)]
    pub website: String,
}

/// Interest rate tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateTier {
    /// Tier name (e.g., "Standard", "Premium", "Elite")
    #[serde(default)]
    pub name: String,
    /// Maximum amount for this tier (null = unlimited)
    pub max_amount: Option<f64>,
    /// Interest rate percentage
    pub rate: f64,
}

/// Interest rates configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InterestRatesConfig {
    #[serde(default)]
    pub tiers: Vec<RateTier>,
    #[serde(default)]
    pub base_rate: f64,
}

/// Loan limits configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoanLimitsConfig {
    pub min: f64,
    pub max: f64,
}

/// Domain constants (source of truth for business values)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainConstants {
    #[serde(default)]
    pub interest_rates: InterestRatesConfig,
    #[serde(default)]
    pub ltv_percent: f64,
    #[serde(default)]
    pub loan_limits: LoanLimitsConfig,
    #[serde(default)]
    pub processing_fee_percent: f64,
    /// Asset price per unit (e.g., gold price per gram for gold loan domain)
    #[serde(default, alias = "gold_price_per_gram")]
    pub asset_price_per_unit: f64,
    /// Variant factors (e.g., purity factors for gold: K24=1.0, K22=0.916)
    #[serde(default, alias = "purity_factors")]
    pub variant_factors: HashMap<String, f64>,
}

impl DomainConstants {
    /// Legacy accessor for gold_price_per_gram
    pub fn gold_price_per_gram(&self) -> f64 {
        self.asset_price_per_unit
    }

    /// Legacy accessor for purity_factors
    pub fn purity_factors(&self) -> &HashMap<String, f64> {
        &self.variant_factors
    }
}

/// Competitor configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompetitorEntry {
    pub display_name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub typical_rate: f64,
    #[serde(default)]
    pub ltv_percent: f64,
    #[serde(default)]
    pub competitor_type: String,
}

/// Product variant configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProductVariant {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub min_amount: Option<f64>,
    #[serde(default)]
    pub max_amount: Option<f64>,
    #[serde(default)]
    pub features: Vec<String>,
}

/// High-value customer configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HighValueConfig {
    #[serde(default)]
    pub amount_threshold: f64,
    #[serde(default)]
    pub weight_threshold_grams: f64,
    #[serde(default)]
    pub features: Vec<String>,
}

/// P15 FIX: Domain vocabulary configuration for text processing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VocabularyConfig {
    /// Domain-specific terms to preserve
    #[serde(default)]
    pub terms: Vec<String>,
    /// Common phrases in domain
    #[serde(default)]
    pub phrases: Vec<String>,
    /// Abbreviations and their expansions
    #[serde(default)]
    pub abbreviations: HashMap<String, String>,
    /// Entity types to preserve
    #[serde(default)]
    pub preserve_entities: Vec<String>,
}

/// Contextual correction rule for phonetic corrector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextualRule {
    /// Previous word context
    pub context: String,
    /// Error word to correct
    pub error: String,
    /// Corrected value
    pub correction: String,
}

/// Phonetic corrector configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneticCorrectorParams {
    /// Maximum edit distance for fuzzy matching
    #[serde(default = "default_max_edit_distance")]
    pub max_edit_distance: i64,
    /// Minimum word length to attempt correction
    #[serde(default = "default_min_word_length")]
    pub min_word_length: usize,
    /// Whether to fix sentence-start errors (e.g., "Why" -> "I")
    #[serde(default = "default_true")]
    pub fix_sentence_start: bool,
}

fn default_max_edit_distance() -> i64 { 2 }
fn default_min_word_length() -> usize { 3 }
fn default_true() -> bool { true }

impl Default for PhoneticCorrectorParams {
    fn default() -> Self {
        Self {
            max_edit_distance: 2,
            min_word_length: 3,
            fix_sentence_start: true,
        }
    }
}

/// P16 FIX: Phonetic ASR error correction configuration
/// Moved from hardcoded defaults to config-driven domain-specific corrections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PhoneticCorrectionsConfig {
    /// Direct confusion rules: misspelling -> correct
    #[serde(default)]
    pub confusion_rules: HashMap<String, String>,
    /// Contextual rules: (prev_word, error_word) -> correction
    #[serde(default)]
    pub contextual_rules: Vec<ContextualRule>,
    /// Phrase-level corrections: phrase -> replacement
    #[serde(default)]
    pub phrase_rules: HashMap<String, String>,
    /// Configuration parameters
    #[serde(default)]
    pub config: PhoneticCorrectorParams,
}

/// Query expansion configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryExpansionSettings {
    /// Maximum number of expansions per term
    #[serde(default = "default_max_expansions")]
    pub max_expansions: usize,
    /// Enable synonym expansion
    #[serde(default = "default_true")]
    pub use_synonyms: bool,
    /// Enable transliteration expansion
    #[serde(default = "default_true")]
    pub use_transliterations: bool,
    /// Enable stopword filtering
    #[serde(default = "default_true")]
    pub enable_stopword_filter: bool,
}

fn default_max_expansions() -> usize {
    3
}

impl Default for QueryExpansionSettings {
    fn default() -> Self {
        Self {
            max_expansions: 3,
            use_synonyms: true,
            use_transliterations: true,
            enable_stopword_filter: true,
        }
    }
}

/// Query expansion configuration for RAG
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryExpansionConfig {
    /// Configuration settings
    #[serde(default)]
    pub config: QueryExpansionSettings,
    /// Stopwords to filter from queries
    #[serde(default)]
    pub stopwords: Vec<String>,
    /// Synonym mappings (term -> list of synonyms)
    #[serde(default)]
    pub synonyms: HashMap<String, Vec<String>>,
    /// Hindi-Roman transliterations
    #[serde(default)]
    pub transliterations: HashMap<String, Vec<String>>,
}

/// Domain boost term entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainBoostTermEntry {
    /// The term to boost
    pub term: String,
    /// Category of the term
    pub category: String,
    /// Boost multiplier
    pub boost: f64,
    /// Related terms
    #[serde(default)]
    pub related: Vec<String>,
}

/// Domain boosting configuration for RAG
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainBoostConfig {
    /// Default boost multiplier
    #[serde(default = "default_boost")]
    pub default_boost: f64,
    /// Category-specific boost multipliers
    #[serde(default)]
    pub category_boosts: HashMap<String, f64>,
    /// Domain-specific terms with boosting
    #[serde(default)]
    pub terms: Vec<DomainBoostTermEntry>,
}

fn default_boost() -> f64 {
    1.0
}

// ============================================================================
// P18 FIX: Memory Compressor Configuration (Domain-Agnostic)
// ============================================================================

/// Domain keywords configuration for memory compressor
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainKeywordsConfig {
    /// Product/service specific terms (e.g., loan, insurance, policy)
    #[serde(default)]
    pub product_terms: Vec<String>,
    /// Unit terms (e.g., gram, lakh, percent)
    #[serde(default)]
    pub unit_terms: Vec<String>,
    /// Quality/variant terms (e.g., purity grades, quality tiers)
    #[serde(default)]
    pub quality_terms: Vec<String>,
    /// Regional language terms (Hindi, Tamil, etc.)
    #[serde(default)]
    pub regional_terms: Vec<String>,
    /// Whether to automatically include competitor names from config
    #[serde(default)]
    pub include_competitor_names: bool,
    /// Whether to automatically include brand name from config
    #[serde(default)]
    pub include_brand_name: bool,
}

/// Entity pattern configuration for memory compressor
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EntityPatternConfig {
    /// Pattern strings to match for this entity type
    #[serde(default)]
    pub patterns: Vec<String>,
}

/// Intent keyword configuration for memory compressor
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IntentKeywordConfig {
    /// Keywords/patterns to match for this intent
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Whether to include competitor aliases as patterns
    #[serde(default)]
    pub include_competitor_aliases: bool,
}

/// Slot display mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlotDisplayConfig {
    /// Display label for this slot type
    #[serde(default)]
    pub display: String,
    /// Alternative slot names that map to this display
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// Memory compressor configuration (config-driven, domain-agnostic)
///
/// P18 FIX: All domain-specific keywords, entity patterns, and intent mappings
/// are now loaded from config instead of being hardcoded.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryCompressorConfig {
    /// Domain keywords for sentence scoring
    #[serde(default)]
    pub domain_keywords: DomainKeywordsConfig,
    /// Entity patterns for extraction (entity_type -> patterns)
    #[serde(default)]
    pub entity_patterns: HashMap<String, EntityPatternConfig>,
    /// Intent keywords for relevance scoring (intent -> keywords)
    #[serde(default)]
    pub intent_keywords: HashMap<String, IntentKeywordConfig>,
    /// Slot display mappings (slot_name -> display config)
    #[serde(default)]
    pub slot_display_mappings: HashMap<String, SlotDisplayConfig>,
    /// Filler patterns by language (language_code -> patterns)
    #[serde(default)]
    pub filler_patterns: HashMap<String, Vec<String>>,
}

// ============================================================================
// P18 FIX: Currency Configuration (Domain-Agnostic)
// ============================================================================

/// Display unit for currency amounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayUnit {
    /// Unit name (e.g., "lakh", "thousand", "million")
    #[serde(default)]
    pub name: String,
    /// Amount this unit represents (e.g., 100000 for lakh)
    #[serde(default = "default_unit_amount")]
    pub amount: f64,
}

fn default_unit_amount() -> f64 {
    1.0
}

impl Default for DisplayUnit {
    fn default() -> Self {
        Self {
            name: "unit".to_string(),
            amount: 1.0,
        }
    }
}

/// Display units configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DisplayUnitsConfig {
    /// Primary unit for savings calculations (e.g., lakh)
    #[serde(default)]
    pub savings_unit: DisplayUnit,
    /// Unit for large amounts (e.g., crore)
    #[serde(default)]
    pub large_unit: DisplayUnit,
    /// Unit for small amounts
    #[serde(default)]
    pub small_unit: DisplayUnit,
}

fn default_currency_code() -> String {
    "INR".to_string()
}

fn default_currency_symbol() -> String {
    "₹".to_string()
}

/// Currency configuration (domain-agnostic)
///
/// P18 FIX: Currency and display unit configuration is now loaded from config
/// instead of hardcoding "lakh" (100,000) as the savings unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyConfig {
    /// ISO 4217 currency code
    #[serde(default = "default_currency_code")]
    pub code: String,
    /// Currency symbol
    #[serde(default = "default_currency_symbol")]
    pub symbol: String,
    /// P2.6 FIX: Suffix for JSON field names (e.g., "inr" for "amount_inr")
    /// This replaces hardcoded "_inr" suffixes in tool output fields.
    #[serde(default = "default_field_suffix")]
    pub field_suffix: String,
    /// Display units for different amount ranges
    #[serde(default)]
    pub display_units: DisplayUnitsConfig,
}

fn default_field_suffix() -> String {
    "inr".to_string()
}

impl Default for CurrencyConfig {
    fn default() -> Self {
        Self {
            code: default_currency_code(),
            symbol: default_currency_symbol(),
            field_suffix: default_field_suffix(),
            display_units: DisplayUnitsConfig::default(),
        }
    }
}

/// Master domain configuration - the complete config for a domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterDomainConfig {
    /// Domain identifier (matches directory name in config/domains/)
    pub domain_id: String,
    /// Human-readable domain name
    pub display_name: String,
    /// Version
    #[serde(default = "default_version")]
    pub version: String,
    /// Brand configuration
    #[serde(default)]
    pub brand: BrandConfig,
    /// Business constants
    #[serde(default)]
    pub constants: DomainConstants,
    /// Competitors
    #[serde(default)]
    pub competitors: HashMap<String, CompetitorEntry>,
    /// Product variants
    #[serde(default)]
    pub products: HashMap<String, ProductVariant>,
    /// High-value customer config
    #[serde(default)]
    pub high_value: HighValueConfig,
    /// P15 FIX: Domain vocabulary for text processing
    #[serde(default)]
    pub vocabulary: VocabularyConfig,
    /// P16 FIX: Phonetic ASR error correction rules (loaded from domain.yaml)
    #[serde(default)]
    pub phonetic_corrections: PhoneticCorrectionsConfig,
    /// P16 FIX: Domain-specific terms for LLM relevance scoring in speculative execution
    /// Used by SpeculativeExecutor.estimate_domain_relevance()
    #[serde(default)]
    pub relevance_terms: Vec<String>,
    /// P17 FIX: Query expansion configuration for RAG
    #[serde(default)]
    pub query_expansion: QueryExpansionConfig,
    /// P17 FIX: Domain boosting configuration for RAG
    #[serde(default)]
    pub domain_boost: DomainBoostConfig,
    /// P18 FIX: Memory compressor configuration (domain-agnostic)
    #[serde(default)]
    pub memory_compressor: MemoryCompressorConfig,
    /// P18 FIX: Currency and display unit configuration (domain-agnostic)
    #[serde(default)]
    pub currency: CurrencyConfig,
    /// P18 FIX: RAG collection name for this domain (domain-agnostic)
    /// Defaults to "{domain_id}_knowledge" pattern
    #[serde(default)]
    pub rag_collection_name: Option<String>,
    /// Slot definitions for DST (loaded from slots.yaml)
    #[serde(skip)]
    pub slots: SlotsConfig,
    /// Stage definitions for conversation flow (loaded from stages.yaml)
    #[serde(skip)]
    pub stages: StagesConfig,
    /// Lead scoring configuration (loaded from scoring.yaml)
    #[serde(skip)]
    pub scoring: ScoringConfig,
    /// Tool schemas for LLM function calling (loaded from tools/schemas.yaml)
    #[serde(skip)]
    pub tools: ToolsConfig,
    /// Prompt templates (loaded from prompts/system.yaml)
    #[serde(skip)]
    pub prompts: PromptsConfig,
    /// Objection handling configuration (loaded from objections.yaml)
    #[serde(skip)]
    pub objections: ObjectionsConfig,
    /// Branch data (loaded from tools/branches.yaml)
    #[serde(skip)]
    pub branches: BranchesConfig,
    /// SMS templates (loaded from tools/sms_templates.yaml)
    #[serde(skip)]
    pub sms_templates: SmsTemplatesConfig,
    /// Extended competitor data (loaded from competitors.yaml)
    #[serde(skip)]
    pub competitors_config: CompetitorsConfig,
    /// Customer segment definitions (loaded from segments.yaml)
    #[serde(skip)]
    pub segments: SegmentsConfig,
    /// Goals configuration (loaded from goals.yaml)
    #[serde(skip)]
    pub goals: GoalsConfig,
    /// Features configuration (loaded from features.yaml)
    #[serde(skip)]
    pub features: FeaturesConfig,
    /// P16 FIX: Document requirements (loaded from tools/documents.yaml)
    #[serde(skip)]
    pub documents: DocumentsConfig,
    /// P16 FIX: Tool response templates (loaded from tools/responses.yaml)
    #[serde(skip)]
    pub tool_responses: ToolResponsesConfig,
    /// P21 FIX: Compliance rules (loaded from compliance.yaml)
    #[serde(skip)]
    pub compliance: super::ComplianceConfig,
    /// P21 FIX: Adaptation/personalization config (loaded from adaptation.yaml)
    #[serde(skip)]
    pub adaptation: super::AdaptationConfig,
    /// P21 FIX: Extraction patterns for domain-agnostic slot extraction
    #[serde(skip)]
    pub extraction_patterns: super::ExtractionPatternsConfig,
    /// P22 FIX: Intent definitions (loaded from intents.yaml)
    #[serde(skip)]
    pub intents: IntentsConfig,
    /// P22 FIX: Full vocabulary with ASR boost, phonetic corrections (loaded from vocabulary.yaml)
    #[serde(skip)]
    pub vocabulary_full: FullVocabularyConfig,
    /// P22 FIX: Entity type definitions (loaded from entities.yaml)
    #[serde(skip)]
    pub entities: EntitiesConfig,
    /// P23 FIX: Signal definitions for lead scoring (loaded from signals.yaml)
    #[serde(skip)]
    pub signals: SignalsConfig,
    /// P24 FIX: Persona configurations for tone/style (loaded from personas.yaml)
    #[serde(skip)]
    pub personas: PersonasConfig,
    // P23 FIX: Removed raw_config field - was never accessed
    // Use typed config fields instead
}

fn default_version() -> String {
    "1.0.0".to_string()
}

impl Default for MasterDomainConfig {
    /// Creates an unconfigured default - should only be used for testing.
    /// In production, always load from YAML config files.
    fn default() -> Self {
        Self {
            domain_id: "unconfigured".to_string(),
            display_name: "Unconfigured Domain".to_string(),
            version: default_version(),
            brand: BrandConfig::default(),
            constants: DomainConstants::default(),
            competitors: HashMap::new(),
            products: HashMap::new(),
            high_value: HighValueConfig::default(),
            vocabulary: VocabularyConfig::default(),
            phonetic_corrections: PhoneticCorrectionsConfig::default(),
            relevance_terms: Vec::new(),
            query_expansion: QueryExpansionConfig::default(),
            domain_boost: DomainBoostConfig::default(),
            memory_compressor: MemoryCompressorConfig::default(),
            currency: CurrencyConfig::default(),
            rag_collection_name: None, // Will derive from domain_id
            slots: SlotsConfig::default(),
            stages: StagesConfig::default(),
            scoring: ScoringConfig::default(),
            tools: ToolsConfig::default(),
            prompts: PromptsConfig::default(),
            objections: ObjectionsConfig::default(),
            branches: BranchesConfig::default(),
            sms_templates: SmsTemplatesConfig::default(),
            competitors_config: CompetitorsConfig::default(),
            segments: SegmentsConfig::default(),
            goals: GoalsConfig::default(),
            features: FeaturesConfig::default(),
            documents: DocumentsConfig::default(),
            tool_responses: ToolResponsesConfig::default(),
            compliance: super::ComplianceConfig::default(),
            adaptation: super::AdaptationConfig::default(),
            extraction_patterns: super::ExtractionPatternsConfig::default(),
            intents: IntentsConfig::default(),
            vocabulary_full: FullVocabularyConfig::default(),
            entities: EntitiesConfig::default(),
            signals: SignalsConfig::default(),
            personas: PersonasConfig::default(),
            // P23 FIX: Removed raw_config - use typed config fields
        }
    }
}

impl MasterDomainConfig {
    /// Load configuration from a directory structure
    ///
    /// Expects:
    /// - config_dir/base/defaults.yaml (optional)
    /// - config_dir/domains/{domain_id}/domain.yaml
    pub fn load(domain_id: &str, config_dir: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let config_dir = config_dir.as_ref();

        // 1. Try to load base defaults
        let base_path = config_dir.join("base/defaults.yaml");
        let base_config: Option<JsonValue> = if base_path.exists() {
            let content = std::fs::read_to_string(&base_path)
                .map_err(|e| ConfigError::ParseError(format!("Failed to read base config: {}", e)))?;
            Some(serde_yaml::from_str(&content)
                .map_err(|e| ConfigError::ParseError(format!("Failed to parse base config: {}", e)))?)
        } else {
            tracing::debug!("No base config found at {:?}", base_path);
            None
        };

        // 2. Load domain-specific config
        let domain_path = config_dir.join(format!("domains/{}/domain.yaml", domain_id));
        if !domain_path.exists() {
            return Err(ConfigError::FileNotFound(domain_path.display().to_string()));
        }

        let domain_content = std::fs::read_to_string(&domain_path)
            .map_err(|e| ConfigError::ParseError(format!("Failed to read domain config: {}", e)))?;

        let domain_config: JsonValue = serde_yaml::from_str(&domain_content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse domain config: {}", e)))?;

        // 3. Merge configs (domain overrides base)
        let merged = if let Some(base) = base_config {
            merge_json(base, domain_config)
        } else {
            domain_config
        };

        // 4. Parse into typed config
        let mut config: MasterDomainConfig = serde_json::from_value(merged.clone())
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse merged config: {}", e)))?;

        // P23 FIX: Removed raw_config storage - use typed config fields instead

        // 5. Load slots configuration (optional)
        let slots_path = config_dir.join(format!("domains/{}/slots.yaml", domain_id));
        if slots_path.exists() {
            match SlotsConfig::load(&slots_path) {
                Ok(slots) => {
                    tracing::info!(
                        slots_count = slots.slots.len(),
                        goals_count = slots.goals.len(),
                        "Loaded slots configuration"
                    );
                    config.slots = slots;
                }
                Err(e) => {
                    tracing::warn!("Failed to load slots config: {}", e);
                }
            }
        } else {
            tracing::debug!("No slots config found at {:?}", slots_path);
        }

        // 6. Load stages configuration (optional)
        let stages_path = config_dir.join(format!("domains/{}/stages.yaml", domain_id));
        if stages_path.exists() {
            match StagesConfig::load(&stages_path) {
                Ok(stages) => {
                    tracing::info!(
                        stages_count = stages.stages.len(),
                        initial_stage = %stages.initial_stage,
                        "Loaded stages configuration"
                    );
                    config.stages = stages;
                }
                Err(e) => {
                    tracing::warn!("Failed to load stages config: {}", e);
                }
            }
        } else {
            tracing::debug!("No stages config found at {:?}", stages_path);
        }

        // 7. Load scoring configuration (optional)
        let scoring_path = config_dir.join(format!("domains/{}/scoring.yaml", domain_id));
        if scoring_path.exists() {
            match ScoringConfig::load(&scoring_path) {
                Ok(scoring) => {
                    tracing::info!(
                        high_value_threshold = %scoring.escalation.high_value_threshold,
                        "Loaded scoring configuration"
                    );
                    config.scoring = scoring;
                }
                Err(e) => {
                    tracing::warn!("Failed to load scoring config: {}", e);
                }
            }
        } else {
            tracing::debug!("No scoring config found at {:?}", scoring_path);
        }

        // 8. Load tools configuration (optional)
        let tools_path = config_dir.join(format!("domains/{}/tools/schemas.yaml", domain_id));
        if tools_path.exists() {
            match ToolsConfig::load(&tools_path) {
                Ok(tools) => {
                    tracing::info!(
                        tools_count = tools.tools.len(),
                        "Loaded tools configuration"
                    );
                    config.tools = tools;
                }
                Err(e) => {
                    tracing::warn!("Failed to load tools config: {}", e);
                }
            }
        } else {
            tracing::debug!("No tools config found at {:?}", tools_path);
        }

        // 8b. P16 FIX: Load intent-to-tool mappings (optional, merges into tools config)
        let mappings_path = config_dir.join(format!("domains/{}/intent_tool_mappings.yaml", domain_id));
        if mappings_path.exists() {
            match super::tools::IntentToolMappingsConfig::load(&mappings_path) {
                Ok(mappings) => {
                    // Expand aliases and merge into tools.intent_to_tool
                    let expanded = mappings.expand_aliases();
                    tracing::info!(
                        intent_mappings = expanded.len(),
                        slot_aliases = mappings.slot_aliases.len(),
                        tool_defaults = mappings.tool_defaults.len(),
                        argument_mappings = mappings.argument_mappings.len(),
                        "Loaded intent-to-tool mappings"
                    );
                    // Merge expanded mappings into tools config
                    for (intent, mapping) in expanded {
                        config.tools.intent_to_tool.insert(intent, mapping);
                    }
                    // P16 FIX: Merge slot_aliases, tool_defaults, and argument_mappings
                    for (alias, canonical) in mappings.slot_aliases {
                        config.tools.slot_aliases.insert(alias, canonical);
                    }
                    for (tool, defaults) in mappings.tool_defaults {
                        config.tools.tool_defaults.insert(tool, defaults);
                    }
                    for (tool, arg_mapping) in mappings.argument_mappings {
                        config.tools.argument_mappings.insert(tool, arg_mapping);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to load intent-to-tool mappings: {}", e);
                }
            }
        } else {
            tracing::debug!("No intent-to-tool mappings found at {:?}", mappings_path);
        }

        // 9. Load prompts configuration (optional)
        let prompts_path = config_dir.join(format!("domains/{}/prompts/system.yaml", domain_id));
        if prompts_path.exists() {
            match PromptsConfig::load(&prompts_path) {
                Ok(prompts) => {
                    tracing::info!(
                        has_system_prompt = !prompts.system_prompt.is_empty(),
                        "Loaded prompts configuration"
                    );
                    config.prompts = prompts;
                }
                Err(e) => {
                    tracing::warn!("Failed to load prompts config: {}", e);
                }
            }
        } else {
            tracing::debug!("No prompts config found at {:?}", prompts_path);
        }

        // 10. Load objections configuration (optional)
        let objections_path = config_dir.join(format!("domains/{}/objections.yaml", domain_id));
        if objections_path.exists() {
            match ObjectionsConfig::load(&objections_path) {
                Ok(objections) => {
                    tracing::info!(
                        objection_types = objections.objections.len(),
                        "Loaded objections configuration"
                    );
                    config.objections = objections;
                }
                Err(e) => {
                    tracing::warn!("Failed to load objections config: {}", e);
                }
            }
        } else {
            tracing::debug!("No objections config found at {:?}", objections_path);
        }

        // 11. Load branches configuration (optional)
        let branches_path = config_dir.join(format!("domains/{}/tools/branches.yaml", domain_id));
        if branches_path.exists() {
            match BranchesConfig::load(&branches_path) {
                Ok(branches) => {
                    tracing::info!(
                        branches_count = branches.branches.len(),
                        "Loaded branches configuration"
                    );
                    config.branches = branches;
                }
                Err(e) => {
                    tracing::warn!("Failed to load branches config: {}", e);
                }
            }
        } else {
            tracing::debug!("No branches config found at {:?}", branches_path);
        }

        // 12. Load SMS templates configuration (optional)
        let sms_path = config_dir.join(format!("domains/{}/tools/sms_templates.yaml", domain_id));
        if sms_path.exists() {
            match SmsTemplatesConfig::load(&sms_path) {
                Ok(sms) => {
                    tracing::info!(
                        template_types = sms.templates.len(),
                        "Loaded SMS templates configuration"
                    );
                    config.sms_templates = sms;
                }
                Err(e) => {
                    tracing::warn!("Failed to load SMS templates config: {}", e);
                }
            }
        } else {
            tracing::debug!("No SMS templates config found at {:?}", sms_path);
        }

        // 13. Load competitors configuration (optional)
        let competitors_path = config_dir.join(format!("domains/{}/competitors.yaml", domain_id));
        if competitors_path.exists() {
            match CompetitorsConfig::load(&competitors_path) {
                Ok(competitors) => {
                    tracing::info!(
                        competitors_count = competitors.competitors.len(),
                        "Loaded competitors configuration"
                    );
                    config.competitors_config = competitors;
                }
                Err(e) => {
                    tracing::warn!("Failed to load competitors config: {}", e);
                }
            }
        } else {
            tracing::debug!("No competitors config found at {:?}", competitors_path);
        }

        // 14. Load segments configuration (optional)
        let segments_path = config_dir.join(format!("domains/{}/segments.yaml", domain_id));
        if segments_path.exists() {
            match SegmentsConfig::load(&segments_path) {
                Ok(segments) => {
                    tracing::info!(
                        segments_count = segments.segments.len(),
                        "Loaded segments configuration"
                    );
                    config.segments = segments;
                }
                Err(e) => {
                    tracing::warn!("Failed to load segments config: {}", e);
                }
            }
        } else {
            tracing::debug!("No segments config found at {:?}", segments_path);
        }

        // 15. Load goals configuration (optional)
        let goals_path = config_dir.join(format!("domains/{}/goals.yaml", domain_id));
        if goals_path.exists() {
            match GoalsConfig::load(&goals_path) {
                Ok(goals) => {
                    tracing::info!(
                        goals_count = goals.goals.len(),
                        intent_mappings_count = goals.intent_mappings.len(),
                        "Loaded goals configuration"
                    );
                    config.goals = goals;
                }
                Err(e) => {
                    tracing::warn!("Failed to load goals config: {}", e);
                }
            }
        } else {
            tracing::debug!("No goals config found at {:?}", goals_path);
        }

        // 16. Load features configuration (optional)
        let features_path = config_dir.join(format!("domains/{}/features.yaml", domain_id));
        if features_path.exists() {
            let content = std::fs::read_to_string(&features_path)
                .map_err(|e| ConfigError::ParseError(format!("Failed to read features config: {}", e)))?;
            match serde_yaml::from_str::<FeaturesConfig>(&content) {
                Ok(features) => {
                    tracing::info!(
                        features_count = features.features.len(),
                        segments_with_features = features.segment_features.len(),
                        "Loaded features configuration"
                    );
                    config.features = features;
                }
                Err(e) => {
                    tracing::warn!("Failed to parse features config: {}", e);
                }
            }
        } else {
            tracing::debug!("No features config found at {:?}", features_path);
        }

        // 17. Load documents configuration (optional)
        let documents_path = config_dir.join(format!("domains/{}/tools/documents.yaml", domain_id));
        if documents_path.exists() {
            match DocumentsConfig::load(&documents_path) {
                Ok(documents) => {
                    tracing::info!(
                        service_types = documents.service_types.len(),
                        customer_types = documents.customer_types.len(),
                        mandatory_docs = documents.mandatory_documents.len(),
                        "Loaded documents configuration"
                    );
                    config.documents = documents;
                }
                Err(e) => {
                    tracing::warn!("Failed to load documents config: {}", e);
                }
            }
        } else {
            tracing::debug!("No documents config found at {:?}", documents_path);
        }

        // 18. P16 FIX: Load tool response templates (optional)
        let responses_path = config_dir.join(format!("domains/{}/tools/responses.yaml", domain_id));
        if responses_path.exists() {
            match ToolResponsesConfig::load(&responses_path) {
                Ok(responses) => {
                    tracing::info!(
                        tools_with_templates = responses.templates.len(),
                        "Loaded tool response templates"
                    );
                    config.tool_responses = responses;
                }
                Err(e) => {
                    tracing::warn!("Failed to load tool response templates: {}", e);
                }
            }
        } else {
            tracing::debug!("No tool response templates found at {:?}", responses_path);
        }

        // 19. P21 FIX: Load compliance rules (optional)
        let compliance_path = config_dir.join(format!("domains/{}/compliance.yaml", domain_id));
        if compliance_path.exists() {
            match super::ComplianceConfig::load(&compliance_path) {
                Ok(compliance) => {
                    tracing::info!(
                        version = %compliance.version,
                        forbidden_phrases = compliance.forbidden_phrases.len(),
                        claims_rules = compliance.claims_requiring_disclaimer.len(),
                        "Loaded compliance configuration"
                    );
                    config.compliance = compliance;
                }
                Err(e) => {
                    tracing::warn!("Failed to load compliance config: {}", e);
                }
            }
        } else {
            tracing::debug!("No compliance config found at {:?}", compliance_path);
        }

        // 20. P21 FIX: Load adaptation/personalization config (optional)
        let adaptation_path = config_dir.join(format!("domains/{}/adaptation.yaml", domain_id));
        if adaptation_path.exists() {
            match super::AdaptationConfig::load(&adaptation_path) {
                Ok(adaptation) => {
                    tracing::info!(
                        schema_version = %adaptation.schema_version,
                        variables = adaptation.variables.len(),
                        segments = adaptation.segment_adaptations.len(),
                        features = adaptation.enabled_features.len(),
                        "Loaded adaptation configuration"
                    );
                    config.adaptation = adaptation;
                }
                Err(e) => {
                    tracing::warn!("Failed to load adaptation config: {}", e);
                }
            }
        } else {
            tracing::debug!("No adaptation config found at {:?}", adaptation_path);
        }

        // 21. P21 FIX: Load extraction patterns config (optional)
        let extraction_path = config_dir.join(format!("domains/{}/extraction_patterns.yaml", domain_id));
        if extraction_path.exists() {
            match super::ExtractionPatternsConfig::load(&extraction_path) {
                Ok(patterns) => {
                    tracing::info!(
                        quality_tiers = patterns.asset_quality.tiers.len(),
                        cities = patterns.locations.cities.len(),
                        purposes = patterns.purposes.categories.len(),
                        "Loaded extraction patterns configuration"
                    );
                    config.extraction_patterns = patterns;
                }
                Err(e) => {
                    tracing::warn!("Failed to load extraction patterns config: {}", e);
                }
            }
        } else {
            tracing::debug!("No extraction patterns config found at {:?}", extraction_path);
        }

        // 22. P22 FIX: Load intents configuration (optional)
        let intents_path = config_dir.join(format!("domains/{}/intents.yaml", domain_id));
        if intents_path.exists() {
            match IntentsConfig::load(&intents_path) {
                Ok(intents) => {
                    tracing::info!(
                        intents_count = intents.intents.len(),
                        default_intent = %intents.default_intent,
                        "Loaded intents configuration"
                    );
                    config.intents = intents;
                }
                Err(e) => {
                    tracing::warn!("Failed to load intents config: {}", e);
                }
            }
        } else {
            tracing::debug!("No intents config found at {:?}", intents_path);
        }

        // 23. P22 FIX: Load full vocabulary configuration (optional)
        let vocabulary_path = config_dir.join(format!("domains/{}/vocabulary.yaml", domain_id));
        if vocabulary_path.exists() {
            match FullVocabularyConfig::load(&vocabulary_path) {
                Ok(vocab) => {
                    tracing::info!(
                        domain_terms = vocab.domain_terms.len(),
                        abbreviations = vocab.abbreviations.len(),
                        phonetic_corrections = vocab.phonetic_corrections.len(),
                        hindi_numbers = vocab.hindi_numbers.len(),
                        "Loaded full vocabulary configuration"
                    );
                    config.vocabulary_full = vocab;
                }
                Err(e) => {
                    tracing::warn!("Failed to load vocabulary config: {}", e);
                }
            }
        } else {
            tracing::debug!("No vocabulary config found at {:?}", vocabulary_path);
        }

        // 24. P22 FIX: Load entities configuration (optional)
        let entities_path = config_dir.join(format!("domains/{}/entities.yaml", domain_id));
        if entities_path.exists() {
            match EntitiesConfig::load(&entities_path) {
                Ok(entities) => {
                    tracing::info!(
                        entity_types = entities.entity_types.len(),
                        categories = entities.categories.len(),
                        extraction_priority = entities.extraction_priority.len(),
                        "Loaded entities configuration"
                    );
                    config.entities = entities;
                }
                Err(e) => {
                    tracing::warn!("Failed to load entities config: {}", e);
                }
            }
        } else {
            tracing::debug!("No entities config found at {:?}", entities_path);
        }

        // 25. P23 FIX: Load signals configuration for lead scoring (optional)
        let signals_path = config_dir.join(format!("domains/{}/signals.yaml", domain_id));
        if signals_path.exists() {
            match SignalsConfig::load(&signals_path) {
                Ok(signals) => {
                    tracing::info!(
                        signal_count = signals.signals.len(),
                        categories = signals.categories.len(),
                        escalation_triggers = signals.escalation_triggers.len(),
                        "Loaded signals configuration"
                    );
                    config.signals = signals;
                }
                Err(e) => {
                    tracing::warn!("Failed to load signals config: {}", e);
                }
            }
        } else {
            tracing::debug!("No signals config found at {:?}", signals_path);
        }

        // 26. P24 FIX: Load personas configuration for tone/style (optional)
        let personas_path = config_dir.join(format!("domains/{}/personas.yaml", domain_id));
        if personas_path.exists() {
            match PersonasConfig::load(&personas_path) {
                Ok(personas) => {
                    tracing::info!(
                        tones = personas.tones.len(),
                        warmth_thresholds = personas.warmth_thresholds.len(),
                        complexity_levels = personas.complexity_levels.len(),
                        adaptation_rules = personas.adaptation_rules.len(),
                        "Loaded personas configuration"
                    );
                    config.personas = personas;
                }
                Err(e) => {
                    tracing::warn!("Failed to load personas config: {}", e);
                }
            }
        } else {
            tracing::debug!("No personas config found at {:?}", personas_path);
        }

        // 27. P16 FIX: Apply variable substitution to all text configs
        // This allows YAML files to use {{variable_name}} placeholders
        // that are replaced with values from adaptation.yaml variables
        config.substitute_all_variables();

        tracing::info!(
            domain_id = %config.domain_id,
            display_name = %config.display_name,
            "Loaded domain configuration"
        );

        Ok(config)
    }

    /// Load from environment variable DOMAIN_ID (required, no default)
    ///
    /// P16 FIX: DOMAIN_ID is now REQUIRED - this is a domain-agnostic system.
    /// Returns an error if DOMAIN_ID is not set.
    pub fn load_from_env(config_dir: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let domain_id = std::env::var("DOMAIN_ID")
            .map_err(|_| ConfigError::MissingField(
                "DOMAIN_ID environment variable is not set. \
                 This is a domain-agnostic system - you MUST specify which domain to use. \
                 Set DOMAIN_ID to the name of the domain config directory \
                 (e.g., DOMAIN_ID=my_domain for config/domains/my_domain/).".to_string()
            ))?;

        if domain_id.is_empty() {
            return Err(ConfigError::MissingField(
                "DOMAIN_ID environment variable is empty. Please specify a domain ID.".to_string()
            ));
        }

        Self::load(&domain_id, config_dir)
    }

    // P23 FIX: Removed get_constant() - was never called
    // Use typed config fields (e.g., self.constants.interest_rates) instead of raw JSON access

    /// Get the best interest rate for a given loan amount
    pub fn get_rate_for_amount(&self, amount: f64) -> f64 {
        for tier in &self.constants.interest_rates.tiers {
            if let Some(max) = tier.max_amount {
                if amount <= max {
                    return tier.rate;
                }
            } else {
                // No max = this is the rate for amounts above all thresholds
                return tier.rate;
            }
        }
        // Fallback to base rate
        self.constants.interest_rates.base_rate
    }

    /// Check if this is a high-value customer
    pub fn is_high_value(&self, amount: Option<f64>, weight_grams: Option<f64>) -> bool {
        if let Some(amt) = amount {
            if amt >= self.high_value.amount_threshold {
                return true;
            }
        }
        if let Some(wt) = weight_grams {
            if wt >= self.high_value.weight_threshold_grams {
                return true;
            }
        }
        false
    }

    /// Get competitor by name or alias
    pub fn get_competitor(&self, name: &str) -> Option<&CompetitorEntry> {
        let name_lower = name.to_lowercase();

        // Direct match
        if let Some(comp) = self.competitors.get(&name_lower) {
            return Some(comp);
        }

        // Search aliases
        for (_, comp) in &self.competitors {
            if comp.aliases.iter().any(|a| a.to_lowercase() == name_lower) {
                return Some(comp);
            }
        }

        None
    }

    /// P16 FIX: Apply variable substitution to all text configs
    ///
    /// Replaces {{variable_name}} placeholders with values from adaptation.yaml
    /// This enables domain-agnostic config files that use variables for
    /// company names, rates, and other domain-specific content.
    pub fn substitute_all_variables(&mut self) {
        // Skip if no variables defined
        if self.adaptation.variables.is_empty() {
            return;
        }

        // Helper closure to substitute in a string
        let substitute = |s: &str| -> String {
            let mut result = s.to_string();
            for (key, value) in &self.adaptation.variables {
                result = result.replace(&format!("{{{{{}}}}}", key), value);
            }
            result
        };

        // Substitute in stages config
        for stage in self.stages.stages.values_mut() {
            stage.guidance = substitute(&stage.guidance);
            for question in &mut stage.suggested_questions {
                *question = substitute(question);
            }
        }

        // Substitute in segments config
        for segment in self.segments.segments.values_mut() {
            for props in segment.value_props.values_mut() {
                for prop in props.iter_mut() {
                    *prop = substitute(prop);
                }
            }
        }

        // Substitute in competitors config
        for point in &mut self.competitors_config.comparison_points {
            point.our_advantage = substitute(&point.our_advantage);
        }
        for feature in &mut self.competitors_config.our_features {
            *feature = substitute(feature);
        }

        // P23 FIX: Substitute in objections config
        for objection in self.objections.objections.values_mut() {
            for responses in objection.responses.values_mut() {
                responses.acknowledge = substitute(&responses.acknowledge);
                responses.reframe = substitute(&responses.reframe);
                responses.evidence = substitute(&responses.evidence);
                responses.call_to_action = substitute(&responses.call_to_action);
            }
        }
        // Substitute in default objection responses (if present)
        if let Some(ref mut default_obj) = self.objections.default_objection {
            for responses in default_obj.responses.values_mut() {
                responses.acknowledge = substitute(&responses.acknowledge);
                responses.reframe = substitute(&responses.reframe);
                responses.evidence = substitute(&responses.evidence);
                responses.call_to_action = substitute(&responses.call_to_action);
            }
        }

        tracing::debug!(
            variables_count = self.adaptation.variables.len(),
            "Applied variable substitution to config"
        );
    }
}

/// Deep merge two JSON values (right overrides left)
fn merge_json(left: JsonValue, right: JsonValue) -> JsonValue {
    match (left, right) {
        (JsonValue::Object(mut left_map), JsonValue::Object(right_map)) => {
            for (key, right_val) in right_map {
                let merged_val = if let Some(left_val) = left_map.remove(&key) {
                    merge_json(left_val, right_val)
                } else {
                    right_val
                };
                left_map.insert(key, merged_val);
            }
            JsonValue::Object(left_map)
        }
        (_, right) => right, // Right value wins for non-objects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MasterDomainConfig::default();
        // Default is now "unconfigured" to enforce explicit domain configuration
        assert_eq!(config.domain_id, "unconfigured");
        assert_eq!(config.display_name, "Unconfigured Domain");
    }

    #[test]
    fn test_merge_json() {
        let base = serde_json::json!({
            "a": 1,
            "b": { "c": 2, "d": 3 }
        });
        let overlay = serde_json::json!({
            "a": 10,
            "b": { "c": 20 },
            "e": 5
        });
        let merged = merge_json(base, overlay);

        assert_eq!(merged["a"], 10);
        assert_eq!(merged["b"]["c"], 20);
        assert_eq!(merged["b"]["d"], 3);
        assert_eq!(merged["e"], 5);
    }
}
