//! P2-5 FIX: Domain-Agnostic Entity Extraction
//!
//! Extracts entities from text for collateral-based services:
//! - Offer amounts (with regional unit support: lakh/crore)
//! - Collateral weight (grams, tola, kg)
//! - Interest rates (percentage)
//! - Tenures (months, years)
//! - Customer names
//! - Collateral quality tier (e.g., karat for jewelry)
//!
//! # Design Principle
//!
//! This module uses domain-agnostic terminology:
//! - `collateral_weight` instead of gold_weight
//! - `collateral_quality` instead of gold_purity
//! - `current_provider` instead of current_lender
//!
//! Domain-specific providers (competitors) are loaded from config.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_text_processing::entities::EntityExtractor;
//!
//! let extractor = EntityExtractor::new();
//! let entities = extractor.extract("I want 5 lakh for 12 months at 10% interest");
//!
//! assert_eq!(entities.amount.unwrap().rupees(), 500000.0);
//! assert_eq!(entities.tenure.unwrap().months(), 12.0);
//! ```

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Currency value extracted from text
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Currency {
    /// Amount in base units (paise for INR)
    pub value: i64,
    /// Currency code (default: INR)
    pub unit: String,
    /// Original text span
    pub text: String,
}

impl Currency {
    /// Format as rupees string
    pub fn as_rupees(&self) -> String {
        let rupees = self.value / 100;
        format!("₹{}", rupees)
    }

    /// Get value in rupees
    pub fn rupees(&self) -> f64 {
        self.value as f64 / 100.0
    }
}

/// Weight value extracted from text
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Weight {
    /// Weight in milligrams
    pub value_mg: i64,
    /// Original unit (grams, tola, etc.)
    pub unit: String,
    /// Original text span
    pub text: String,
}

impl Weight {
    /// Get weight in grams
    pub fn grams(&self) -> f64 {
        self.value_mg as f64 / 1000.0
    }

    /// Get weight in tola
    pub fn tola(&self) -> f64 {
        self.grams() / 11.66
    }
}

/// Percentage value extracted from text
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Percentage {
    /// Percentage value (e.g., 10.5 for 10.5%)
    pub value: f64,
    /// Original text span
    pub text: String,
}

/// Duration value extracted from text
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Duration {
    /// Duration in days
    pub days: i32,
    /// Original unit (months, years, days)
    pub unit: String,
    /// Original text span
    pub text: String,
}

impl Duration {
    /// Get duration in months
    pub fn months(&self) -> f64 {
        self.days as f64 / 30.0
    }

    /// Get duration in years
    pub fn years(&self) -> f64 {
        self.days as f64 / 365.0
    }
}

/// All entities extracted from text
///
/// P18 FIX: Uses domain-agnostic field names with backward-compatible aliases.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractedEntities {
    /// Requested/offer amount
    pub amount: Option<Currency>,
    /// Collateral weight (domain-agnostic: could be gold, silver, etc.)
    pub collateral_weight: Option<Weight>,
    /// Interest rate
    pub interest_rate: Option<Percentage>,
    /// Service tenure
    pub tenure: Option<Duration>,
    /// Customer name (if mentioned)
    pub customer_name: Option<String>,
    /// Collateral quality tier (e.g., karat for jewelry, grade for other assets)
    pub collateral_quality: Option<u8>,
    /// Current provider (for balance transfer scenarios)
    pub current_provider: Option<String>,
}

impl ExtractedEntities {
    /// Check if any entities were extracted
    pub fn is_empty(&self) -> bool {
        self.amount.is_none()
            && self.collateral_weight.is_none()
            && self.interest_rate.is_none()
            && self.tenure.is_none()
            && self.customer_name.is_none()
            && self.collateral_quality.is_none()
            && self.current_provider.is_none()
    }

