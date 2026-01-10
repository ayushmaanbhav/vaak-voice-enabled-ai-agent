//! Extraction Patterns Configuration
//!
//! P21 FIX: Config-driven text patterns for domain-agnostic slot extraction.
//!
//! This module defines configuration structures for extracting entities from text.
//! Each domain can define its own patterns (asset quality, cities, purposes, etc.),
//! enabling truly domain-agnostic slot extraction.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root extraction patterns configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExtractionPatternsConfig {
    /// Asset quality/purity patterns (e.g., gold karat, car condition)
    #[serde(default)]
    pub asset_quality: AssetQualityConfig,

    /// Location/city patterns
    #[serde(default)]
    pub locations: LocationsConfig,

    /// Loan/product purpose patterns
    #[serde(default)]
    pub purposes: PurposesConfig,

    /// P21 FIX: Repayment type patterns (EMI, bullet, overdraft, etc.)
    #[serde(default)]
    pub repayment_types: RepaymentTypesConfig,

    /// Unit conversions (weight, currency)
    #[serde(default)]
    pub unit_conversions: UnitConversionsConfig,

    /// Validation thresholds
    #[serde(default)]
    pub validation: ValidationConfig,

    /// Confidence boost keywords
    #[serde(default)]
    pub confidence_boosters: ConfidenceBoostersConfig,

    /// Name exclusion list
    #[serde(default)]
    pub name_exclusions: Vec<String>,
}

