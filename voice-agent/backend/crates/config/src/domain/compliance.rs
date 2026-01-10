//! Compliance Configuration
//!
//! # P21 FIX: Config-driven compliance rules
//!
//! Defines compliance rules loaded from domain config YAML.
//! Enables domain-agnostic compliance checking by moving all
//! regulatory and content rules to configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root compliance configuration loaded from compliance.yaml
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceConfig {
    /// Configuration version
    #[serde(default = "default_version")]
    pub version: String,

    /// Rate validation rules
    #[serde(default)]
    pub rate_rules: RateRules,

    /// Phrases that are forbidden in agent output
    #[serde(default)]
    pub forbidden_phrases: Vec<String>,

    /// Claims that require disclaimers
    #[serde(default)]
    pub claims_requiring_disclaimer: Vec<ClaimRule>,

    /// Required disclosures based on context
    #[serde(default)]
    pub required_disclosures: Vec<RequiredDisclosure>,

    /// Rules for competitor mentions
    #[serde(default)]
    pub competitor_rules: CompetitorRules,

    /// Regulatory body references
    #[serde(default)]
    pub regulatory: RegulatoryInfo,

    /// Language-specific compliance settings
    #[serde(default)]
    pub language_rules: HashMap<String, LanguageRules>,

    /// Severity levels for violations
    #[serde(default)]
    pub severity_levels: SeverityLevels,

    /// Auto-correction rules
    #[serde(default)]
    pub auto_corrections: AutoCorrections,

    /// P16 FIX: AI disclosure messages by language (RBI compliance)
    /// Key is language code (en, hi, mr, ta, etc.), value is the disclosure message
    #[serde(default)]
    pub ai_disclosures: HashMap<String, String>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

/// Rate validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateRules {
    /// Minimum allowed interest rate (%)
    pub min_rate: f64,
    /// Maximum allowed interest rate (%)
    pub max_rate: f64,
    /// Decimal precision for rate display
    #[serde(default = "default_precision")]
    pub precision: u8,
}

fn default_precision() -> u8 {
    2
}

impl Default for RateRules {
    fn default() -> Self {
        Self {
            min_rate: 7.0,
            max_rate: 24.0,
            precision: 2,
        }
    }
}

/// Rule for claims that require disclaimers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRule {
    /// Regex pattern to detect the claim
    pub pattern: String,
    /// Disclaimer text to append
    pub disclaimer: String,
    /// Description of this rule
    #[serde(default)]
    pub description: String,
}

/// Required disclosure based on context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredDisclosure {
    /// Pattern that triggers this disclosure
    pub trigger_pattern: String,
    /// Disclosure text to include
    #[serde(default)]
    pub disclosure: String,
    /// Position of disclosure (start, end)
    #[serde(default = "default_position")]
    pub position: String,
}

fn default_position() -> String {
    "end".to_string()
}

/// Rules for competitor mentions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompetitorRules {
    /// List of known competitor names
    #[serde(default)]
    pub competitors: Vec<String>,
    /// Whether disparaging competitors is allowed
    #[serde(default)]
    pub allow_disparagement: bool,
    /// Whether factual comparison is allowed
    #[serde(default = "default_true")]
    pub allow_comparison: bool,
}

fn default_true() -> bool {
    true
}

/// Regulatory body information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegulatoryInfo {
    /// Primary regulator code (e.g., "RBI")
    #[serde(default)]
    pub primary_regulator: String,
    /// Full name of regulator
    #[serde(default)]
    pub regulator_full_name: String,
    /// License type held
    #[serde(default)]
    pub license_type: String,
    /// Standard compliance statement
    #[serde(default)]
    pub compliance_statement: String,
}

/// Language-specific compliance settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LanguageRules {
    /// Prefix for disclaimers
    #[serde(default)]
    pub disclaimer_prefix: String,
    /// Standard terms reference
    #[serde(default)]
    pub terms_reference: String,
}

