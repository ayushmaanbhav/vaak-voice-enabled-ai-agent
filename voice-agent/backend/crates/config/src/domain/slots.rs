//! Slot Schema Configuration
//!
//! Defines config-driven slot definitions for dialogue state tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Slot schema loaded from slots.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotsConfig {
    /// Slot definitions keyed by slot name
    #[serde(default)]
    pub slots: HashMap<String, SlotDefinition>,
    /// Goal definitions keyed by goal name
    #[serde(default)]
    pub goals: HashMap<String, GoalDefinition>,
    /// Intent to goal mapping
    #[serde(default)]
    pub intent_mapping: HashMap<String, Vec<String>>,
    /// P16 FIX: Slot name aliases for normalization
    /// Maps alternative slot names to canonical fact keys
    /// e.g., {"weight": "asset_quantity", "gold_weight": "asset_quantity"}
    #[serde(default)]
    pub slot_aliases: HashMap<String, String>,
    /// P16 FIX: Slots that should trigger customer name update (instead of fact storage)
    #[serde(default)]
    pub customer_name_slots: Vec<String>,
}

impl Default for SlotsConfig {
    fn default() -> Self {
        Self {
            slots: HashMap::new(),
            goals: HashMap::new(),
            intent_mapping: HashMap::new(),
            slot_aliases: HashMap::new(),
            customer_name_slots: vec!["customer_name".to_string(), "name".to_string()],
        }
    }
}