impl ExtractionPatternsConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ExtractionPatternsError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ExtractionPatternsError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| ExtractionPatternsError::ParseError(e.to_string()))
    }

    /// Get all city names (lowercase) for pattern matching
    pub fn city_names(&self) -> Vec<String> {
        self.locations
            .cities
            .iter()
            .map(|c| c.name.to_lowercase())
            .collect()
    }

    /// Get city pattern for a language
    pub fn city_pattern(&self, lang: &str) -> String {
        let patterns: Vec<String> = self
            .locations
            .cities
            .iter()
            .filter_map(|c| {
                if lang == "hi" {
                    c.pattern_hi.clone()
                } else {
                    Some(c.pattern_en.clone())
                }
            })
            .collect();
        patterns.join("|")
    }

    /// Get asset quality tier by value
    pub fn get_quality_tier(&self, value: &str) -> Option<&AssetQualityTier> {
        self.asset_quality.tiers.iter().find(|t| t.value == value)
    }

    /// Get default quality tier
    pub fn default_quality_tier(&self) -> Option<&AssetQualityTier> {
        self.asset_quality.tiers.iter().find(|t| t.default.unwrap_or(false))
    }

    /// Get purpose category by id
    pub fn get_purpose(&self, id: &str) -> Option<&PurposeCategory> {
        self.purposes.categories.iter().find(|p| p.id == id)
    }

    /// Get tola to grams conversion factor
    pub fn tola_to_grams(&self) -> f64 {
        self.unit_conversions.weight.tola_to_grams
    }

    /// Get currency multiplier by name
    pub fn currency_multiplier(&self, name: &str) -> Option<f64> {
        let lower = name.to_lowercase();
        self.unit_conversions
            .currency
            .get(&lower)
            .copied()
            .map(|v| v as f64)
            .or_else(|| {
                self.unit_conversions
                    .currency_hindi
                    .get(name)
                    .copied()
                    .map(|v| v as f64)
            })
    }

    /// Check if a word is in the name exclusion list
    pub fn is_name_excluded(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        self.name_exclusions.iter().any(|e| e.to_lowercase() == lower)
    }

    /// Get confidence boost keywords for a category and language
    pub fn confidence_keywords(&self, category: &str, lang: &str) -> Vec<&str> {
        let keywords = match category {
            "amount" => &self.confidence_boosters.amount,
            "weight" => &self.confidence_boosters.weight,
            "rate" => &self.confidence_boosters.rate,
            _ => return vec![],
        };

        if lang == "hi" {
            keywords.hi.iter().map(|s| s.as_str()).collect()
        } else {
            keywords.en.iter().map(|s| s.as_str()).collect()
        }
    }

    /// P21 FIX: Get repayment type patterns for a language
    /// Returns patterns compiled from config for repayment type detection
    pub fn repayment_patterns(&self, lang: &str) -> Vec<(&str, &str, f32)> {
        self.repayment_types.get_patterns(lang)
    }

    /// P21 FIX: Get repayment type configuration
    pub fn repayment_types_config(&self) -> &RepaymentTypesConfig {
        &self.repayment_types
    }

    /// P1.1 FIX: Build compiled quality tier patterns for slot extraction
    ///
    /// Combines patterns for all languages into case-insensitive regex patterns.
    /// Returns a vector of compiled patterns that can be used by SlotExtractor.
    ///
    /// # Example
    /// ```ignore
    /// let config = ExtractionPatternsConfig::load("path/to/extraction_patterns.yaml")?;
    /// let patterns = config.compile_quality_patterns();
    /// let extractor = SlotExtractor::with_quality_tiers(patterns);
    /// ```
    pub fn compile_quality_patterns(&self) -> Vec<CompiledQualityTier> {
        self.asset_quality
            .tiers
            .iter()
            .filter_map(|tier| {
                // Combine all language patterns into one case-insensitive regex
                let all_patterns: Vec<&str> = tier
                    .patterns
                    .values()
                    .flat_map(|v| v.iter().map(|s| s.as_str()))
                    .collect();

                if all_patterns.is_empty() {
                    return None;
                }

                // Build combined pattern with case-insensitive flag
                let combined = format!("(?i)({})", all_patterns.join("|"));
                match Regex::new(&combined) {
                    Ok(regex) => Some(CompiledQualityTier {
                        id: tier.id.clone(),
                        value: tier.value.clone(),
                        display_name: tier.display_name.clone(),
                        pattern: regex,
                        confidence: tier.confidence,
                        is_default: tier.default.unwrap_or(false),
                    }),
                    Err(e) => {
                        tracing::warn!(
                            "Failed to compile quality tier pattern for {}: {}",
                            tier.id,
                            e
                        );
                        None
                    }
                }
            })
            .collect()
    }

    /// P1.1 FIX: Get quality validation range
    pub fn quality_validation_range(&self) -> (u32, u32) {
        (self.asset_quality.validation.min, self.asset_quality.validation.max)
    }

    /// P2.1 FIX: Compile city patterns for domain-agnostic location extraction
    ///
    /// Returns compiled regex patterns for each city, combining English and Hindi patterns.
    pub fn compile_city_patterns(&self) -> Vec<CompiledCityPattern> {
        self.locations
            .cities
            .iter()
            .filter_map(|city| {
                // Build combined pattern from all languages
                let mut patterns = vec![city.pattern_en.clone()];
                if let Some(ref hi) = city.pattern_hi {
                    if !hi.is_empty() {
                        patterns.push(hi.clone());
                    }
                }
                // Add aliases as patterns
                for alias in &city.aliases {
                    patterns.push(format!("(?i)\\b{}\\b", regex::escape(alias)));
                }

                if patterns.is_empty() || patterns.iter().all(|p| p.is_empty()) {
                    return None;
                }

                let combined = format!("(?i)({})", patterns.join("|"));
                match Regex::new(&combined) {
                    Ok(regex) => Some(CompiledCityPattern {
                        name: city.name.clone(),
                        aliases: city.aliases.clone(),
                        pattern: regex,
                        confidence: 0.85, // Default confidence
                    }),
                    Err(e) => {
                        tracing::warn!("Failed to compile city pattern for {}: {}", city.name, e);
                        None
                    }
                }
            })
            .collect()
    }

    /// P2.1 FIX: Compile purpose patterns for domain-agnostic purpose extraction
    ///
    /// Returns compiled keyword patterns for each purpose category.
    pub fn compile_purpose_patterns(&self) -> Vec<CompiledPurposePattern> {
        self.purposes
            .categories
            .iter()
            .filter_map(|purpose| {
                // Combine all language keywords into one pattern
                let mut all_keywords: Vec<String> = Vec::new();
                all_keywords.extend(purpose.keywords.en.iter().cloned());
                all_keywords.extend(purpose.keywords.hi.iter().cloned());

                if all_keywords.is_empty() {
                    return None;
                }

                // Escape and join keywords for regex
                let escaped: Vec<String> = all_keywords
                    .iter()
                    .map(|k| regex::escape(k))
                    .collect();
                let combined = format!("(?i)({})", escaped.join("|"));

                match Regex::new(&combined) {
                    Ok(regex) => Some(CompiledPurposePattern {
                        id: purpose.id.clone(),
                        display_name: purpose.display_name.clone(),
                        pattern: regex,
                        confidence: 0.80, // Default confidence
                    }),
                    Err(e) => {
                        tracing::warn!("Failed to compile purpose pattern for {}: {}", purpose.id, e);
                        None
                    }
                }
            })
            .collect()
    }
}

