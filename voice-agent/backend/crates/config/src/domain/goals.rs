//! Goals Configuration
//!
//! Defines config-driven conversation goals for the dialog state tracker.
//! Goals specify required slots, completion tools, and intent mappings.
//!
//! Also provides action instruction templates for domain-agnostic agent behavior.
//! Templates use placeholders: {tool_name}, {slot_display}, {brand.bank_name}, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Action instruction template with multilingual support
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionTemplate {
    /// English template
    #[serde(default)]
    pub en: String,
    /// Hindi template
    #[serde(default)]
    pub hi: String,
}

impl ActionTemplate {
    /// Get template for a language with fallback to English
    pub fn get(&self, language: &str) -> &str {
        match language {
            "hi" if !self.hi.is_empty() => &self.hi,
            _ => &self.en,
        }
    }

    /// Render template with context substitutions
    pub fn render(&self, language: &str, context: &ActionContext) -> String {
        let template = self.get(language);
        context.substitute(template)
    }
}

/// Action templates configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionTemplatesConfig {
    /// Template for "call tool" action
    #[serde(default)]
    pub call_tool: ActionTemplate,
    /// Template for "ask for slot" action
    #[serde(default)]
    pub ask_for: ActionTemplate,
    /// Template for "offer appointment" action
    #[serde(default)]
    pub offer_appointment: ActionTemplate,
    /// Template for "explain process" action
    #[serde(default)]
    pub explain_process: ActionTemplate,
    /// Template for "discover intent" action
    #[serde(default)]
    pub discover_intent: ActionTemplate,
    /// Template for "capture lead" action
    #[serde(default)]
    pub capture_lead: ActionTemplate,
}

impl ActionTemplatesConfig {
    /// Get a template by action type name
    pub fn get_template(&self, action_type: &str) -> Option<&ActionTemplate> {
        match action_type {
            "call_tool" => Some(&self.call_tool),
            "ask_for" => Some(&self.ask_for),
            "offer_appointment" => Some(&self.offer_appointment),
            "explain_process" => Some(&self.explain_process),
            "discover_intent" => Some(&self.discover_intent),
            "capture_lead" => Some(&self.capture_lead),
            _ => None,
        }
    }
}

/// Context for rendering action templates
///
/// Provides substitution values for template placeholders like:
/// - {tool_name} - current tool name
/// - {slot_display} - human-readable slot name
/// - {brand.company_name} - brand's company/organization name
/// - {brand.product_name} - brand's product name
/// - {brand.agent_name} - brand's agent name
///
/// P16 FIX: Renamed bank_name to company_name for domain-agnostic design.
#[derive(Debug, Clone, Default)]
pub struct ActionContext {
    /// Current tool name (if applicable)
    pub tool_name: Option<String>,
    /// Current slot ID (if applicable)
    pub slot_id: Option<String>,
    /// Human-readable slot display name
    pub slot_display: Option<String>,
    /// Brand company/organization name
    pub company_name: String,
    /// Brand product name (e.g., "Gold Loan", "Insurance")
    pub product_name: String,
    /// Brand agent name
    pub agent_name: String,
}

impl ActionContext {
    /// Create a new action context with brand info
    pub fn new(company_name: &str, product_name: &str, agent_name: &str) -> Self {
        Self {
            tool_name: None,
            slot_id: None,
            slot_display: None,
            company_name: company_name.to_string(),
            product_name: product_name.to_string(),
            agent_name: agent_name.to_string(),
        }
    }

    /// Set the tool name for this context
    pub fn with_tool(mut self, tool_name: &str) -> Self {
        self.tool_name = Some(tool_name.to_string());
        self
    }

    /// Set the slot for this context
    pub fn with_slot(mut self, slot_id: &str, slot_display: &str) -> Self {
        self.slot_id = Some(slot_id.to_string());
        self.slot_display = Some(slot_display.to_string());
        self
    }

