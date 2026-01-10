//! Tool Argument Provider trait for config-driven tool defaults
//!
//! This module provides a domain-agnostic interface for managing tool arguments,
//! defaults, and intent-to-tool mappings. All definitions are loaded from
//! configuration (intent_tool_mappings.yaml, tools/schemas.yaml).
//!
//! # P20 FIX: Replaces hardcoded fallbacks in agent/tools.rs
//!
//! The previous implementation had hardcoded fallback mappings like:
//! ```ignore
//! match intent.intent.as_str() {
//!     "eligibility_check" => Some("check_eligibility"),
//!     // ...hardcoded
//! }
//! ```
//!
//! And hardcoded tool defaults like:
//! ```ignore
//! if name == "check_eligibility" {
//!     args.insert("collateral_variant", defaults.default_gold_purity);
//! }
//! ```
//!
//! This trait enables fully config-driven tool resolution and defaults.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::ToolArgumentProvider;
//!
//! // Provider is created from domain config
//! let provider = config_bridge.tool_argument_provider();
//!
//! // Resolve tool from intent
//! let tool = provider.resolve_tool_for_intent("eligibility_check", &["asset_quantity"]);
//!
//! // Get defaults for a tool
//! let defaults = provider.get_tool_defaults("check_eligibility");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for providing tool arguments from config
///
/// This trait replaces hardcoded intent-to-tool mappings and tool defaults.
/// All mappings come from configuration files.
pub trait ToolArgumentProvider: Send + Sync {
    /// Resolve tool name from intent
    ///
    /// # Arguments
    /// * `intent` - The detected intent (e.g., "eligibility_check")
    /// * `available_slots` - Slots that are available for this call
    ///
    /// # Returns
    /// Tool name if a mapping exists and required slots are present
    fn resolve_tool_for_intent(&self, intent: &str, available_slots: &[&str]) -> Option<String>;

    /// Get default argument values for a tool
    ///
    /// These defaults are applied when the argument is not provided by the user.
    fn get_tool_defaults(&self, tool_name: &str) -> HashMap<String, serde_json::Value>;

    /// Get argument name mappings for a tool
    ///
    /// Maps slot names to tool argument names.
    /// E.g., "asset_quantity" -> "collateral_weight"
    fn get_argument_mapping(&self, tool_name: &str) -> HashMap<String, String>;

    /// Get required slots for a tool
    ///
    /// These slots must be present for the tool to be called.
    fn get_required_slots(&self, tool_name: &str) -> Vec<String>;

    /// Get optional slots for a tool
    fn get_optional_slots(&self, tool_name: &str) -> Vec<String>;

    /// Check if intent has a tool mapping
    fn has_tool_mapping(&self, intent: &str) -> bool;

    /// Get all intent-to-tool mappings
    fn all_intent_mappings(&self) -> HashMap<String, String>;

    /// Validate tool arguments against schema
    fn validate_arguments(
        &self,
        tool_name: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ArgumentValidationError>;
}

/// Argument validation error
#[derive(Debug, Clone)]
pub struct ArgumentValidationError {
    pub tool_name: String,
    pub errors: Vec<String>,
}

impl std::fmt::Display for ArgumentValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Validation errors for tool '{}': {}",
            self.tool_name,
            self.errors.join(", ")
        )
    }
}

impl std::error::Error for ArgumentValidationError {}

/// Intent to tool mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentToolMapping {
    /// Intent name or pattern
    pub intent: String,
    /// Aliases for the intent
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Tool to invoke
    pub tool: String,
    /// Required slots (all must be present)
    #[serde(default)]
    pub required_slots: Vec<String>,
    /// Optional slots (enhance the call if present)
    #[serde(default)]
    pub optional_slots: Vec<String>,
    /// Whether this mapping is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// Tool defaults configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefaults {
    /// Tool name
    pub tool: String,
    /// Default argument values
    #[serde(default)]
    pub defaults: HashMap<String, serde_json::Value>,
    /// Argument name mappings (slot_name -> arg_name)
    #[serde(default)]
    pub argument_mapping: HashMap<String, String>,
}

/// Config-driven tool argument provider
#[derive(Debug, Clone)]
pub struct ConfigToolArgumentProvider {
    /// Intent to tool mappings (intent -> mapping)
    intent_mappings: HashMap<String, IntentToolMapping>,
    /// Intent aliases (alias -> canonical intent)
    intent_aliases: HashMap<String, String>,
    /// Tool defaults (tool_name -> defaults)
    tool_defaults: HashMap<String, ToolDefaults>,
}

impl ConfigToolArgumentProvider {
    /// Create from config structures
    pub fn new(
        mappings: Vec<IntentToolMapping>,
        defaults: Vec<ToolDefaults>,
    ) -> Self {
        let mut intent_mappings = HashMap::new();
        let mut intent_aliases = HashMap::new();

        for mapping in mappings {
            if mapping.enabled {
                // Register aliases
                for alias in &mapping.aliases {
                    intent_aliases.insert(alias.clone(), mapping.intent.clone());
                }
                intent_mappings.insert(mapping.intent.clone(), mapping);
            }
        }

        let tool_defaults = defaults
            .into_iter()
            .map(|d| (d.tool.clone(), d))
            .collect();

        Self {
            intent_mappings,
            intent_aliases,
            tool_defaults,
        }
    }