/// P1.1 FIX: Compiled quality tier pattern for slot extraction
#[derive(Debug, Clone)]
pub struct CompiledQualityTier {
    /// Tier ID (e.g., "tier_1", "tier_2")
    pub id: String,
    /// Value to return when matched (e.g., "24", "22")
    pub value: String,
    /// Display name (e.g., "24K Pure Gold")
    pub display_name: String,
    /// Compiled regex pattern
    pub pattern: Regex,
    /// Confidence score for matches
    pub confidence: f32,
    /// Whether this is the default tier
    pub is_default: bool,
}

/// P2.1 FIX: Compiled city pattern for slot extraction
#[derive(Debug, Clone)]
pub struct CompiledCityPattern {
    /// Canonical city name
    pub name: String,
    /// Alternative names/spellings
    pub aliases: Vec<String>,
    /// Compiled regex pattern
    pub pattern: Regex,
    /// Confidence score for matches
    pub confidence: f32,
}

/// P2.1 FIX: Compiled purpose pattern for slot extraction
#[derive(Debug, Clone)]
pub struct CompiledPurposePattern {
    /// Purpose ID (e.g., "business", "medical")
    pub id: String,
    /// Display name
    pub display_name: String,
    /// Compiled regex pattern
    pub pattern: Regex,
    /// Confidence score for matches
    pub confidence: f32,
}

// =============================================================================
// Asset Quality Configuration
// =============================================================================

/// Asset quality/purity configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetQualityConfig {
    /// Quality tiers (e.g., 24K, 22K for gold; excellent, good for cars)
    #[serde(default)]
    pub tiers: Vec<AssetQualityTier>,

    /// Validation range
    #[serde(default)]
    pub validation: QualityValidation,
}

/// A single asset quality tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetQualityTier {
    /// Unique identifier (e.g., "tier_1", "excellent")
    pub id: String,

    /// Display name (e.g., "24K Pure Gold", "Excellent Condition")
    pub display_name: String,

    /// Regex patterns by language
    #[serde(default)]
    pub patterns: HashMap<String, Vec<String>>,

    /// Value to store when matched (e.g., "24", "excellent")
    pub value: String,

    /// Confidence score for matches
    #[serde(default = "default_confidence")]
    pub confidence: f32,

    /// Whether this is the default tier
    #[serde(default)]
    pub default: Option<bool>,
}

fn default_confidence() -> f32 {
    0.85
}

/// Quality validation range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityValidation {
    /// Minimum value (for numeric quality like karat)
    #[serde(default = "default_quality_min")]
    pub min: u32,

    /// Maximum value
    #[serde(default = "default_quality_max")]
    pub max: u32,

    /// Unit of measurement
    #[serde(default)]
    pub unit: String,
}