/// Severity levels for different violation types
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeverityLevels {
    /// Critical violations
    #[serde(default)]
    pub critical: Vec<String>,
    /// Warning-level violations
    #[serde(default)]
    pub warning: Vec<String>,
    /// Informational items
    #[serde(default)]
    pub info: Vec<String>,
}

/// Auto-correction rules
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoCorrections {
    /// Phrase replacements
    #[serde(default)]
    pub replacements: HashMap<String, String>,
}

impl ComplianceConfig {
    /// Load compliance config from YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ComplianceConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ComplianceConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| ComplianceConfigError::ParseError(e.to_string()))
    }

    /// Check if a phrase is forbidden
    pub fn is_forbidden(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        self.forbidden_phrases
            .iter()
            .any(|p| lower.contains(&p.to_lowercase()))
    }

    /// Check if rate is within allowed bounds
    pub fn is_rate_valid(&self, rate: f64) -> bool {
        rate >= self.rate_rules.min_rate && rate <= self.rate_rules.max_rate
    }

    /// Get auto-correction for a phrase
    pub fn get_correction(&self, phrase: &str) -> Option<&str> {
        self.auto_corrections
            .replacements
            .get(phrase)
            .map(|s| s.as_str())
    }

    /// Get language-specific rules
    pub fn language_rules(&self, language: &str) -> Option<&LanguageRules> {
        self.language_rules.get(language)
    }

    /// Check if a competitor name is known
    pub fn is_known_competitor(&self, name: &str) -> bool {
        let lower = name.to_lowercase();
        self.competitor_rules
            .competitors
            .iter()
            .any(|c| lower.contains(&c.to_lowercase()))
    }

    /// P16 FIX: Get AI disclosure message for a language
    ///
    /// Returns the localized AI disclosure message. Falls back to English
    /// if the requested language is not available. Returns a default message
    /// if no disclosures are configured.
    pub fn get_ai_disclosure(&self, language: &str) -> &str {
        // Try exact match first
        if let Some(msg) = self.ai_disclosures.get(language) {
            return msg.as_str();
        }

        // Try normalized language code (e.g., "hindi" -> "hi")
        let normalized = match language {
            "hindi" => "hi",
            "marathi" => "mr",
            "tamil" => "ta",
            "telugu" => "te",
            "bengali" => "bn",
            "gujarati" => "gu",
            "kannada" => "kn",
            "malayalam" => "ml",
            "punjabi" => "pa",
            "odia" => "or",
            _ => language,
        };

        if let Some(msg) = self.ai_disclosures.get(normalized) {
            return msg.as_str();
        }

        // Fall back to English
        if let Some(msg) = self.ai_disclosures.get("en") {
            return msg.as_str();
        }

        // Default message if nothing configured
        "This is an AI assistant. You can speak with a human agent at any time by saying 'speak to agent'."
    }
}

/// Errors during compliance config loading
#[derive(Debug)]
pub enum ComplianceConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for ComplianceConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => write!(f, "Compliance config not found at {}: {}", path, err),
            Self::ParseError(err) => write!(f, "Failed to parse compliance config: {}", err),
        }
    }
}

impl std::error::Error for ComplianceConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ComplianceConfig::default();
        assert_eq!(config.rate_rules.min_rate, 7.0);
        assert_eq!(config.rate_rules.max_rate, 24.0);
        assert!(config.forbidden_phrases.is_empty());
    }

    #[test]
    fn test_rate_validation() {
        let config = ComplianceConfig::default();
        assert!(config.is_rate_valid(10.0));
        assert!(config.is_rate_valid(7.0));
        assert!(config.is_rate_valid(24.0));
        assert!(!config.is_rate_valid(6.0));
        assert!(!config.is_rate_valid(25.0));
    }

    #[test]
    fn test_forbidden_phrase_detection() {
        let mut config = ComplianceConfig::default();
        config.forbidden_phrases = vec!["guaranteed approval".to_string()];

        assert!(config.is_forbidden("Get guaranteed approval today!"));
        assert!(config.is_forbidden("GUARANTEED APPROVAL"));
        assert!(!config.is_forbidden("High approval rate"));
    }
}
