//! Entity Types Configuration
//!
//! P22 FIX: Config-driven entity type definitions for domain-agnostic entity extraction.
//! Maps generic entity types to domain-specific display names and categories.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Single entity type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTypeDefinition {
    /// Display name in English
    pub display_name: String,
    /// Display name in Hindi
    #[serde(default)]
    pub display_name_hi: String,
    /// Category (Asset, Financial, Provider, Customer)
    pub category: String,
    /// Unit for quantity entities (e.g., "grams", "percent")
    #[serde(default)]
    pub unit: Option<String>,
    /// Unit in Hindi
    #[serde(default)]
    pub unit_hi: Option<String>,
    /// Currency code for monetary entities
    #[serde(default)]
    pub currency: Option<String>,
    /// Description of the entity type
    #[serde(default)]
    pub description: String,
    /// Aliases/alternative names for this entity type
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// P23 FIX: Competitor type definition from entity_types.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitorTypeDefinition {
    /// Display name
    pub display_name: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Default values for this competitor type
    #[serde(default)]
    pub default_values: CompetitorTypeDefaults,
    /// Aliases for this type
    #[serde(default)]
    pub aliases: Vec<String>,
}

/// Default values for a competitor type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompetitorTypeDefaults {
    /// Default interest rate for this type
    #[serde(default)]
    pub rate: f64,
    /// Default processing fee percentage
    #[serde(default)]
    pub processing_fee: f64,
    /// Whether this type typically has prepayment penalty
    #[serde(default)]
    pub prepayment_penalty: bool,
    /// Typical LTV ratio
    #[serde(default)]
    pub typical_ltv: f64,
}

impl EntityTypeDefinition {
    /// Get display name for a given language
    pub fn display_name_for_language(&self, language: &str) -> &str {
        if language == "hi" && !self.display_name_hi.is_empty() {
            &self.display_name_hi
        } else {
            &self.display_name
        }
    }

    /// Get unit for a given language
    pub fn unit_for_language(&self, language: &str) -> Option<&str> {
        if language == "hi" {
            self.unit_hi.as_deref().or(self.unit.as_deref())
        } else {
            self.unit.as_deref()
        }
    }
}

/// Entity category definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCategory {
    /// Description of category
    pub description: String,
    /// Icon identifier for UI
    #[serde(default)]
    pub icon: String,
}

/// Entities configuration loaded from entities.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesConfig {
    /// Entity type definitions (canonical_name -> definition)
    #[serde(default)]
    pub entity_types: HashMap<String, EntityTypeDefinition>,
    /// Category definitions
    #[serde(default)]
    pub categories: HashMap<String, EntityCategory>,
    /// Extraction priority order (higher priority = extracted first)
    #[serde(default)]
    pub extraction_priority: Vec<String>,
    /// Display format templates by entity type
    #[serde(default)]
    pub display_formats: HashMap<String, String>,
    /// P23 FIX: Competitor type definitions with default rates
    #[serde(default)]
    pub competitor_types: HashMap<String, CompetitorTypeDefinition>,
}

impl Default for EntitiesConfig {
    fn default() -> Self {
        Self {
            entity_types: HashMap::new(),
            categories: HashMap::new(),
            extraction_priority: Vec::new(),
            display_formats: HashMap::new(),
            competitor_types: HashMap::new(),
        }
    }
}