    /// Substitute placeholders in a template string
    pub fn substitute(&self, template: &str) -> String {
        let mut result = template.to_string();

        // Tool name
        if let Some(ref tool) = self.tool_name {
            result = result.replace("{tool_name}", tool);
        }

        // Slot info
        if let Some(ref slot) = self.slot_display {
            result = result.replace("{slot_display}", slot);
        }
        if let Some(ref slot_id) = self.slot_id {
            result = result.replace("{slot_id}", slot_id);
        }

        // Brand info - support both new and legacy placeholders
        result = result.replace("{brand.company_name}", &self.company_name);
        result = result.replace("{brand.bank_name}", &self.company_name); // Legacy
        result = result.replace("{brand.product_name}", &self.product_name);
        result = result.replace("{brand.agent_name}", &self.agent_name);

        result
    }
}

/// Goals configuration loaded from goals.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalsConfig {
    /// Action instruction templates (domain-agnostic)
    #[serde(default)]
    pub action_templates: ActionTemplatesConfig,
    /// Goal definitions keyed by goal ID
    #[serde(default)]
    pub goals: HashMap<String, GoalEntry>,
    /// Intent to goal mappings
    #[serde(default)]
    pub intent_mappings: HashMap<String, String>,
    /// Default goal when no intent is detected
    #[serde(default = "default_goal")]
    pub default_goal: String,
}

fn default_goal() -> String {
    "exploration".to_string()
}

impl Default for GoalsConfig {
    fn default() -> Self {
        Self {
            action_templates: ActionTemplatesConfig::default(),
            goals: HashMap::new(),
            intent_mappings: HashMap::new(),
            default_goal: default_goal(),
        }
    }
}

impl GoalsConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, GoalsConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            GoalsConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content).map_err(|e| GoalsConfigError::ParseError(e.to_string()))
    }

    /// Get a goal entry by ID
    pub fn get_goal(&self, id: &str) -> Option<&GoalEntry> {
        self.goals.get(id)
    }

    /// Get all goal IDs
    pub fn goal_ids(&self) -> Vec<&str> {
        self.goals.keys().map(|s| s.as_str()).collect()
    }

    /// Map an intent to a goal ID
    pub fn goal_for_intent(&self, intent: &str) -> Option<&str> {
        self.intent_mappings.get(intent).map(|s| s.as_str())
    }

    /// Get the default goal ID
    pub fn default_goal(&self) -> &str {
        &self.default_goal
    }

    /// Get slot prompt for a specific language
    pub fn slot_prompt(&self, goal_id: &str, slot: &str, language: &str) -> Option<&str> {
        self.goals
            .get(goal_id)?
            .slot_prompts
            .as_ref()?
            .get(slot)?
            .get(language)
            .map(|s| s.as_str())
    }
}

/// Single goal entry from configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalEntry {
    /// Human-readable display name
    pub display_name: String,
    /// Goal description
    #[serde(default)]
    pub description: String,
    /// Priority level (lower = higher priority)
    #[serde(default = "default_priority")]
    pub priority: u8,
    /// Required slots that must be filled
    #[serde(default)]
    pub required_slots: Vec<String>,
    /// Optional slots for enhanced service
    #[serde(default)]
    pub optional_slots: Vec<String>,
    /// Tool to call when goal is complete
    pub completion_tool: Option<String>,
    /// Slot prompt templates keyed by slot name, then by language
    #[serde(default)]
    pub slot_prompts: Option<HashMap<String, HashMap<String, String>>>,
}

fn default_priority() -> u8 {
    50
}

impl Default for GoalEntry {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            description: String::new(),
            priority: default_priority(),
            required_slots: Vec::new(),
            optional_slots: Vec::new(),
            completion_tool: None,
            slot_prompts: None,
        }
    }
}

impl GoalEntry {
    /// Get slot prompt for a language, with fallback to English
    pub fn get_slot_prompt(&self, slot: &str, language: &str) -> Option<&str> {
        let prompts = self.slot_prompts.as_ref()?;
        let slot_prompts = prompts.get(slot)?;

        // Try requested language first, fallback to English
        slot_prompts
            .get(language)
            .or_else(|| slot_prompts.get("en"))
            .map(|s| s.as_str())
    }

    /// Check if goal is complete given filled slots
    pub fn is_complete(&self, filled_slots: &HashMap<String, String>) -> bool {
        self.required_slots
            .iter()
            .all(|slot| filled_slots.contains_key(slot))
    }