impl SlotsConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, SlotsConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            SlotsConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content).map_err(|e| SlotsConfigError::ParseError(e.to_string()))
    }

    /// Get a slot definition by name
    pub fn get_slot(&self, name: &str) -> Option<&SlotDefinition> {
        self.slots.get(name)
    }

    /// Get a goal definition by name
    pub fn get_goal(&self, name: &str) -> Option<&GoalDefinition> {
        self.goals.get(name)
    }

    /// Map an intent to a goal
    pub fn goal_for_intent(&self, intent: &str) -> Option<&str> {
        for (goal, intents) in &self.intent_mapping {
            if intents.iter().any(|i| i == intent) {
                return Some(goal);
            }
        }
        None
    }

    /// Get extraction patterns for a slot
    pub fn extraction_patterns(&self, slot_name: &str, language: &str) -> Vec<&str> {
        self.slots
            .get(slot_name)
            .and_then(|s| s.extraction_patterns.as_ref())
            .and_then(|p| p.get(language))
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get enum values for a slot
    pub fn enum_values(&self, slot_name: &str) -> Vec<&EnumValue> {
        self.slots
            .get(slot_name)
            .and_then(|s| s.values.as_ref())
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// P18 FIX: Get quality factor for a tier (domain-agnostic)
    /// Replaces domain-specific purity_factor method
    pub fn quality_factor(&self, slot_name: &str, tier_id: &str) -> f64 {
        self.slots
            .get(slot_name)
            .and_then(|s| s.values.as_ref())
            .and_then(|values| values.iter().find(|v| v.id == tier_id))
            .and_then(|v| v.quality_factor)
            .unwrap_or(1.0)
    }

    // ====== P18 FIX: Config-Driven Quality Tier Parsing ======

    /// Parse free text to quality tier ID using config patterns
    ///
    /// P18 FIX: Replaces hardcoded parse_purity_id() in dst/slots.rs
    /// Returns the tier ID that best matches the input text
    pub fn parse_quality_tier(&self, slot_name: &str, text: &str) -> Option<String> {
        let slot = self.slots.get(slot_name)?;
        let parsing = slot.parsing.as_ref()?;
        let lower = text.to_lowercase();

        // Try numeric patterns first (e.g., "24" -> tier_1)
        for rule in &parsing.numeric_patterns {
            if lower.contains(&rule.value.to_string()) {
                return Some(rule.tier_id.clone());
            }
        }

        // Try text patterns (e.g., "24k", "24 karat" -> tier_1)
        for (tier_id, patterns) in &parsing.text_patterns {
            for pattern in patterns {
                if lower.contains(&pattern.to_lowercase()) {
                    return Some(tier_id.clone());
                }
            }
        }

        // Return default if configured
        if !parsing.default_id.is_empty() {
            Some(parsing.default_id.clone())
        } else {
            None
        }
    }

    /// Get display string for a tier ID
    ///
    /// P18 FIX: Replaces hardcoded format_purity_display() in dst/slots.rs
    pub fn format_quality_display(&self, slot_name: &str, tier_id: &str) -> Option<&str> {
        let slot = self.slots.get(slot_name)?;
        slot.values.as_ref()?
            .iter()
            .find(|v| v.id == tier_id)
            .map(|v| v.display.as_str())
    }

    /// Get all tier IDs for a slot
    pub fn quality_tier_ids(&self, slot_name: &str) -> Vec<&str> {
        self.slots
            .get(slot_name)
            .and_then(|s| s.values.as_ref())
            .map(|v| v.iter().map(|e| e.id.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get typical rate for a lender
    pub fn lender_rate(&self, lender_id: &str) -> Option<f64> {
        self.slots
            .get("current_lender")
            .and_then(|s| s.values.as_ref())
            .and_then(|values| values.iter().find(|v| v.id == lender_id))
            .and_then(|v| v.typical_rate)
    }

    /// Get unit conversion factor
    pub fn unit_conversion(&self, slot_name: &str, unit: &str) -> Option<f64> {
        self.slots
            .get(slot_name)
            .and_then(|s| s.unit_conversions.as_ref())
            .and_then(|c| c.get(unit))
            .copied()
    }

    // ====== P16 FIX: Slot Alias Resolution ======

    /// Resolve a slot name to its canonical fact key using aliases
    /// Returns the canonical key if an alias exists, otherwise None
    pub fn resolve_slot_alias(&self, slot_name: &str) -> Option<&str> {
        self.slot_aliases.get(slot_name).map(|s| s.as_str())
    }

    /// Check if a slot name should trigger customer name update
    pub fn is_customer_name_slot(&self, slot_name: &str) -> bool {
        self.customer_name_slots.iter().any(|s| s == slot_name)
    }

    /// Get the canonical fact key for a slot, checking aliases first
    /// If no alias exists, returns the original slot name
    pub fn canonical_fact_key<'a>(&'a self, slot_name: &'a str) -> &'a str {
        self.slot_aliases.get(slot_name)
            .map(|s| s.as_str())
            .unwrap_or(slot_name)
    }

    /// Check if slot aliases are configured
    pub fn has_slot_aliases(&self) -> bool {
        !self.slot_aliases.is_empty()
    }

    /// P19 FIX: Get display label for a slot
    /// Returns the configured display_name if set, otherwise formats the slot name
    /// (e.g., "asset_quantity" -> "Asset Quantity")
    pub fn get_slot_display_label(&self, slot_name: &str) -> String {
        // Check canonical slot name first
        let canonical = self.canonical_fact_key(slot_name);
        if let Some(slot) = self.slots.get(canonical) {
            if let Some(ref display) = slot.display_name {
                return display.clone();
            }
        }
        // Check original slot name
        if let Some(slot) = self.slots.get(slot_name) {
            if let Some(ref display) = slot.display_name {
                return display.clone();
            }
        }
        // Fallback: format slot name (snake_case -> Title Case)
        slot_name
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// P21 FIX: Get all slot names defined in config
    /// Returns a vector of all slot identifiers
    pub fn all_slot_names(&self) -> Vec<&str> {
        self.slots.keys().map(|s| s.as_str()).collect()
    }

    /// P21 FIX: Get all slot display labels as (slot_name, display_label) pairs
    /// Useful for building display mappings without hardcoding slot names
    pub fn all_slot_display_labels(&self) -> Vec<(String, String)> {
        self.slots
            .keys()
            .map(|name| {
                let label = self.get_slot_display_label(name);
                (name.clone(), label)
            })
            .collect()
    }

    /// P20 FIX: Get asset unit from asset_quantity slot (e.g., "grams")
    /// Returns default "unit" if not configured
    pub fn asset_unit(&self) -> &str {
        self.slots
            .get("asset_quantity")
            .and_then(|s| s.unit.as_deref())
            .unwrap_or("unit")
    }

    /// P20 FIX: Get unit for a specific slot
    pub fn slot_unit(&self, slot_name: &str) -> Option<&str> {
        self.slots
            .get(slot_name)
            .and_then(|s| s.unit.as_deref())
    }

    /// P20 FIX: Get currency for a specific slot
    pub fn slot_currency(&self, slot_name: &str) -> Option<&str> {
        self.slots
            .get(slot_name)
            .and_then(|s| s.currency.as_deref())
    }
}

/// Definition for a single slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotDefinition {
    /// Slot type: string, number, enum, date
    #[serde(rename = "type")]
    pub slot_type: SlotType,
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    /// P19 FIX: Display label for UI/summary (e.g., "Gold Weight" for asset_quantity slot)
    #[serde(default)]
    pub display_name: Option<String>,
    /// Validation regex (for string/number types)
    #[serde(default)]
    pub validation: Option<String>,
    /// Minimum value (for number types)
    #[serde(default)]
    pub min: Option<f64>,
    /// Maximum value (for number types)
    #[serde(default)]
    pub max: Option<f64>,
    /// Enum values (for enum types)
    #[serde(default)]
    pub values: Option<Vec<EnumValue>>,
    /// Default value
    #[serde(default)]
    pub default: Option<String>,
    /// Extraction patterns by language
    #[serde(default)]
    pub extraction_patterns: Option<HashMap<String, Vec<String>>>,
    /// Unit conversions (e.g., tola -> grams)
    #[serde(default)]
    pub unit_conversions: Option<HashMap<String, f64>>,
    /// P18 FIX: Parsing configuration for enum slots (quality tiers)
    #[serde(default)]
    pub parsing: Option<EnumParsingConfig>,
    /// P20 FIX: Unit of measurement (e.g., "grams" for asset_quantity)
    #[serde(default)]
    pub unit: Option<String>,
    /// P20 FIX: Currency code (e.g., "INR" for offer_amount)
    #[serde(default)]
    pub currency: Option<String>,
}

/// Slot type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlotType {
    String,
    Number,
    Enum,
    Date,
}

/// Enum value definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValue {
    /// Unique identifier
    pub id: String,
    /// Display name (full form, e.g., "24 karat")
    pub display: String,
    /// P20 FIX: Short code for compact display (e.g., "24K")
    /// If not set, falls back to first pattern or id
    #[serde(default)]
    pub short_code: Option<String>,
    /// P20 FIX: Description of this tier (e.g., "Pure gold (99.9%)")
    #[serde(default)]
    pub description: Option<String>,
    /// Pattern strings for extraction
    #[serde(default)]
    pub patterns: Vec<String>,
    /// P18 FIX: Renamed from purity_factor to quality_factor for domain-agnostic design
    /// Quality/variant factor (e.g., purity for gold, condition for used items)
    #[serde(default, alias = "purity_factor")]
    pub quality_factor: Option<f64>,
    /// Typical rate (for lenders/competitors)
    #[serde(default)]
    pub typical_rate: Option<f64>,
}

impl EnumValue {
    /// Legacy accessor for purity_factor (now quality_factor)
    pub fn purity_factor(&self) -> Option<f64> {
        self.quality_factor
    }

    /// P20 FIX: Get short code for this tier (e.g., "24K")
    /// Falls back to first pattern (uppercased) or id if not set
    pub fn short_code(&self) -> &str {
        self.short_code
            .as_deref()
            .or_else(|| self.patterns.first().map(|s| s.as_str()))
            .unwrap_or(&self.id)
    }

    /// P20 FIX: Get description for this tier
    /// Returns empty string if not set
    pub fn description(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }
}

// ============================================================================
// P18 FIX: Enum Parsing Configuration (Domain-Agnostic Quality Tiers)
// ============================================================================

/// Numeric pattern rule for parsing (e.g., "24" -> "tier_1")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericPatternRule {
    /// Numeric value to match in text
    pub value: i32,
    /// Tier ID to return when matched
    pub tier_id: String,
}

/// Parsing configuration for enum slots (domain-agnostic quality tiers)
///
/// P18 FIX: Replaces hardcoded parse_purity_id() and format_purity_display()
/// functions with config-driven parsing.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnumParsingConfig {
    /// Default tier ID if no pattern matches
    #[serde(default)]
    pub default_id: String,
    /// Numeric patterns (e.g., "24" in text -> tier_1)
    #[serde(default)]
    pub numeric_patterns: Vec<NumericPatternRule>,
    /// Text patterns by tier ID (tier_id -> list of patterns)
    #[serde(default)]
    pub text_patterns: HashMap<String, Vec<String>>,
}