impl EntitiesConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, EntitiesConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            EntitiesConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content).map_err(|e| EntitiesConfigError::ParseError(e.to_string()))
    }

    /// Get entity type definition by canonical name
    pub fn get_entity_type(&self, name: &str) -> Option<&EntityTypeDefinition> {
        self.entity_types.get(name)
    }

    /// Get display name for an entity type
    pub fn display_name(&self, entity_type: &str, language: &str) -> Option<&str> {
        self.entity_types
            .get(entity_type)
            .map(|e| e.display_name_for_language(language))
    }

    /// Get unit for an entity type
    pub fn unit(&self, entity_type: &str, language: &str) -> Option<&str> {
        self.entity_types
            .get(entity_type)
            .and_then(|e| e.unit_for_language(language))
    }

    /// Get currency for an entity type
    pub fn currency(&self, entity_type: &str) -> Option<&str> {
        self.entity_types
            .get(entity_type)
            .and_then(|e| e.currency.as_deref())
    }

    /// Get category for an entity type
    pub fn category(&self, entity_type: &str) -> Option<&str> {
        self.entity_types.get(entity_type).map(|e| e.category.as_str())
    }

    /// Get all entities in a category
    pub fn entities_in_category(&self, category: &str) -> Vec<&str> {
        self.entity_types
            .iter()
            .filter(|(_, def)| def.category == category)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    /// Get all category names
    pub fn category_names(&self) -> Vec<&str> {
        self.categories.keys().map(|s| s.as_str()).collect()
    }

    /// Get category description
    pub fn category_description(&self, category: &str) -> Option<&str> {
        self.categories.get(category).map(|c| c.description.as_str())
    }

    /// Get category icon
    pub fn category_icon(&self, category: &str) -> Option<&str> {
        self.categories
            .get(category)
            .map(|c| c.icon.as_str())
            .filter(|s| !s.is_empty())
    }

    /// Get extraction priority order for entity type (lower = higher priority)
    pub fn extraction_order(&self, entity_type: &str) -> usize {
        self.extraction_priority
            .iter()
            .position(|e| e == entity_type)
            .unwrap_or(usize::MAX)
    }

    /// Get entities sorted by extraction priority
    pub fn entities_by_priority(&self) -> Vec<&str> {
        let mut entities: Vec<&str> = self.entity_types.keys().map(|s| s.as_str()).collect();
        entities.sort_by_key(|e| self.extraction_order(e));
        entities
    }

    /// Format entity value using display template
    pub fn format_value(
        &self,
        entity_type: &str,
        value: &str,
        language: &str,
    ) -> String {
        if let Some(template) = self.display_formats.get(entity_type) {
            let unit = self.unit(entity_type, language).unwrap_or("");
            let currency = self.currency(entity_type).unwrap_or("");

            template
                .replace("{value}", value)
                .replace("{unit}", unit)
                .replace("{currency}", currency)
        } else {
            value.to_string()
        }
    }

    /// Resolve alias to canonical entity type name
    pub fn resolve_alias<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        // Check if it's already a canonical name
        if self.entity_types.contains_key(name) {
            return Some(name);
        }
        // Check aliases
        for (canonical, def) in &self.entity_types {
            if def.aliases.iter().any(|a| a == name) {
                return Some(canonical.as_str());
            }
        }
        None
    }

    /// Get all aliases for an entity type
    pub fn aliases_for(&self, entity_type: &str) -> Vec<&str> {
        self.entity_types
            .get(entity_type)
            .map(|e| e.aliases.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Check if an entity type exists (by canonical name or alias)
    pub fn has_entity(&self, name: &str) -> bool {
        self.resolve_alias(name).is_some()
    }

    // =========================================================================
    // P23 FIX: Competitor type methods
    // =========================================================================

    /// Get competitor type definition by ID
    pub fn get_competitor_type(&self, type_id: &str) -> Option<&CompetitorTypeDefinition> {
        self.competitor_types.get(type_id)
    }

    /// Get default rate for a competitor type
    pub fn competitor_default_rate(&self, type_id: &str) -> Option<f64> {
        self.competitor_types.get(type_id).map(|t| t.default_values.rate)
    }

    /// Build a HashMap of type_id -> default_rate for all competitor types
    ///
    /// This is used by ConfigCompetitorAnalyzer for fallback rates.
    pub fn competitor_type_default_rates(&self) -> HashMap<String, f64> {
        self.competitor_types
            .iter()
            .map(|(id, def)| (id.clone(), def.default_values.rate))
            .collect()
    }

    /// Resolve competitor type alias to canonical type ID
    pub fn resolve_competitor_type_alias<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        // Check if it's already a canonical name
        if self.competitor_types.contains_key(name) {
            return Some(name);
        }
        // Check aliases
        for (canonical, def) in &self.competitor_types {
            if def.aliases.iter().any(|a| a == name) {
                return Some(canonical.as_str());
            }
        }
        None
    }

    /// Get all competitor type IDs
    pub fn competitor_type_ids(&self) -> Vec<&str> {
        self.competitor_types.keys().map(|s| s.as_str()).collect()
    }
}