    /// Resolve intent alias to canonical name
    fn resolve_intent<'a>(&'a self, intent: &'a str) -> &'a str {
        self.intent_aliases
            .get(intent)
            .map(|s| s.as_str())
            .unwrap_or(intent)
    }
}

impl ToolArgumentProvider for ConfigToolArgumentProvider {
    fn resolve_tool_for_intent(&self, intent: &str, available_slots: &[&str]) -> Option<String> {
        let canonical = self.resolve_intent(intent);

        self.intent_mappings.get(canonical).and_then(|mapping| {
            // Check if all required slots are present
            let has_required = mapping.required_slots.iter().all(|req| {
                available_slots.iter().any(|s| s == req)
            });

            if has_required || mapping.required_slots.is_empty() {
                Some(mapping.tool.clone())
            } else {
                None
            }
        })
    }

    fn get_tool_defaults(&self, tool_name: &str) -> HashMap<String, serde_json::Value> {
        self.tool_defaults
            .get(tool_name)
            .map(|d| d.defaults.clone())
            .unwrap_or_default()
    }

    fn get_argument_mapping(&self, tool_name: &str) -> HashMap<String, String> {
        self.tool_defaults
            .get(tool_name)
            .map(|d| d.argument_mapping.clone())
            .unwrap_or_default()
    }

    fn get_required_slots(&self, tool_name: &str) -> Vec<String> {
        // Find mapping that uses this tool
        self.intent_mappings
            .values()
            .find(|m| m.tool == tool_name)
            .map(|m| m.required_slots.clone())
            .unwrap_or_default()
    }

    fn get_optional_slots(&self, tool_name: &str) -> Vec<String> {
        self.intent_mappings
            .values()
            .find(|m| m.tool == tool_name)
            .map(|m| m.optional_slots.clone())
            .unwrap_or_default()
    }

    fn has_tool_mapping(&self, intent: &str) -> bool {
        let canonical = self.resolve_intent(intent);
        self.intent_mappings.contains_key(canonical)
    }

    fn all_intent_mappings(&self) -> HashMap<String, String> {
        self.intent_mappings
            .iter()
            .map(|(k, v)| (k.clone(), v.tool.clone()))
            .collect()
    }

    fn validate_arguments(
        &self,
        tool_name: &str,
        args: &HashMap<String, serde_json::Value>,
    ) -> Result<(), ArgumentValidationError> {
        let required = self.get_required_slots(tool_name);
        let mut errors = Vec::new();

        for req in required {
            if !args.contains_key(&req) {
                errors.push(format!("Missing required argument: {}", req));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ArgumentValidationError {
                tool_name: tool_name.to_string(),
                errors,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_tool_for_intent() {
        let mappings = vec![
            IntentToolMapping {
                intent: "eligibility_check".to_string(),
                aliases: vec!["check_eligibility".to_string()],
                tool: "check_eligibility".to_string(),
                required_slots: vec!["asset_quantity".to_string()],
                optional_slots: vec!["asset_quality".to_string()],
                enabled: true,
            },
            IntentToolMapping {
                intent: "switch_lender".to_string(),
                aliases: vec!["balance_transfer".to_string()],
                tool: "calculate_savings".to_string(),
                required_slots: vec!["current_provider".to_string()],
                optional_slots: vec![],
                enabled: true,
            },
        ];

        let defaults = vec![
            ToolDefaults {
                tool: "check_eligibility".to_string(),
                defaults: {
                    let mut m = HashMap::new();
                    m.insert("collateral_variant".to_string(), serde_json::json!("tier_2"));
                    m
                },
                argument_mapping: {
                    let mut m = HashMap::new();
                    m.insert("asset_quantity".to_string(), "collateral_weight".to_string());
                    m
                },
            },
        ];

        let provider = ConfigToolArgumentProvider::new(mappings, defaults);

        // Test tool resolution with required slot
        let tool = provider.resolve_tool_for_intent("eligibility_check", &["asset_quantity"]);
        assert_eq!(tool, Some("check_eligibility".to_string()));

        // Test tool resolution without required slot
        let tool = provider.resolve_tool_for_intent("eligibility_check", &[]);
        assert_eq!(tool, None);

        // Test alias resolution
        let tool = provider.resolve_tool_for_intent("balance_transfer", &["current_provider"]);
        assert_eq!(tool, Some("calculate_savings".to_string()));

        // Test defaults
        let defaults = provider.get_tool_defaults("check_eligibility");
        assert_eq!(defaults.get("collateral_variant"), Some(&serde_json::json!("tier_2")));

        // Test argument mapping
        let mapping = provider.get_argument_mapping("check_eligibility");
        assert_eq!(mapping.get("asset_quantity"), Some(&"collateral_weight".to_string()));
    }
}