    /// Merge with another ExtractedEntities, preferring non-None values from other
    pub fn merge(&mut self, other: &ExtractedEntities) {
        if other.amount.is_some() {
            self.amount = other.amount.clone();
        }
        if other.collateral_weight.is_some() {
            self.collateral_weight = other.collateral_weight.clone();
        }
        if other.interest_rate.is_some() {
            self.interest_rate = other.interest_rate.clone();
        }
        if other.tenure.is_some() {
            self.tenure = other.tenure.clone();
        }
        if other.customer_name.is_some() {
            self.customer_name = other.customer_name.clone();
        }
        if other.collateral_quality.is_some() {
            self.collateral_quality = other.collateral_quality;
        }
        if other.current_provider.is_some() {
            self.current_provider = other.current_provider.clone();
        }
    }
}

// Compiled regex patterns
static AMOUNT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // P2-5 FIX: Use word boundaries to avoid matching "l" in "loan" as "lakh"
    Regex::new(r"(?i)(?:rs\.?|rupees?|₹|inr)?\s*(\d+(?:\.\d+)?)\s*\b(lakh|lac|lakhs?|crore|crores?|hazar|hazaar|thousand|k\b|l\b|cr\b)?\b?(?:\s*(?:rs\.?|rupees?|₹|inr))?").unwrap()
});

static HINDI_AMOUNT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    // Hindi number words
    Regex::new(r"(?i)(एक|दो|तीन|चार|पांच|पाँच|छह|छः|सात|आठ|नौ|दस|बीस|तीस|चालीस|पचास|साठ|सत्तर|अस्सी|नब्बे|सौ)\s*(लाख|करोड़|हज़ार|हजार)?").unwrap()
});

static WEIGHT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(gram|grams?|gm|g|tola|tolas?|kg|kilogram)s?").unwrap()
});

static RATE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:%|percent|प्रतिशत|prतिshat)").unwrap());

static TENURE_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(\d+)\s*(month|months?|year|years?|yr|yrs?|day|days?|mahine?|saal)s?").unwrap()
});

static PURITY_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(\d{1,2})\s*(?:k|karat|carat|kt)").unwrap());

static NAME_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:my\s+name\s+is|i\s+am|mera\s+naam|मेरा\s+नाम)\s+([A-Za-z\u0900-\u097F]+(?:\s+[A-Za-z\u0900-\u097F]+)?)").unwrap()
});

// P0 FIX: LENDER_PATTERNS removed - lenders must be loaded from domain config
// Use EntityExtractor::with_lenders() to provide domain-specific lender patterns
// from config/domains/{domain}/competitors.yaml

/// Domain-agnostic entity extractor
///
/// P18 FIX: Renamed from EntityExtractor to EntityExtractor for domain-agnostic operation.
/// Use `with_providers()` to load competitor/provider names from domain config.
/// P1.1 FIX: Quality tier validation range is now configurable.
pub struct EntityExtractor {
    /// Whether to extract Hindi/Devanagari numbers
    pub support_hindi: bool,
    /// Config-driven provider patterns (competitor names from domain config)
    provider_patterns: Vec<(String, Regex)>,
    /// P1.1 FIX: Quality tier validation range (min, max) - e.g., (10, 24) for karat
    quality_tier_range: (u8, u8),
}

impl Default for EntityExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityExtractor {
    /// Create a new extractor with default settings
    ///
    /// NOTE: For domain-agnostic operation, use `with_providers()` to provide
    /// competitor names from domain config. The default extractor has no
    /// provider patterns and will not extract current_provider values.
    ///
    /// P1.1 FIX: Quality tier validation defaults to (10, 24) for karat-style grading.
    /// Use `with_quality_tier_range()` or `with_config()` for custom validation.
    pub fn new() -> Self {
        Self {
            support_hindi: true,
            provider_patterns: Vec::new(), // P0 FIX: Empty by default, load from config
            quality_tier_range: (10, 24),  // Default karat range
        }
    }