fn default_quality_min() -> u32 {
    10
}

fn default_quality_max() -> u32 {
    24
}

impl Default for QualityValidation {
    fn default() -> Self {
        Self {
            min: default_quality_min(),
            max: default_quality_max(),
            unit: "karat".to_string(),
        }
    }
}

// =============================================================================
// Location Configuration
// =============================================================================

/// Locations/cities configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocationsConfig {
    /// Supported cities
    #[serde(default)]
    pub cities: Vec<CityEntry>,

    /// Context keywords that indicate location
    #[serde(default)]
    pub context_keywords: LanguageKeywords,
}

/// A city entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityEntry {
    /// City name
    pub name: String,

    /// Alternative names/spellings
    #[serde(default)]
    pub aliases: Vec<String>,

    /// English regex pattern
    #[serde(default)]
    pub pattern_en: String,

    /// Hindi regex pattern
    #[serde(default)]
    pub pattern_hi: Option<String>,
}

/// Keywords by language
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LanguageKeywords {
    #[serde(default)]
    pub en: Vec<String>,
    #[serde(default)]
    pub hi: Vec<String>,
}

// =============================================================================
// Purpose Configuration
// =============================================================================

/// Loan/product purposes configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PurposesConfig {
    /// Purpose categories
    #[serde(default)]
    pub categories: Vec<PurposeCategory>,
}

/// A purpose category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurposeCategory {
    /// Unique identifier (e.g., "business", "medical")
    pub id: String,

    /// Display name
    pub display_name: String,

    /// Keywords by language
    #[serde(default)]
    pub keywords: LanguageKeywords,
}

// =============================================================================
// Repayment Types Configuration (P21 FIX)
// =============================================================================

/// Repayment types configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepaymentTypesConfig {
    /// Repayment type categories
    #[serde(default)]
    pub categories: Vec<RepaymentTypeCategory>,

    /// Default repayment type if not specified
    #[serde(default = "default_repayment_type")]
    pub default: String,
}

fn default_repayment_type() -> String {
    "emi".to_string()
}

/// A repayment type category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepaymentTypeCategory {
    /// Unique identifier (e.g., "emi", "bullet", "overdraft")
    pub id: String,

    /// Display name (e.g., "EMI", "Bullet Payment")
    pub display_name: String,

    /// Description of this repayment type
    #[serde(default)]
    pub description: String,

    /// Regex patterns by language
    #[serde(default)]
    pub patterns: HashMap<String, Vec<String>>,

    /// Confidence score for matches
    #[serde(default = "default_repayment_confidence")]
    pub confidence: f32,
}

fn default_repayment_confidence() -> f32 {
    0.8
}

impl RepaymentTypesConfig {
    /// Get all repayment patterns for a language
    /// Returns a vector of (pattern, type_id, confidence) tuples
    pub fn get_patterns(&self, lang: &str) -> Vec<(&str, &str, f32)> {
        self.categories
            .iter()
            .flat_map(|cat| {
                cat.patterns
                    .get(lang)
                    .into_iter()
                    .flatten()
                    .map(move |pattern| (pattern.as_str(), cat.id.as_str(), cat.confidence))
            })
            .collect()
    }

    /// Get repayment type by ID
    pub fn get_type(&self, id: &str) -> Option<&RepaymentTypeCategory> {
        self.categories.iter().find(|c| c.id == id)
    }

    /// Get display name for a repayment type
    pub fn get_display_name(&self, id: &str) -> Option<&str> {
        self.get_type(id).map(|t| t.display_name.as_str())
    }
}

// =============================================================================
// Unit Conversions Configuration
// =============================================================================

