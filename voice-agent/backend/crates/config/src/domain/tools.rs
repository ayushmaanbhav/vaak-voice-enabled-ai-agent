//! Tool Schema Configuration
//!
//! Defines config-driven tool schemas for LLM function calling.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::path::Path;

/// Tools configuration loaded from tools/schemas.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsConfig {
    /// Tool definitions keyed by tool name
    #[serde(default)]
    pub tools: HashMap<String, ToolSchema>,
    /// Usage guidelines for the LLM
    #[serde(default)]
    pub usage_guidelines: HashMap<String, String>,
}

impl Default for ToolsConfig {
    fn default() -> Self {
        Self {
            tools: HashMap::new(),
            usage_guidelines: HashMap::new(),
        }
    }
}

impl ToolsConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ToolsConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ToolsConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content).map_err(|e| ToolsConfigError::ParseError(e.to_string()))
    }

    /// Get a tool schema by name
    pub fn get_tool(&self, name: &str) -> Option<&ToolSchema> {
        self.tools.get(name)
    }

    /// Get all tool names
    pub fn tool_names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// Get usage guideline by key
    pub fn get_guideline(&self, key: &str) -> Option<&str> {
        self.usage_guidelines.get(key).map(|s| s.as_str())
    }

    /// Convert all tools to JSON Schema format for LLM consumption
    pub fn to_json_schemas(&self) -> Vec<JsonValue> {
        self.tools.values().map(|t| t.to_json_schema()).collect()
    }

    /// Get tool names that are enabled
    pub fn enabled_tool_names(&self) -> Vec<&str> {
        self.tools
            .iter()
            .filter(|(_, t)| t.enabled.unwrap_or(true))
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

/// Schema definition for a single tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    /// Tool name (identifier)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Tool parameters
    #[serde(default)]
    pub parameters: Vec<ToolParameter>,
    /// Whether the tool is enabled (default: true)
    #[serde(default)]
    pub enabled: Option<bool>,
}

impl ToolSchema {
    /// Convert to JSON Schema format compatible with LLM tool_use
    pub fn to_json_schema(&self) -> JsonValue {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &self.parameters {
            let mut prop = serde_json::Map::new();
            prop.insert(
                "type".to_string(),
                JsonValue::String(param.param_type.clone()),
            );
            prop.insert(
                "description".to_string(),
                JsonValue::String(param.description.clone()),
            );

            // Add enum constraint if present
            if let Some(enum_values) = &param.enum_values {
                let values: Vec<JsonValue> = enum_values
                    .iter()
                    .map(|v| JsonValue::String(v.clone()))
                    .collect();
                prop.insert("enum".to_string(), JsonValue::Array(values));
            }

            // Add min/max constraints for numbers
            if let Some(min) = param.min {
                prop.insert("minimum".to_string(), serde_json::json!(min));
            }
            if let Some(max) = param.max {
                prop.insert("maximum".to_string(), serde_json::json!(max));
            }

            properties.insert(param.name.clone(), JsonValue::Object(prop));

            if param.required {
                required.push(JsonValue::String(param.name.clone()));
            }
        }

        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "parameters": {
                "type": "object",
                "properties": properties,
                "required": required,
            }
        })
    }
}

/// Parameter definition for a tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    /// Parameter name
    pub name: String,
    /// Parameter type (string, number, integer, boolean, array, object)
    #[serde(rename = "type")]
    pub param_type: String,
    /// Human-readable description
    pub description: String,
    /// Whether the parameter is required
    #[serde(default)]
    pub required: bool,
    /// Enum values for string parameters
    #[serde(rename = "enum", default)]
    pub enum_values: Option<Vec<String>>,
    /// Minimum value for number parameters
    #[serde(default)]
    pub min: Option<f64>,
    /// Maximum value for number parameters
    #[serde(default)]
    pub max: Option<f64>,
    /// Default value
    #[serde(default)]
    pub default: Option<String>,
}

/// Errors when loading tools configuration
#[derive(Debug)]
pub enum ToolsConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for ToolsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Tools config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse tools config: {}", err),
        }
    }
}

impl std::error::Error for ToolsConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_schema_deserialization() {
        let yaml = r#"
tools:
  check_eligibility:
    name: check_eligibility
    description: "Check loan eligibility"
    parameters:
      - name: gold_weight
        type: number
        description: "Weight in grams"
        required: true
        min: 1.0
        max: 10000.0
      - name: gold_purity
        type: string
        description: "Purity level"
        required: false
        enum: ["24K", "22K", "18K"]
"#;
        let config: ToolsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.tools.len(), 1);

        let tool = config.get_tool("check_eligibility").unwrap();
        assert_eq!(tool.name, "check_eligibility");
        assert_eq!(tool.parameters.len(), 2);
        assert!(tool.parameters[0].required);
        assert!(!tool.parameters[1].required);
    }

    #[test]
    fn test_to_json_schema() {
        let tool = ToolSchema {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            enabled: None,
            parameters: vec![
                ToolParameter {
                    name: "required_param".to_string(),
                    param_type: "string".to_string(),
                    description: "A required param".to_string(),
                    required: true,
                    enum_values: Some(vec!["a".to_string(), "b".to_string()]),
                    min: None,
                    max: None,
                    default: None,
                },
                ToolParameter {
                    name: "optional_param".to_string(),
                    param_type: "number".to_string(),
                    description: "An optional param".to_string(),
                    required: false,
                    enum_values: None,
                    min: Some(0.0),
                    max: Some(100.0),
                    default: None,
                },
            ],
        };

        let schema = tool.to_json_schema();
        assert_eq!(schema["name"], "test_tool");
        assert!(schema["parameters"]["properties"]["required_param"]["enum"].is_array());
        assert_eq!(schema["parameters"]["properties"]["optional_param"]["minimum"], 0.0);
        assert_eq!(schema["parameters"]["required"][0], "required_param");
    }

    #[test]
    fn test_usage_guidelines() {
        let yaml = r#"
tools: {}
usage_guidelines:
  general: "Use tools wisely"
  eligibility: "Check when customer asks"
"#;
        let config: ToolsConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.get_guideline("general"), Some("Use tools wisely"));
        assert_eq!(
            config.get_guideline("eligibility"),
            Some("Check when customer asks")
        );
        assert_eq!(config.get_guideline("unknown"), None);
    }
}