    /// Create extractor with config-driven provider patterns
    ///
    /// # Arguments
    /// * `provider_names` - List of competitor/provider names from domain config
    ///
    /// # Example
    /// ```ignore
    /// let competitors = config.competitors.iter().map(|c| c.name.clone()).collect();
    /// let extractor = EntityExtractor::with_providers(competitors);
    /// ```
    pub fn with_providers(provider_names: Vec<String>) -> Self {
        let provider_patterns = provider_names
            .into_iter()
            .filter_map(|name| {
                // Create case-insensitive regex for the provider name
                let pattern = format!(r"(?i)\b{}\b", regex::escape(&name));
                Regex::new(&pattern)
                    .ok()
                    .map(|regex| (name, regex))
            })
            .collect();

        Self {
            support_hindi: true,
            provider_patterns,
            quality_tier_range: (10, 24), // Default karat range
        }
    }

    /// P1.1 FIX: Create extractor with custom quality tier validation range
    ///
    /// # Arguments
    /// * `min` - Minimum valid quality tier value
    /// * `max` - Maximum valid quality tier value
    ///
    /// # Example
    /// ```ignore
    /// // For karat-based jewelry (10-24 karat)
    /// let extractor = EntityExtractor::with_quality_tier_range(10, 24);
    ///
    /// // For grade-based assessment (1-5)
    /// let extractor = EntityExtractor::with_quality_tier_range(1, 5);
    /// ```
    pub fn with_quality_tier_range(min: u8, max: u8) -> Self {
        Self {
            support_hindi: true,
            provider_patterns: Vec::new(),
            quality_tier_range: (min, max),
        }
    }

    /// P1.1 FIX: Create extractor with full config (providers + quality range)
    ///
    /// # Arguments
    /// * `provider_names` - List of competitor/provider names from domain config
    /// * `quality_min` - Minimum valid quality tier value
    /// * `quality_max` - Maximum valid quality tier value
    pub fn with_config(provider_names: Vec<String>, quality_min: u8, quality_max: u8) -> Self {
        let provider_patterns = provider_names
            .into_iter()
            .filter_map(|name| {
                let pattern = format!(r"(?i)\b{}\b", regex::escape(&name));
                Regex::new(&pattern)
                    .ok()
                    .map(|regex| (name, regex))
            })
            .collect();

        Self {
            support_hindi: true,
            provider_patterns,
            quality_tier_range: (quality_min, quality_max),
        }
    }

    /// Add provider patterns from config (builder pattern)
    pub fn add_providers(mut self, provider_names: Vec<String>) -> Self {
        for name in provider_names {
            let pattern = format!(r"(?i)\b{}\b", regex::escape(&name));
            if let Ok(regex) = Regex::new(&pattern) {
                self.provider_patterns.push((name, regex));
            }
        }
        self
    }

    /// P1.1 FIX: Set quality tier validation range (builder pattern)
    pub fn with_tier_range(mut self, min: u8, max: u8) -> Self {
        self.quality_tier_range = (min, max);
        self
    }

    /// Extract all entities from text
    pub fn extract(&self, text: &str) -> ExtractedEntities {
        ExtractedEntities {
            amount: self.extract_amount(text),
            collateral_weight: self.extract_weight(text),
            interest_rate: self.extract_rate(text),
            tenure: self.extract_tenure(text),
            customer_name: self.extract_name(text),
            collateral_quality: self.extract_quality_tier(text),
            current_provider: self.extract_provider(text),
        }
    }
}

impl EntityExtractor {