/// Unit conversions configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitConversionsConfig {
    /// Weight conversions
    #[serde(default)]
    pub weight: WeightConversions,

    /// Currency multipliers
    #[serde(default)]
    pub currency: HashMap<String, u64>,

    /// Hindi currency terms
    #[serde(default)]
    pub currency_hindi: HashMap<String, u64>,
}

impl Default for UnitConversionsConfig {
    fn default() -> Self {
        let mut currency = HashMap::new();
        currency.insert("crore".to_string(), 10_000_000);
        currency.insert("cr".to_string(), 10_000_000);
        currency.insert("lakh".to_string(), 100_000);
        currency.insert("lac".to_string(), 100_000);
        currency.insert("thousand".to_string(), 1_000);
        currency.insert("k".to_string(), 1_000);

        Self {
            weight: WeightConversions::default(),
            currency,
            currency_hindi: HashMap::new(),
        }
    }
}

/// Weight unit conversions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightConversions {
    /// Tola to grams (default: 11.66)
    #[serde(default = "default_tola")]
    pub tola_to_grams: f64,

    /// Gram to milligrams
    #[serde(default = "default_gram_to_mg")]
    pub gram_to_mg: f64,

    /// Kilogram to grams
    #[serde(default = "default_kg_to_grams")]
    pub kg_to_grams: f64,

    /// Troy ounce to grams
    #[serde(default = "default_oz_to_grams")]
    pub oz_to_grams: f64,
}

fn default_tola() -> f64 {
    11.66
}

fn default_gram_to_mg() -> f64 {
    1000.0
}

fn default_kg_to_grams() -> f64 {
    1000.0
}

fn default_oz_to_grams() -> f64 {
    31.1
}

impl Default for WeightConversions {
    fn default() -> Self {
        Self {
            tola_to_grams: default_tola(),
            gram_to_mg: default_gram_to_mg(),
            kg_to_grams: default_kg_to_grams(),
            oz_to_grams: default_oz_to_grams(),
        }
    }
}

// =============================================================================
// Validation Configuration
// =============================================================================

/// Validation thresholds configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationConfig {
    /// Amount validation
    #[serde(default)]
    pub amount: AmountValidation,

    /// Interest rate validation
    #[serde(default)]
    pub rate: RateValidation,

    /// Asset weight validation
    #[serde(default)]
    pub asset_weight: WeightValidation,

    /// Tenure validation
    #[serde(default)]
    pub tenure: TenureValidation,
}

/// Amount validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmountValidation {
    #[serde(default = "default_amount_min")]
    pub min: f64,
    #[serde(default = "default_amount_max")]
    pub max: f64,
    #[serde(default)]
    pub currency: String,
}

fn default_amount_min() -> f64 {
    10_000.0
}

fn default_amount_max() -> f64 {
    100_000_000.0
}

impl Default for AmountValidation {
    fn default() -> Self {
        Self {
            min: default_amount_min(),
            max: default_amount_max(),
            currency: "INR".to_string(),
        }
    }
}

/// Rate validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateValidation {
    #[serde(default = "default_rate_min")]
    pub min: f64,
    #[serde(default = "default_rate_max")]
    pub max: f64,
    #[serde(default)]
    pub unit: String,
}

fn default_rate_min() -> f64 {
    5.0
}

fn default_rate_max() -> f64 {
    30.0
}

impl Default for RateValidation {
    fn default() -> Self {
        Self {
            min: default_rate_min(),
            max: default_rate_max(),
            unit: "percent_per_annum".to_string(),
        }
    }
}

/// Weight validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightValidation {
    #[serde(default = "default_weight_min")]
    pub min: f64,
    #[serde(default = "default_weight_max")]
    pub max: f64,
    #[serde(default)]
    pub unit: String,
}

fn default_weight_min() -> f64 {
    1.0
}

fn default_weight_max() -> f64 {
    100_000.0
}

impl Default for WeightValidation {
    fn default() -> Self {
        Self {
            min: default_weight_min(),
            max: default_weight_max(),
            unit: "grams".to_string(),
        }
    }
}