    /// Get missing required slots
    pub fn missing_slots<'a>(&'a self, filled_slots: &HashMap<String, String>) -> Vec<&'a str> {
        self.required_slots
            .iter()
            .filter(|slot| !filled_slots.contains_key(*slot))
            .map(|s| s.as_str())
            .collect()
    }

    /// Get completion percentage
    pub fn completion_percentage(&self, filled_slots: &HashMap<String, String>) -> f32 {
        if self.required_slots.is_empty() {
            return 100.0;
        }

        let filled_count = self
            .required_slots
            .iter()
            .filter(|slot| filled_slots.contains_key(*slot))
            .count();

        (filled_count as f32 / self.required_slots.len() as f32) * 100.0
    }
}

/// Errors when loading goals configuration
#[derive(Debug)]
pub enum GoalsConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for GoalsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Goals config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse goals config: {}", err),
        }
    }
}

impl std::error::Error for GoalsConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goals_config_deserialization() {
        let yaml = r#"
goals:
  exploration:
    display_name: "Exploration"
    description: "General exploration"
    priority: 100
    required_slots: []
    optional_slots: []

  balance_transfer:
    display_name: "Balance Transfer"
    description: "Transfer loan from competitor"
    priority: 20
    required_slots:
      - current_lender
      - loan_amount
    optional_slots:
      - current_interest_rate
    completion_tool: calculate_savings
    slot_prompts:
      current_lender:
        en: "Which lender do you currently have your loan with?"
        hi: "आपका मौजूदा लोन किस बैंक से है?"

intent_mappings:
  balance_transfer: balance_transfer
  switch_lender: balance_transfer
  new_loan: new_loan

default_goal: exploration
"#;