/// Goal definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalDefinition {
    /// Description of the goal
    #[serde(default)]
    pub description: String,
    /// Required slots for this goal
    #[serde(default)]
    pub required_slots: Vec<String>,
    /// Optional slots that enhance the goal
    #[serde(default)]
    pub optional_slots: Vec<String>,
    /// Action to take when goal is complete
    #[serde(default)]
    pub completion_action: Option<String>,
}

/// Errors when loading slot configuration
#[derive(Debug)]
pub enum SlotsConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for SlotsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => write!(f, "Slots config not found at {}: {}", path, err),
            Self::ParseError(err) => write!(f, "Failed to parse slots config: {}", err),
        }
    }
}

impl std::error::Error for SlotsConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_type_deserialization() {
        let yaml = r#"
slots:
  test_string:
    type: string
    description: "A test string slot"
  test_number:
    type: number
    min: 0
    max: 100
  test_enum:
    type: enum
    values:
      - id: option1
        display: "Option 1"
        patterns: ["one", "first"]
"#;
        let config: SlotsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.slots.len(), 3);
        assert_eq!(config.slots["test_string"].slot_type, SlotType::String);
        assert_eq!(config.slots["test_number"].slot_type, SlotType::Number);
        assert_eq!(config.slots["test_enum"].slot_type, SlotType::Enum);
    }

    #[test]
    fn test_goal_deserialization() {
        let yaml = r#"
goals:
  test_goal:
    description: "A test goal"
    required_slots:
      - slot1
      - slot2
    optional_slots:
      - slot3
    completion_action: test_action
"#;
        let config: SlotsConfig = serde_yaml::from_str(yaml).unwrap();
        let goal = config.get_goal("test_goal").unwrap();
        assert_eq!(goal.required_slots, vec!["slot1", "slot2"]);
        assert_eq!(goal.optional_slots, vec!["slot3"]);
        assert_eq!(goal.completion_action, Some("test_action".to_string()));
    }

    #[test]
    fn test_intent_mapping() {
        let yaml = r#"
intent_mapping:
  goal_a:
    - intent1
    - intent2
  goal_b:
    - intent3
"#;
        let config: SlotsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.goal_for_intent("intent1"), Some("goal_a"));
        assert_eq!(config.goal_for_intent("intent2"), Some("goal_a"));
        assert_eq!(config.goal_for_intent("intent3"), Some("goal_b"));
        assert_eq!(config.goal_for_intent("unknown"), None);
    }

    #[test]
    fn test_unit_conversion() {
        let yaml = r#"
slots:
  weight:
    type: number
    unit_conversions:
      tola: 11.66
      oz: 31.1
"#;
        let config: SlotsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.unit_conversion("weight", "tola"), Some(11.66));
        assert_eq!(config.unit_conversion("weight", "oz"), Some(31.1));
        assert_eq!(config.unit_conversion("weight", "unknown"), None);
    }
}