/// Tenure validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenureValidation {
    #[serde(default = "default_tenure_min")]
    pub min_months: u32,
    #[serde(default = "default_tenure_max")]
    pub max_months: u32,
    #[serde(default = "default_tenure_default")]
    pub default_months: u32,
}

fn default_tenure_min() -> u32 {
    1
}

fn default_tenure_max() -> u32 {
    36
}

fn default_tenure_default() -> u32 {
    12
}

impl Default for TenureValidation {
    fn default() -> Self {
        Self {
            min_months: default_tenure_min(),
            max_months: default_tenure_max(),
            default_months: default_tenure_default(),
        }
    }
}

// =============================================================================
// Confidence Boosters Configuration
// =============================================================================

/// Confidence boost keywords configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfidenceBoostersConfig {
    /// Amount-related keywords
    #[serde(default)]
    pub amount: LanguageKeywords,

    /// Weight-related keywords
    #[serde(default)]
    pub weight: LanguageKeywords,

    /// Rate-related keywords
    #[serde(default)]
    pub rate: LanguageKeywords,
}

// =============================================================================
// Error Types
// =============================================================================

/// Errors when loading extraction patterns configuration
#[derive(Debug)]
pub enum ExtractionPatternsError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for ExtractionPatternsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Extraction patterns config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => {
                write!(f, "Failed to parse extraction patterns config: {}", err)
            }
        }
    }
}

impl std::error::Error for ExtractionPatternsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ExtractionPatternsConfig::default();
        assert_eq!(config.tola_to_grams(), 11.66);
        assert!(config.asset_quality.tiers.is_empty());
    }

    #[test]
    fn test_currency_multiplier() {
        let config = ExtractionPatternsConfig::default();
        assert_eq!(config.currency_multiplier("crore"), Some(10_000_000.0));
        assert_eq!(config.currency_multiplier("lakh"), Some(100_000.0));
        assert_eq!(config.currency_multiplier("LAKH"), Some(100_000.0));
    }

    #[test]
    fn test_compile_quality_patterns() {
        // P1.1 FIX: Test that quality patterns compile from config
        let yaml = r#"
asset_quality:
  tiers:
    - id: tier_1
      display_name: "Premium"
      patterns:
        en:
          - "premium"
          - "top\\s*grade"
        hi:
          - "प्रीमियम"
      value: "premium"
      confidence: 0.9
    - id: tier_2
      display_name: "Standard"
      patterns:
        en:
          - "standard"
          - "normal"
      value: "standard"
      confidence: 0.85
      default: true
  validation:
    min: 1
    max: 3
    unit: "grade"
"#;

        let config: ExtractionPatternsConfig = serde_yaml::from_str(yaml).unwrap();
        let compiled = config.compile_quality_patterns();

        assert_eq!(compiled.len(), 2);

        // Check first tier (premium)
        let premium = &compiled[0];
        assert_eq!(premium.id, "tier_1");
        assert_eq!(premium.value, "premium");
        assert!(!premium.is_default);
        assert!(premium.pattern.is_match("premium quality"));
        assert!(premium.pattern.is_match("top grade items"));
        assert!(premium.pattern.is_match("प्रीमियम")); // Hindi

        // Check second tier (standard - default)
        let standard = &compiled[1];
        assert_eq!(standard.id, "tier_2");
        assert_eq!(standard.value, "standard");
        assert!(standard.is_default);
        assert!(standard.pattern.is_match("normal quality"));
    }

    #[test]
    fn test_quality_validation_range() {
        let yaml = r#"
asset_quality:
  tiers: []
  validation:
    min: 10
    max: 24
    unit: "karat"
"#;

        let config: ExtractionPatternsConfig = serde_yaml::from_str(yaml).unwrap();
        let (min, max) = config.quality_validation_range();
        assert_eq!(min, 10);
        assert_eq!(max, 24);
    }
}
