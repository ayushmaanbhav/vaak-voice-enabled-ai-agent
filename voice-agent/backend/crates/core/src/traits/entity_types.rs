//! Entity Type Provider for Domain-Agnostic Type Definitions
//!
//! P23 FIX: This module provides config-driven entity types that replace
//! hardcoded enums like CompetitorType and CustomerSegment.
//!
//! Instead of domain-specific enums like:
//! ```ignore
//! pub enum CompetitorType { Bank, Nbfc, Informal }
//! pub enum CustomerSegment { HighValue, TrustSeeker, FirstTime }
//! ```
//!
//! Entity types are now defined in config (entity_types.yaml) and accessed generically.
//!
//! # Example Config (entity_types.yaml)
//!
//! ```yaml
//! entity_types:
//!   competitor_types:
//!     bank:
//!       display_name: "Bank"
//!       default_values:
//!         rate: 11.0
//!     nbfc:
//!       display_name: "NBFC"
//!       default_values:
//!         rate: 18.0
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use voice_agent_core::traits::EntityTypeProvider;
//!
//! fn get_competitor_rate(provider: &dyn EntityTypeProvider, type_id: &str) -> f64 {
//!     provider.get_default_value("competitor_types", type_id, "rate")
//!         .and_then(|v| v.as_f64())
//!         .unwrap_or(15.0) // Fallback rate
//! }
//! ```

use std::collections::HashMap;
use serde_json::Value as JsonValue;

/// Definition of a single entity type (e.g., "bank", "nbfc")
#[derive(Debug, Clone)]
pub struct EntityTypeDefinition {
    /// Type identifier (e.g., "bank")
    pub id: String,
    /// Display name for UI (e.g., "Bank")
    pub display_name: String,
    /// Default values for this type (e.g., {"rate": 11.0})
    pub default_values: HashMap<String, JsonValue>,
    /// Aliases for matching (e.g., ["scheduled_bank", "commercial_bank"])
    pub aliases: Vec<String>,
    /// Description for documentation
    pub description: Option<String>,
}

impl EntityTypeDefinition {
    /// Get a default value as f64
    pub fn get_f64(&self, key: &str) -> Option<f64> {
        self.default_values.get(key).and_then(|v| v.as_f64())
    }

    /// Get a default value as string
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.default_values.get(key).and_then(|v| v.as_str())
    }

    /// Get a default value as bool
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.default_values.get(key).and_then(|v| v.as_bool())
    }

    /// Check if an alias matches this type
    pub fn matches_alias(&self, alias: &str) -> bool {
        let lower = alias.to_lowercase();
        self.id.to_lowercase() == lower
            || self.display_name.to_lowercase() == lower
            || self.aliases.iter().any(|a| a.to_lowercase() == lower)
    }
}

/// Category of entity types (e.g., "competitor_types", "customer_segments")
#[derive(Debug, Clone)]
pub struct EntityTypeCategory {
    /// Category identifier
    pub id: String,
    /// Display name
    pub display_name: String,
    /// Types in this category
    pub types: HashMap<String, EntityTypeDefinition>,
    /// Description
    pub description: Option<String>,
}

impl EntityTypeCategory {
    /// Get a type by ID
    pub fn get_type(&self, type_id: &str) -> Option<&EntityTypeDefinition> {
        self.types.get(type_id)
    }

    /// Get a type by alias (searches all types)
    pub fn get_type_by_alias(&self, alias: &str) -> Option<&EntityTypeDefinition> {
        self.types.values().find(|t| t.matches_alias(alias))
    }

    /// Get all type IDs
    pub fn type_ids(&self) -> Vec<&str> {
        self.types.keys().map(|s| s.as_str()).collect()
    }

    /// Get all type definitions
    pub fn all_types(&self) -> impl Iterator<Item = &EntityTypeDefinition> {
        self.types.values()
    }
}

/// P23 FIX: Provider trait for config-driven entity types
///
/// Replaces hardcoded enums with config-driven type definitions.
pub trait EntityTypeProvider: Send + Sync {
    /// Get all categories
    fn categories(&self) -> Vec<String>;

    /// Get a category by ID
    fn get_category(&self, category_id: &str) -> Option<&EntityTypeCategory>;

    /// Get a type definition
    fn get_type(&self, category_id: &str, type_id: &str) -> Option<&EntityTypeDefinition>;

    /// Get a type by alias (searches within category)
    fn get_type_by_alias(&self, category_id: &str, alias: &str) -> Option<&EntityTypeDefinition>;

    /// Get a default value for a type
    fn get_default_value(
        &self,
        category_id: &str,
        type_id: &str,
        value_key: &str,
    ) -> Option<&JsonValue>;

    /// Get all type IDs in a category
    fn type_ids(&self, category_id: &str) -> Vec<String>;

    /// Check if a type exists
    fn has_type(&self, category_id: &str, type_id: &str) -> bool {
        self.get_type(category_id, type_id).is_some()
    }
}

/// Default implementation using a HashMap
#[derive(Debug, Clone, Default)]
pub struct EntityTypeStore {
    categories: HashMap<String, EntityTypeCategory>,
}