/// Errors when loading entities configuration
#[derive(Debug)]
pub enum EntitiesConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for EntitiesConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Entities config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse entities config: {}", err),
        }
    }
}

impl std::error::Error for EntitiesConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entities_config_deserialization() {
        let yaml = r#"
entity_types:
  asset_quantity:
    display_name: "Gold Weight"
    display_name_hi: "सोने का वजन"
    category: "Asset"
    unit: "grams"
    unit_hi: "ग्राम"
    description: "Quantity/weight of the collateral asset"
    aliases:
      - gold_weight
      - weight

  offer_amount:
    display_name: "Loan Amount"
    display_name_hi: "ऋण राशि"
    category: "Financial"
    currency: "INR"
    description: "Requested or offered loan amount"
    aliases:
      - loan_amount
      - amount

categories:
  Asset:
    description: "Entities related to collateral assets"
    icon: "gold"
  Financial:
    description: "Monetary and rate-related entities"
    icon: "currency"

extraction_priority:
  - offer_amount
  - asset_quantity

display_formats:
  asset_quantity: "{value} {unit}"
  offer_amount: "{currency}{value}"
"#;
        let config: EntitiesConfig = serde_yaml::from_str(yaml).unwrap();

        // Test entity types
        assert_eq!(config.entity_types.len(), 2);

        // Test display name
        assert_eq!(config.display_name("asset_quantity", "en"), Some("Gold Weight"));
        assert_eq!(config.display_name("asset_quantity", "hi"), Some("सोने का वजन"));

        // Test unit
        assert_eq!(config.unit("asset_quantity", "en"), Some("grams"));
        assert_eq!(config.unit("asset_quantity", "hi"), Some("ग्राम"));

        // Test currency
        assert_eq!(config.currency("offer_amount"), Some("INR"));
        assert_eq!(config.currency("asset_quantity"), None);

        // Test categories
        assert_eq!(config.category("asset_quantity"), Some("Asset"));
        assert_eq!(config.category_description("Asset"), Some("Entities related to collateral assets"));

        // Test extraction order
        assert_eq!(config.extraction_order("offer_amount"), 0);
        assert_eq!(config.extraction_order("asset_quantity"), 1);
        assert_eq!(config.extraction_order("unknown"), usize::MAX);

        // Test alias resolution
        assert_eq!(config.resolve_alias("gold_weight"), Some("asset_quantity"));
        assert_eq!(config.resolve_alias("asset_quantity"), Some("asset_quantity"));
        assert_eq!(config.resolve_alias("unknown"), None);

        // Test format value
        assert_eq!(config.format_value("asset_quantity", "100", "en"), "100 grams");
        assert_eq!(config.format_value("offer_amount", "50000", "en"), "INR50000");
    }

    #[test]
    fn test_entities_in_category() {
        let config = EntitiesConfig {
            entity_types: [
                ("entity_a".to_string(), EntityTypeDefinition {
                    display_name: "Entity A".to_string(),
                    display_name_hi: String::new(),
                    category: "Cat1".to_string(),
                    unit: None,
                    unit_hi: None,
                    currency: None,
                    description: String::new(),
                    aliases: vec![],
                }),
                ("entity_b".to_string(), EntityTypeDefinition {
                    display_name: "Entity B".to_string(),
                    display_name_hi: String::new(),
                    category: "Cat1".to_string(),
                    unit: None,
                    unit_hi: None,
                    currency: None,
                    description: String::new(),
                    aliases: vec![],
                }),
                ("entity_c".to_string(), EntityTypeDefinition {
                    display_name: "Entity C".to_string(),
                    display_name_hi: String::new(),
                    category: "Cat2".to_string(),
                    unit: None,
                    unit_hi: None,
                    currency: None,
                    description: String::new(),
                    aliases: vec![],
                }),
            ].into_iter().collect(),
            ..Default::default()
        };

        let cat1_entities = config.entities_in_category("Cat1");
        assert_eq!(cat1_entities.len(), 2);
        assert!(cat1_entities.contains(&"entity_a"));
        assert!(cat1_entities.contains(&"entity_b"));

        let cat2_entities = config.entities_in_category("Cat2");
        assert_eq!(cat2_entities.len(), 1);
        assert!(cat2_entities.contains(&"entity_c"));
    }
}