        let config: GoalsConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.goals.len(), 2);
        assert_eq!(config.default_goal(), "exploration");

        let bt = config.get_goal("balance_transfer").unwrap();
        assert_eq!(bt.display_name, "Balance Transfer");
        assert_eq!(bt.priority, 20);
        assert_eq!(bt.required_slots, vec!["current_lender", "loan_amount"]);
        assert_eq!(bt.completion_tool, Some("calculate_savings".to_string()));

        // Test slot prompts
        assert_eq!(
            bt.get_slot_prompt("current_lender", "en"),
            Some("Which lender do you currently have your loan with?")
        );
        assert_eq!(
            bt.get_slot_prompt("current_lender", "hi"),
            Some("आपका मौजूदा लोन किस बैंक से है?")
        );
        // Fallback to English
        assert_eq!(
            bt.get_slot_prompt("current_lender", "ta"),
            Some("Which lender do you currently have your loan with?")
        );

        // Test intent mapping
        assert_eq!(config.goal_for_intent("switch_lender"), Some("balance_transfer"));
        assert_eq!(config.goal_for_intent("new_loan"), Some("new_loan"));
    }

    #[test]
    fn test_goal_completion() {
        let goal = GoalEntry {
            display_name: "Test".to_string(),
            description: "Test goal".to_string(),
            priority: 50,
            required_slots: vec!["slot1".to_string(), "slot2".to_string()],
            optional_slots: vec![],
            completion_tool: None,
            slot_prompts: None,
        };

        let mut slots = HashMap::new();
        assert!(!goal.is_complete(&slots));
        assert_eq!(goal.completion_percentage(&slots), 0.0);

        slots.insert("slot1".to_string(), "value1".to_string());
        assert!(!goal.is_complete(&slots));
        assert_eq!(goal.completion_percentage(&slots), 50.0);

        slots.insert("slot2".to_string(), "value2".to_string());
        assert!(goal.is_complete(&slots));
        assert_eq!(goal.completion_percentage(&slots), 100.0);
    }

    #[test]
    fn test_missing_slots() {
        let goal = GoalEntry {
            display_name: "Test".to_string(),
            description: "Test goal".to_string(),
            priority: 50,
            required_slots: vec!["a".to_string(), "b".to_string(), "c".to_string()],
            optional_slots: vec![],
            completion_tool: None,
            slot_prompts: None,
        };

        let mut slots = HashMap::new();
        slots.insert("b".to_string(), "value".to_string());

        let missing = goal.missing_slots(&slots);
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"a"));
        assert!(missing.contains(&"c"));
    }

    #[test]
    fn test_default_values() {
        let yaml = r#"
goals:
  simple:
    display_name: "Simple Goal"
"#;
        let config: GoalsConfig = serde_yaml::from_str(yaml).unwrap();
        let goal = config.get_goal("simple").unwrap();

        assert_eq!(goal.priority, 50); // default
        assert!(goal.required_slots.is_empty());
        assert!(goal.optional_slots.is_empty());
        assert!(goal.completion_tool.is_none());
    }

    // ====== Action Templates Tests ======

    #[test]
    fn test_action_template_language_fallback() {
        let template = ActionTemplate {
            en: "Hello {brand.bank_name}".to_string(),
            hi: "नमस्ते {brand.bank_name}".to_string(),
        };

        assert_eq!(template.get("en"), "Hello {brand.bank_name}");
        assert_eq!(template.get("hi"), "नमस्ते {brand.bank_name}");
        // Fallback to English for unknown language
        assert_eq!(template.get("fr"), "Hello {brand.bank_name}");
    }

    #[test]
    fn test_action_template_empty_fallback() {
        let template = ActionTemplate {
            en: "English only".to_string(),
            hi: String::new(), // Empty Hindi
        };

        // Should fallback to English when Hindi is empty
        assert_eq!(template.get("hi"), "English only");
    }

    #[test]
    fn test_action_context_substitution() {
        let context = ActionContext::new("Test Bank", "Test Product", "Agent Name");

        let result = context.substitute("Welcome to {brand.bank_name} {brand.product_name}");
        assert_eq!(result, "Welcome to Test Bank Test Product");

        // With agent name
        let result = context.substitute("I am {brand.agent_name}");
        assert_eq!(result, "I am Agent Name");
    }

    #[test]
    fn test_action_context_with_tool() {
        let context = ActionContext::new("Bank", "Product", "Agent")
            .with_tool("calculate_savings");

        let result = context.substitute("CALL the {tool_name} tool");
        assert_eq!(result, "CALL the calculate_savings tool");
    }

    #[test]
    fn test_action_context_with_slot() {
        let context = ActionContext::new("Bank", "Product", "Agent")
            .with_slot("current_lender", "current lender");

        let result = context.substitute("ASK for {slot_display}");
        assert_eq!(result, "ASK for current lender");

        let result = context.substitute("Field {slot_id} is required");
        assert_eq!(result, "Field current_lender is required");
    }

    #[test]
    fn test_action_template_render() {
        let template = ActionTemplate {
            en: "ASK what brings them to {brand.bank_name} {brand.product_name} today".to_string(),
            hi: "पूछें कि आज उन्हें {brand.bank_name} {brand.product_name} में क्या लाया".to_string(),
        };

        let context = ActionContext::new("Kotak Mahindra Bank", "Gold Loan", "Priya");

        let en_result = template.render("en", &context);
        assert!(en_result.contains("Kotak Mahindra Bank"));
        assert!(en_result.contains("Gold Loan"));

        let hi_result = template.render("hi", &context);
        assert!(hi_result.contains("Kotak Mahindra Bank"));
        assert!(hi_result.contains("पूछें"));
    }

    #[test]
    fn test_action_templates_config_get_template() {
        let mut config = ActionTemplatesConfig::default();
        config.call_tool = ActionTemplate {
            en: "Call {tool_name}".to_string(),
            hi: String::new(),
        };

        assert!(config.get_template("call_tool").is_some());
        assert!(config.get_template("ask_for").is_some());
        assert!(config.get_template("unknown_action").is_none());
    }

    #[test]
    fn test_goals_config_with_action_templates() {
        let yaml = r#"
action_templates:
  call_tool:
    en: "CALL the {tool_name} tool now"
    hi: "{tool_name} tool अभी call करें"
  discover_intent:
    en: "ASK what brings them to {brand.bank_name} {brand.product_name} today"
    hi: "पूछें कि आज उन्हें {brand.bank_name} {brand.product_name} में क्या लाया"

goals:
  exploration:
    display_name: "Exploration"

default_goal: exploration
"#;

        let config: GoalsConfig = serde_yaml::from_str(yaml).unwrap();

        // Verify action templates loaded
        assert_eq!(
            config.action_templates.call_tool.en,
            "CALL the {tool_name} tool now"
        );
        assert_eq!(
            config.action_templates.discover_intent.hi,
            "पूछें कि आज उन्हें {brand.bank_name} {brand.product_name} में क्या लाया"
        );
    }
}