impl EntityTypeStore {
    /// Create a new empty store
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from category definitions
    pub fn from_categories(categories: Vec<EntityTypeCategory>) -> Self {
        let map = categories.into_iter().map(|c| (c.id.clone(), c)).collect();
        Self { categories: map }
    }

    /// Add a category
    pub fn add_category(&mut self, category: EntityTypeCategory) {
        self.categories.insert(category.id.clone(), category);
    }

    /// Add a type to a category
    pub fn add_type(&mut self, category_id: &str, type_def: EntityTypeDefinition) {
        if let Some(category) = self.categories.get_mut(category_id) {
            category.types.insert(type_def.id.clone(), type_def);
        }
    }
}

impl EntityTypeProvider for EntityTypeStore {
    fn categories(&self) -> Vec<String> {
        self.categories.keys().cloned().collect()
    }

    fn get_category(&self, category_id: &str) -> Option<&EntityTypeCategory> {
        self.categories.get(category_id)
    }

    fn get_type(&self, category_id: &str, type_id: &str) -> Option<&EntityTypeDefinition> {
        self.categories
            .get(category_id)
            .and_then(|c| c.get_type(type_id))
    }

    fn get_type_by_alias(&self, category_id: &str, alias: &str) -> Option<&EntityTypeDefinition> {
        self.categories
            .get(category_id)
            .and_then(|c| c.get_type_by_alias(alias))
    }

    fn get_default_value(
        &self,
        category_id: &str,
        type_id: &str,
        value_key: &str,
    ) -> Option<&JsonValue> {
        self.get_type(category_id, type_id)
            .and_then(|t| t.default_values.get(value_key))
    }

    fn type_ids(&self, category_id: &str) -> Vec<String> {
        self.categories
            .get(category_id)
            .map(|c| c.types.keys().cloned().collect())
            .unwrap_or_default()
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> EntityTypeStore {
        let mut store = EntityTypeStore::new();

        // Add competitor types
        let mut competitor_category = EntityTypeCategory {
            id: "competitor_types".to_string(),
            display_name: "Competitor Types".to_string(),
            types: HashMap::new(),
            description: None,
        };

        competitor_category.types.insert(
            "bank".to_string(),
            EntityTypeDefinition {
                id: "bank".to_string(),
                display_name: "Bank".to_string(),
                default_values: {
                    let mut m = HashMap::new();
                    m.insert("rate".to_string(), serde_json::json!(11.0));
                    m
                },
                aliases: vec!["scheduled_bank".to_string()],
                description: None,
            },
        );

        competitor_category.types.insert(
            "nbfc".to_string(),
            EntityTypeDefinition {
                id: "nbfc".to_string(),
                display_name: "NBFC".to_string(),
                default_values: {
                    let mut m = HashMap::new();
                    m.insert("rate".to_string(), serde_json::json!(18.0));
                    m
                },
                aliases: vec!["finance_company".to_string()],
                description: None,
            },
        );

        store.add_category(competitor_category);

        // Add customer segments
        let mut segment_category = EntityTypeCategory {
            id: "customer_segments".to_string(),
            display_name: "Customer Segments".to_string(),
            types: HashMap::new(),
            description: None,
        };

        segment_category.types.insert(
            "high_value".to_string(),
            EntityTypeDefinition {
                id: "high_value".to_string(),
                display_name: "High Value".to_string(),
                default_values: {
                    let mut m = HashMap::new();
                    m.insert("warmth".to_string(), serde_json::json!(0.9));
                    m
                },
                aliases: vec![],
                description: None,
            },
        );

        store.add_category(segment_category);

        store
    }

    #[test]
    fn test_get_type() {
        let store = create_test_store();

        let bank = store.get_type("competitor_types", "bank").unwrap();
        assert_eq!(bank.display_name, "Bank");
        assert_eq!(bank.get_f64("rate"), Some(11.0));
    }

    #[test]
    fn test_get_by_alias() {
        let store = create_test_store();

        let bank = store
            .get_type_by_alias("competitor_types", "scheduled_bank")
            .unwrap();
        assert_eq!(bank.id, "bank");

        let nbfc = store
            .get_type_by_alias("competitor_types", "finance_company")
            .unwrap();
        assert_eq!(nbfc.id, "nbfc");
    }

    #[test]
    fn test_default_values() {
        let store = create_test_store();

        // Test competitor default values
        let bank_rate = store
            .get_default_value("competitor_types", "bank", "rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(15.0);
        assert_eq!(bank_rate, 11.0);

        let nbfc_rate = store
            .get_default_value("competitor_types", "nbfc", "rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(15.0);
        assert_eq!(nbfc_rate, 18.0);

        // Test unknown type returns None (caller provides fallback)
        let unknown_rate = store
            .get_default_value("competitor_types", "unknown", "rate")
            .and_then(|v| v.as_f64());
        assert!(unknown_rate.is_none());

        // Test segment default values
        let warmth = store
            .get_default_value("customer_segments", "high_value", "warmth")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.9);
        assert_eq!(warmth, 0.9);
    }

    #[test]
    fn test_type_ids() {
        let store = create_test_store();

        let competitor_ids = store.type_ids("competitor_types");
        assert!(competitor_ids.contains(&"bank".to_string()));
        assert!(competitor_ids.contains(&"nbfc".to_string()));
    }
}
