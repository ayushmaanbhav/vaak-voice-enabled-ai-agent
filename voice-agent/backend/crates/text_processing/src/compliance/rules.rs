//! Compliance rules definition and loading

use serde::{Deserialize, Serialize};

/// Compliance rules structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRules {
    /// Version of rules
    pub version: String,
    /// Forbidden phrases (critical violations)
    #[serde(default)]
    pub forbidden_phrases: Vec<String>,
    /// Claims that require disclaimers
    #[serde(default)]
    pub claims_requiring_disclaimer: Vec<ClaimRule>,
    /// Rate validation rules
    #[serde(default)]
    pub rate_rules: RateRules,
    /// Required disclosures
    #[serde(default)]
    pub required_disclosures: Vec<RequiredDisclosure>,
    /// Competitor mention rules
    #[serde(default)]
    pub competitor_rules: CompetitorRules,
}

/// Rule for claims that need disclaimers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimRule {
    /// Pattern to match (regex)
    pub pattern: String,
    /// Required disclaimer text
    pub disclaimer: String,
    /// Description of the rule
    #[serde(default)]
    pub description: String,
}

/// Interest rate validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateRules {
    /// Minimum allowed rate (%)
    pub min_rate: f32,
    /// Maximum allowed rate (%)
    pub max_rate: f32,
    /// Rate precision required
    #[serde(default = "default_precision")]
    pub precision: u32,
}

fn default_precision() -> u32 {
    2
}

impl Default for RateRules {
    fn default() -> Self {
        // Use sentinel values that work for any domain
        // Actual rate bounds should be loaded from compliance.yaml
        Self {
            min_rate: 0.1,   // Technical minimum (any positive rate)
            max_rate: 100.0, // Technical maximum (no rate above 100%)
            precision: 2,
        }
    }
}

/// Required disclosure for specific contexts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredDisclosure {
    /// When to add this disclosure
    pub trigger_pattern: String,
    /// The disclosure text
    pub disclosure: String,
    /// Position: start, end
    #[serde(default = "default_position")]
    pub position: String,
}

fn default_position() -> String {
    "end".to_string()
}

/// Rules for competitor mentions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompetitorRules {
    /// List of competitors
    pub competitors: Vec<String>,
    /// Whether disparagement is allowed
    #[serde(default)]
    pub allow_disparagement: bool,
    /// Whether comparison is allowed
    #[serde(default = "default_true")]
    pub allow_comparison: bool,
}

impl CompetitorRules {
    /// P16 FIX: Create from config-provided competitor names
    ///
    /// Use this with CompetitorsConfig::all_names_and_aliases():
    /// ```ignore
    /// let config = CompetitorsConfig::load("competitors.yaml")?;
    /// let rules = CompetitorRules::from_names(config.all_names_and_aliases());
    /// ```
    pub fn from_names<I, S>(competitors: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            competitors: competitors.into_iter().map(|s| s.into()).collect(),
            allow_disparagement: false,
            allow_comparison: true,
        }
    }
}

fn default_true() -> bool {
    true
}

impl Default for ComplianceRules {
    fn default() -> Self {
        default_rules()
    }
}

/// Load rules from TOML file
pub fn load_rules(path: &str) -> Result<ComplianceRules, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read rules file: {}", e))?;
    toml::from_str(&content).map_err(|e| format!("Failed to parse rules file: {}", e))
}

/// Get default compliance rules (domain-agnostic)
///
/// Returns minimal defaults. For production use, load from compliance.yaml:
/// ```ignore
/// let compliance_config = ComplianceConfig::load("config/domains/{domain}/compliance.yaml")?;
/// let rules = compliance_config.to_rules();
/// ```
pub fn default_rules() -> ComplianceRules {
    // Domain-agnostic defaults - all domain-specific rules come from config
    ComplianceRules {
        version: "1.0.0".to_string(),
        // Empty - load from compliance.yaml:forbidden_phrases
        forbidden_phrases: vec![],
        // Empty - load from compliance.yaml:claims_requiring_disclaimer
        claims_requiring_disclaimer: vec![],
        // Sentinel values - load actual bounds from compliance.yaml:rate_rules
        rate_rules: RateRules::default(),
        // Empty - load from compliance.yaml:required_disclosures
        required_disclosures: vec![],
        // Empty - load competitor names from competitors.yaml via CompetitorsConfig
        competitor_rules: CompetitorRules::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules() {
        let rules = default_rules();
        assert_eq!(rules.version, "1.0.0");
        assert!(!rules.forbidden_phrases.is_empty());
        assert!(!rules.claims_requiring_disclaimer.is_empty());
    }

    #[test]
    fn test_rate_rules() {
        let rate_rules = RateRules::default();
        assert!(rate_rules.min_rate > 0.0);
        assert!(rate_rules.max_rate > rate_rules.min_rate);
    }

    #[test]
    fn test_serialize_rules() {
        let rules = default_rules();
        let toml_str = toml::to_string_pretty(&rules).unwrap();
        assert!(toml_str.contains("version"));
        assert!(toml_str.contains("forbidden_phrases"));
    }
}
