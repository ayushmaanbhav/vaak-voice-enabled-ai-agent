//! Signals Configuration
//!
//! P23 FIX: Defines config-driven signal definitions loaded from signals.yaml.
//! These signals are used for lead scoring, qualification, and escalation detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Signals configuration loaded from signals.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalsConfig {
    /// Signal category definitions
    #[serde(default)]
    pub categories: HashMap<String, SignalCategory>,
    /// Signal definitions
    #[serde(default)]
    pub signals: HashMap<String, SignalDefinition>,
    /// Scoring thresholds for lead qualification
    #[serde(default)]
    pub scoring_thresholds: HashMap<String, ScoringThreshold>,
    /// Escalation triggers based on signals
    #[serde(default)]
    pub escalation_triggers: HashMap<String, EscalationTriggerDef>,
}

impl Default for SignalsConfig {
    fn default() -> Self {
        Self {
            categories: HashMap::new(),
            signals: HashMap::new(),
            scoring_thresholds: HashMap::new(),
            escalation_triggers: HashMap::new(),
        }
    }
}

impl SignalsConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, SignalsConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            SignalsConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content).map_err(|e| SignalsConfigError::ParseError(e.to_string()))
    }

    /// Get signal definition by ID
    pub fn get_signal(&self, signal_id: &str) -> Option<&SignalDefinition> {
        self.signals.get(signal_id)
    }

    /// Get all signals in a category
    pub fn signals_in_category(&self, category: &str) -> Vec<(&str, &SignalDefinition)> {
        self.signals
            .iter()
            .filter(|(_, def)| def.category == category)
            .map(|(id, def)| (id.as_str(), def))
            .collect()
    }

    /// Get all signal IDs
    pub fn signal_ids(&self) -> Vec<&str> {
        self.signals.keys().map(|k| k.as_str()).collect()
    }

    /// Get weight for a signal
    pub fn signal_weight(&self, signal_id: &str) -> i32 {
        self.signals.get(signal_id).map(|s| s.weight).unwrap_or(0)
    }

    /// Get max value for a counter signal
    pub fn signal_max(&self, signal_id: &str) -> Option<u32> {
        self.signals.get(signal_id).and_then(|s| s.max)
    }

    /// Get allowed values for a string signal
    pub fn signal_allowed_values(&self, signal_id: &str) -> Option<&[String]> {
        self.signals
            .get(signal_id)
            .and_then(|s| s.allowed_values.as_deref())
    }

    /// Calculate max score for a category
    pub fn category_max_score(&self, category: &str) -> u32 {
        self.categories.get(category).map(|c| c.max_score).unwrap_or(25)
    }

    /// Get category weight multiplier
    pub fn category_weight(&self, category: &str) -> f64 {
        self.categories.get(category).map(|c| c.weight).unwrap_or(1.0)
    }

    /// Get scoring threshold for a level
    pub fn threshold_min_score(&self, level: &str) -> Option<u32> {
        self.scoring_thresholds.get(level).map(|t| t.min_score)
    }

    /// Convert to core SignalDefinition format for SignalStore
    pub fn to_core_definitions(&self) -> Vec<voice_agent_core::traits::SignalDefinition> {
        self.signals
            .iter()
            .map(|(id, def)| voice_agent_core::traits::SignalDefinition {
                id: id.clone(),
                display_name: def.display_name.clone(),
                signal_type: def.to_core_signal_type(),
                category: def.category.clone(),
                weight: def.weight.max(0) as u32,
                description: Some(def.description.clone()),
            })
            .collect()
    }
}

/// Signal category definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalCategory {
    /// Display name for the category
    pub display_name: String,
    /// Maximum score for this category
    #[serde(default = "default_max_score")]
    pub max_score: u32,
    /// Weight multiplier for this category
    #[serde(default = "default_weight")]
    pub weight: f64,
    /// Description
    #[serde(default)]
    pub description: String,
}

fn default_max_score() -> u32 {
    25
}

fn default_weight() -> f64 {
    1.0
}

/// Signal definition from config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalDefinition {
    /// Display name for UI/logging
    pub display_name: String,
    /// Signal type: boolean, counter, string, numeric
    #[serde(rename = "type")]
    pub signal_type: String,
    /// Category this signal belongs to
    pub category: String,
    /// Weight for scoring (can be negative for penalty signals)
    #[serde(default)]
    pub weight: i32,
    /// Max value for counter signals
    #[serde(default)]
    pub max: Option<u32>,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Allowed values for string signals
    #[serde(default)]
    pub allowed_values: Option<Vec<String>>,
    /// Min value for numeric signals
    #[serde(default)]
    pub min: Option<f64>,
}

impl SignalDefinition {
    /// Convert to core SignalType
    pub fn to_core_signal_type(&self) -> voice_agent_core::traits::SignalType {
        match self.signal_type.as_str() {
            "boolean" => voice_agent_core::traits::SignalType::Boolean,
            "counter" => voice_agent_core::traits::SignalType::Counter { max: self.max },
            "string" => voice_agent_core::traits::SignalType::String {
                allowed_values: self.allowed_values.clone(),
            },
            "numeric" => voice_agent_core::traits::SignalType::Numeric {
                min: self.min,
                max: self.max.map(|v| v as f64),
            },
            _ => voice_agent_core::traits::SignalType::Boolean,
        }
    }
}

/// Scoring threshold definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringThreshold {
    /// Minimum score for this threshold level
    pub min_score: u32,
    /// Description of this level
    #[serde(default)]
    pub description: String,
}

/// Escalation trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationTriggerDef {
    /// Signals that trigger this escalation (all must be met)
    #[serde(default)]
    pub signals: Vec<String>,
    /// Action to take when triggered
    #[serde(default)]
    pub action: String,
    /// Description
    #[serde(default)]
    pub description: String,
}

/// Errors when loading signals configuration
#[derive(Debug)]
pub enum SignalsConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for SignalsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Signals config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse signals config: {}", err),
        }
    }
}

impl std::error::Error for SignalsConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signals_deserialization() {
        let yaml = r#"
categories:
  urgency:
    display_name: "Urgency Signals"
    max_score: 25
    weight: 1.0

signals:
  has_urgency:
    display_name: "Urgency Detected"
    type: boolean
    category: urgency
    weight: 10
    description: "Customer mentioned urgency"

  engagement_turns:
    display_name: "Engagement Turns"
    type: counter
    category: engagement
    weight: 3
    max: 5

scoring_thresholds:
  hot:
    min_score: 60
    description: "Hot lead"
"#;
        let config: SignalsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.signals.len(), 2);
        assert!(config.signals.contains_key("has_urgency"));
        assert_eq!(config.signal_weight("has_urgency"), 10);
        assert_eq!(config.signal_max("engagement_turns"), Some(5));
    }

    #[test]
    fn test_category_methods() {
        let yaml = r#"
categories:
  urgency:
    display_name: "Urgency"
    max_score: 30
    weight: 1.5
"#;
        let config: SignalsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.category_max_score("urgency"), 30);
        assert_eq!(config.category_weight("urgency"), 1.5);
        // Default for unknown category
        assert_eq!(config.category_max_score("unknown"), 25);
    }
}