    /// Extract loan amount
    pub fn extract_amount(&self, text: &str) -> Option<Currency> {
        // Try English pattern first
        if let Some(caps) = AMOUNT_PATTERN.captures(text) {
            let num_str = caps.get(1)?.as_str();
            let multiplier_str = caps.get(2).map(|m| m.as_str().to_lowercase());

            let base: f64 = num_str.parse().ok()?;
            let multiplier = match multiplier_str.as_deref() {
                Some("lakh") | Some("lac") | Some("lakhs") | Some("l") => 100_000.0,
                Some("crore") | Some("crores") | Some("cr") => 10_000_000.0,
                Some("hazar") | Some("hazaar") | Some("thousand") | Some("k") => 1_000.0,
                _ => 1.0,
            };

            let value = (base * multiplier * 100.0) as i64; // Store in paise
            return Some(Currency {
                value,
                unit: "INR".to_string(),
                text: caps.get(0)?.as_str().to_string(),
            });
        }

        // Try Hindi pattern
        if self.support_hindi {
            if let Some(caps) = HINDI_AMOUNT_PATTERN.captures(text) {
                let hindi_num = caps.get(1)?.as_str();
                let multiplier_str = caps.get(2).map(|m| m.as_str());

                // P2.2 FIX: Use shared Hindi module
                let base = crate::hindi::word_to_number(hindi_num)?;
                let multiplier = match multiplier_str {
                    Some("लाख") => 100_000.0,
                    Some("करोड़") => 10_000_000.0,
                    Some("हज़ार") | Some("हजार") => 1_000.0,
                    _ => 1.0,
                };

                let value = (base * multiplier * 100.0) as i64;
                return Some(Currency {
                    value,
                    unit: "INR".to_string(),
                    text: caps.get(0)?.as_str().to_string(),
                });
            }
        }

        None
    }

    /// Extract collateral weight (domain-agnostic)
    pub fn extract_weight(&self, text: &str) -> Option<Weight> {
        let caps = WEIGHT_PATTERN.captures(text)?;
        let num_str = caps.get(1)?.as_str();
        let unit_str = caps.get(2)?.as_str().to_lowercase();

        let base: f64 = num_str.parse().ok()?;

        // Convert to milligrams
        let (value_mg, unit) = match unit_str.as_str() {
            "gram" | "grams" | "gm" | "g" => ((base * 1000.0) as i64, "grams"),
            "tola" | "tolas" => ((base * 11660.0) as i64, "tola"), // 1 tola = 11.66 grams
            "kg" | "kilogram" => ((base * 1_000_000.0) as i64, "kg"),
            _ => return None,
        };

        Some(Weight {
            value_mg,
            unit: unit.to_string(),
            text: caps.get(0)?.as_str().to_string(),
        })
    }

    /// Extract interest rate
    pub fn extract_rate(&self, text: &str) -> Option<Percentage> {
        let caps = RATE_PATTERN.captures(text)?;
        let value: f64 = caps.get(1)?.as_str().parse().ok()?;

        Some(Percentage {
            value,
            text: caps.get(0)?.as_str().to_string(),
        })
    }

    /// Extract loan tenure
    pub fn extract_tenure(&self, text: &str) -> Option<Duration> {
        let caps = TENURE_PATTERN.captures(text)?;
        let num: i32 = caps.get(1)?.as_str().parse().ok()?;
        let unit_str = caps.get(2)?.as_str().to_lowercase();

        let (days, unit) = match unit_str.as_str() {
            "month" | "months" | "mahine" => (num * 30, "months"),
            "year" | "years" | "yr" | "yrs" | "saal" => (num * 365, "years"),
            "day" | "days" => (num, "days"),
            _ => return None,
        };

        Some(Duration {
            days,
            unit: unit.to_string(),
            text: caps.get(0)?.as_str().to_string(),
        })
    }

    /// Extract customer name
    pub fn extract_name(&self, text: &str) -> Option<String> {
        let caps = NAME_PATTERN.captures(text)?;
        Some(caps.get(1)?.as_str().trim().to_string())
    }

    /// Extract collateral quality tier (e.g., karat for jewelry)
    ///
    /// Returns a numeric quality tier (e.g., 18, 22, 24 for karat).
    ///
    /// P1.1 FIX: Validation range is now configurable via `quality_tier_range`.
    /// Use `with_quality_tier_range()` or `with_config()` for custom validation.
    pub fn extract_quality_tier(&self, text: &str) -> Option<u8> {
        let caps = PURITY_PATTERN.captures(text)?;
        let tier: u8 = caps.get(1)?.as_str().parse().ok()?;

        // P1.1 FIX: Use config-driven validation range
        let (min, max) = self.quality_tier_range;
        if (min..=max).contains(&tier) {
            Some(tier)
        } else {
            None
        }
    }

    /// Extract current provider name
    ///
    /// Uses config-driven provider patterns. Returns None if no patterns configured.
    /// For domain-specific extraction, create extractor with `with_providers()`.
    pub fn extract_provider(&self, text: &str) -> Option<String> {
        // Use instance provider_patterns instead of hardcoded static patterns
        for (name, pattern) in &self.provider_patterns {
            if pattern.is_match(text) {
                return Some(name.clone());
            }
        }
        None
    }

    // P2.2 FIX: Removed duplicate hindi_to_number() - now uses crate::hindi::word_to_number()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_amount_lakh() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_amount("I want 5 lakh loan");
        assert!(result.is_some());
        let amount = result.unwrap();
        assert_eq!(amount.rupees(), 500000.0);
    }

    #[test]
    fn test_extract_amount_crore() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_amount("Need 1 crore for business");
        assert!(result.is_some());
        let amount = result.unwrap();
        assert_eq!(amount.rupees(), 10_000_000.0);
    }

    #[test]
    fn test_extract_amount_with_currency_symbol() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_amount("Rs. 50000 loan needed");
        assert!(result.is_some());
        let amount = result.unwrap();
        assert_eq!(amount.rupees(), 50000.0);
    }

    #[test]
    fn test_extract_weight_grams() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_weight("I have 50 grams of gold");
        assert!(result.is_some());
        let weight = result.unwrap();
        assert_eq!(weight.grams(), 50.0);
    }

    #[test]
    fn test_extract_weight_tola() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_weight("Gold weighing 10 tola");
        assert!(result.is_some());
        let weight = result.unwrap();
        // 10 tola = 116.6 grams
        assert!((weight.grams() - 116.6).abs() < 0.1);
    }

    #[test]
    fn test_extract_rate() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_rate("Interest rate is 10.5%");
        assert!(result.is_some());
        let rate = result.unwrap();
        assert_eq!(rate.value, 10.5);
    }

    #[test]
    fn test_extract_tenure_months() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_tenure("12 month loan");
        assert!(result.is_some());
        let tenure = result.unwrap();
        assert_eq!(tenure.months(), 12.0);
    }

    #[test]
    fn test_extract_tenure_years() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_tenure("2 year tenure");
        assert!(result.is_some());
        let tenure = result.unwrap();
        assert_eq!(tenure.years(), 2.0);
    }

    #[test]
    fn test_extract_name() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_name("My name is Rajesh Kumar");
        assert_eq!(result, Some("Rajesh Kumar".to_string()));
    }

    #[test]
    fn test_extract_quality_tier() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_quality_tier("22k gold");
        assert_eq!(result, Some(22));

        let result = extractor.extract_quality_tier("18 karat gold");
        assert_eq!(result, Some(18));
    }

    #[test]
    fn test_extract_provider() {
        // Test with config-driven provider patterns
        let extractor = EntityExtractor::with_providers(vec![
            "Provider A".to_string(),
            "Provider B".to_string(),
            "Provider C".to_string(),
        ]);

        let result = extractor.extract_provider("I have loan from Provider A Finance");
        assert_eq!(result, Some("Provider A".to_string()));

        let result = extractor.extract_provider("Currently with Provider B");
        assert_eq!(result, Some("Provider B".to_string()));
    }

    #[test]
    fn test_extract_provider_no_config() {
        // Test that default extractor returns None for providers
        let extractor = EntityExtractor::new();

        let result = extractor.extract_provider("I have loan from Provider A Finance");
        assert_eq!(result, None); // No patterns configured = no extraction
    }

    #[test]
    fn test_extract_all_entities() {
        // Test with config-driven provider patterns
        let extractor = EntityExtractor::with_providers(vec![
            "Provider A".to_string(),
            "Provider B".to_string(),
        ]);

        let text = "My name is Rahul. I want 5 lakh loan for 12 months at 10% interest. I have 50 grams of 22k gold. Currently with Provider A.";
        let entities = extractor.extract(text);

        assert!(entities.amount.is_some());
        assert_eq!(entities.amount.as_ref().unwrap().rupees(), 500000.0);

        assert!(entities.tenure.is_some());
        assert_eq!(entities.tenure.as_ref().unwrap().months(), 12.0);

        assert!(entities.interest_rate.is_some());
        assert_eq!(entities.interest_rate.as_ref().unwrap().value, 10.0);

        // Use domain-agnostic field names
        assert!(entities.collateral_weight.is_some());
        assert_eq!(entities.collateral_weight.as_ref().unwrap().grams(), 50.0);

        assert_eq!(entities.collateral_quality, Some(22));
        assert_eq!(entities.customer_name, Some("Rahul".to_string()));
        assert_eq!(entities.current_provider, Some("Provider A".to_string()));
    }

    #[test]
    fn test_hindi_amount() {
        let extractor = EntityExtractor::new();

        let result = extractor.extract_amount("पांच लाख");
        assert!(result.is_some());
        let amount = result.unwrap();
        assert_eq!(amount.rupees(), 500000.0);
    }

    #[test]
    fn test_merge_entities() {
        let mut entities1 = ExtractedEntities::default();
        entities1.amount = Some(Currency {
            value: 50000000, // 5 lakh in paise
            unit: "INR".to_string(),
            text: "5 lakh".to_string(),
        });

        let mut entities2 = ExtractedEntities::default();
        entities2.tenure = Some(Duration {
            days: 360,
            unit: "months".to_string(),
            text: "12 months".to_string(),
        });

        entities1.merge(&entities2);

        assert!(entities1.amount.is_some());
        assert!(entities1.tenure.is_some());
    }

    #[test]
    fn test_empty_text() {
        let extractor = EntityExtractor::new();
        let entities = extractor.extract("");
        assert!(entities.is_empty());
    }

    #[test]
    fn test_configurable_quality_tier_range() {
        // P1.1 FIX: Test that quality tier range is configurable

        // Default extractor uses karat range (10-24)
        let default_extractor = EntityExtractor::new();
        assert_eq!(default_extractor.extract_quality_tier("22k gold"), Some(22));
        assert_eq!(default_extractor.extract_quality_tier("24k gold"), Some(24));
        assert_eq!(default_extractor.extract_quality_tier("5k gold"), None); // Out of default range

        // Custom range extractor (1-10)
        let custom_extractor = EntityExtractor::with_quality_tier_range(1, 10);
        assert_eq!(custom_extractor.extract_quality_tier("5k quality"), Some(5));
        assert_eq!(custom_extractor.extract_quality_tier("10k quality"), Some(10));
        assert_eq!(custom_extractor.extract_quality_tier("22k quality"), None); // Out of custom range

        // Builder pattern
        let builder_extractor = EntityExtractor::new().with_tier_range(1, 5);
        assert_eq!(builder_extractor.extract_quality_tier("3k grade"), Some(3));
        assert_eq!(builder_extractor.extract_quality_tier("22k grade"), None); // Out of range

        // Full config
        let config_extractor = EntityExtractor::with_config(
            vec!["TestProvider".to_string()],
            14,
            24,
        );
        assert_eq!(config_extractor.extract_quality_tier("18k gold"), Some(18));
        assert_eq!(config_extractor.extract_quality_tier("10k gold"), None); // Below custom min
    }
}
