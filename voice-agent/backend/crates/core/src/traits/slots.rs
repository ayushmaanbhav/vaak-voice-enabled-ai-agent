//! Slot Schema trait for dynamic slot definitions
//!
//! This module provides a domain-agnostic interface for slot definitions,
//! extraction patterns, validation, and normalization. All slot names and
//! patterns are loaded from configuration.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::SlotSchema;
//!
//! // Schema is created from domain config
//! let schema = config_bridge.slot_schema();
//!
//! // Extract slots from user utterance
//! let slots = schema.extract_slots("I have 100 grams of 22k gold", "en");
//! ```

use std::collections::HashMap;

/// Slot type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum SlotType {
    /// String value
    String,
    /// Numeric value with optional range
    Number {
        min: Option<f64>,
        max: Option<f64>,
    },
    /// Integer value with optional range
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    /// Enumerated value with allowed options
    Enum {
        values: Vec<EnumValue>,
    },
    /// Date value
    Date,
    /// Boolean value
    Boolean,
}

/// Enum value definition with extraction patterns
#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    /// Value ID (e.g., "K24", "muthoot")
    pub id: String,
    /// Display name (e.g., "24 Karat", "Muthoot Finance")
    pub display: String,
    /// Extraction patterns for this value (regex or keywords)
    pub patterns: Vec<String>,
    /// Additional metadata (e.g., purity_factor: 0.916)
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Unit conversion definition
#[derive(Debug, Clone)]
pub struct UnitConversion {
    /// Unit name (e.g., "tola", "lakh")
    pub unit: String,
    /// Multiplication factor to convert to base unit
    pub factor: f64,
}

/// Extracted slot with metadata
#[derive(Debug, Clone)]
pub struct ExtractedSlot {
    /// Slot name
    pub slot_name: String,
    /// Raw extracted value
    pub raw_value: String,
    /// Normalized value (after unit conversion, etc.)
    pub normalized_value: String,
    /// Extraction confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Character span in original text (start, end)
    pub span: Option<(usize, usize)>,
    /// Enum value ID if this is an enum slot
    pub enum_id: Option<String>,
}

/// Validation error for slot values
#[derive(Debug, Clone)]
pub struct SlotValidationError {
    /// Slot name
    pub slot_name: String,
    /// Error message
    pub message: String,
    /// Invalid value
    pub value: String,
}

impl std::fmt::Display for SlotValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot '{}': {} (value: {})", self.slot_name, self.message, self.value)
    }
}

impl std::error::Error for SlotValidationError {}

/// Slot definition trait
///
/// Defines a single slot with its type, validation, extraction patterns,
/// and unit conversions. All definitions are loaded from config.
pub trait SlotDefinition: Send + Sync {
    /// Slot name/identifier (e.g., "asset_quantity", "loan_amount")
    fn name(&self) -> &str;

    /// Human-readable display name
    fn display_name(&self) -> &str;

    /// Slot description
    fn description(&self) -> &str;

    /// Slot type
    fn slot_type(&self) -> &SlotType;

    /// Get extraction patterns for a language
    ///
    /// Returns regex patterns for extracting this slot from text.
    fn extraction_patterns(&self, language: &str) -> Vec<&str>;

    /// Validate a value
    fn validate(&self, value: &str) -> Result<(), SlotValidationError>;

    /// Normalize a value
    ///
    /// Converts values to standard form:
    /// - "5 lakh" → "500000"
    /// - "50 tola" → "583.0" (grams)
    /// - "22k" → "K22"
    fn normalize(&self, value: &str) -> Result<String, SlotValidationError>;

    /// Get unit conversions
    fn unit_conversions(&self) -> Vec<&UnitConversion>;

    /// Apply unit conversion
    fn convert_unit(&self, value: f64, from_unit: &str) -> Option<f64>;

    /// Get default value if any
    fn default_value(&self) -> Option<&str>;

    /// Is this slot required for the domain?
    fn is_required(&self) -> bool;

    /// Get enum values if this is an enum slot
    fn enum_values(&self) -> Option<&[EnumValue]>;

    /// Match enum value from text
    fn match_enum(&self, text: &str) -> Option<&EnumValue>;
}

/// Slot schema manager trait
///
/// Manages all slot definitions for a domain and provides
/// extraction and validation services.
pub trait SlotSchema: Send + Sync {
    /// Get slot definition by name
    fn get_slot(&self, name: &str) -> Option<&dyn SlotDefinition>;

    /// Get all slot names
    fn slot_names(&self) -> Vec<&str>;

    /// Get all slot definitions
    fn all_slots(&self) -> Vec<&dyn SlotDefinition>;

    /// Extract all slots from text
    ///
    /// Runs all extraction patterns against the text and returns
    /// detected slots with their values.
    fn extract_slots(
        &self,
        text: &str,
        language: &str,
    ) -> HashMap<String, ExtractedSlot>;

    /// Extract a specific slot from text
    fn extract_slot(
        &self,
        slot_name: &str,
        text: &str,
        language: &str,
    ) -> Option<ExtractedSlot>;

    /// Validate all slots
    fn validate_slots(
        &self,
        slots: &HashMap<String, String>,
    ) -> Vec<SlotValidationError>;

    /// Normalize all slots
    fn normalize_slots(
        &self,
        slots: &HashMap<String, String>,
    ) -> HashMap<String, String>;

    /// Get slots required for a goal
    fn required_slots_for_goal(&self, goal_id: &str) -> Vec<&str>;

    /// Check if all required slots for a goal are filled
    fn has_required_slots(
        &self,
        goal_id: &str,
        filled_slots: &HashMap<String, String>,
    ) -> bool;

    /// Get missing required slots for a goal
    fn missing_slots_for_goal(
        &self,
        goal_id: &str,
        filled_slots: &HashMap<String, String>,
    ) -> Vec<&str>;
}

/// Config-driven slot definition
#[derive(Debug, Clone)]
pub struct ConfigSlotDefinition {
    name: String,
    display_name: String,
    description: String,
    slot_type: SlotType,
    patterns: HashMap<String, Vec<String>>,
    unit_conversions: Vec<UnitConversion>,
    default_value: Option<String>,
    required: bool,
}

impl ConfigSlotDefinition {
    /// Create a new slot definition
    pub fn new(
        name: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
        slot_type: SlotType,
    ) -> Self {
        Self {
            name: name.into(),
            display_name: display_name.into(),
            description: description.into(),
            slot_type,
            patterns: HashMap::new(),
            unit_conversions: Vec::new(),
            default_value: None,
            required: false,
        }
    }

    /// Add extraction patterns for a language
    pub fn with_patterns(mut self, language: &str, patterns: Vec<String>) -> Self {
        self.patterns.insert(language.to_string(), patterns);
        self
    }

    /// Add unit conversions
    pub fn with_unit_conversions(mut self, conversions: Vec<UnitConversion>) -> Self {
        self.unit_conversions = conversions;
        self
    }

    /// Set default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default_value = Some(default.into());
        self
    }

    /// Set required flag
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    // NOTE: Domain-specific factory methods (asset_quantity, asset_quality, loan_amount,
    // phone_number) have been removed. Use config-driven slots from
    // config/domains/{domain}/slots.yaml via DomainBridge instead.
}

impl SlotDefinition for ConfigSlotDefinition {
    fn name(&self) -> &str {
        &self.name
    }

    fn display_name(&self) -> &str {
        &self.display_name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn slot_type(&self) -> &SlotType {
        &self.slot_type
    }

    fn extraction_patterns(&self, language: &str) -> Vec<&str> {
        self.patterns
            .get(language)
            .map(|p| p.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    fn validate(&self, value: &str) -> Result<(), SlotValidationError> {
        match &self.slot_type {
            SlotType::Number { min, max } => {
                let num: f64 = value.parse().map_err(|_| SlotValidationError {
                    slot_name: self.name.clone(),
                    message: "Must be a number".to_string(),
                    value: value.to_string(),
                })?;

                if let Some(min_val) = min {
                    if num < *min_val {
                        return Err(SlotValidationError {
                            slot_name: self.name.clone(),
                            message: format!("Must be at least {}", min_val),
                            value: value.to_string(),
                        });
                    }
                }

                if let Some(max_val) = max {
                    if num > *max_val {
                        return Err(SlotValidationError {
                            slot_name: self.name.clone(),
                            message: format!("Must be at most {}", max_val),
                            value: value.to_string(),
                        });
                    }
                }

                Ok(())
            }
            SlotType::Enum { values } => {
                let lower = value.to_lowercase();
                if values.iter().any(|v| v.id.to_lowercase() == lower || v.patterns.iter().any(|p| p.to_lowercase() == lower)) {
                    Ok(())
                } else {
                    Err(SlotValidationError {
                        slot_name: self.name.clone(),
                        message: format!("Must be one of: {:?}", values.iter().map(|v| &v.id).collect::<Vec<_>>()),
                        value: value.to_string(),
                    })
                }
            }
            _ => Ok(()),
        }
    }

    fn normalize(&self, value: &str) -> Result<String, SlotValidationError> {
        // For enum types, return the canonical ID
        if let SlotType::Enum { values } = &self.slot_type {
            let lower = value.to_lowercase();
            for v in values {
                if v.id.to_lowercase() == lower {
                    return Ok(v.id.clone());
                }
                for pattern in &v.patterns {
                    if lower.contains(&pattern.to_lowercase()) {
                        return Ok(v.id.clone());
                    }
                }
            }
        }

        // For numbers with units, apply conversion
        // This is a simplified version - full implementation would use regex
        Ok(value.to_string())
    }

    fn unit_conversions(&self) -> Vec<&UnitConversion> {
        self.unit_conversions.iter().collect()
    }

    fn convert_unit(&self, value: f64, from_unit: &str) -> Option<f64> {
        self.unit_conversions
            .iter()
            .find(|c| c.unit.to_lowercase() == from_unit.to_lowercase())
            .map(|c| value * c.factor)
    }

    fn default_value(&self) -> Option<&str> {
        self.default_value.as_deref()
    }

    fn is_required(&self) -> bool {
        self.required
    }

    fn enum_values(&self) -> Option<&[EnumValue]> {
        if let SlotType::Enum { values } = &self.slot_type {
            Some(values)
        } else {
            None
        }
    }

    fn match_enum(&self, text: &str) -> Option<&EnumValue> {
        if let SlotType::Enum { values } = &self.slot_type {
            let lower = text.to_lowercase();
            for v in values {
                for pattern in &v.patterns {
                    if lower.contains(&pattern.to_lowercase()) {
                        return Some(v);
                    }
                }
            }
        }
        None
    }
}

/// Config-driven slot schema implementation
///
/// Manages all slot definitions for a domain and provides
/// extraction and validation services using regex patterns.
pub struct ConfigSlotSchema {
    slots: HashMap<String, ConfigSlotDefinition>,
    /// Goal-to-required-slots mapping
    goal_slots: HashMap<String, Vec<String>>,
    /// Compiled regex patterns per slot per language (lazy-compiled on first use)
    compiled_patterns: std::sync::RwLock<HashMap<String, HashMap<String, Vec<regex::Regex>>>>,
}

impl ConfigSlotSchema {
    /// Create a new slot schema
    pub fn new(slots: Vec<ConfigSlotDefinition>) -> Self {
        let slot_map = slots
            .into_iter()
            .map(|s| (s.name.clone(), s))
            .collect();

        Self {
            slots: slot_map,
            goal_slots: HashMap::new(),
            compiled_patterns: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Add goal-to-slots mapping
    pub fn with_goal_slots(mut self, goal_id: impl Into<String>, slots: Vec<String>) -> Self {
        self.goal_slots.insert(goal_id.into(), slots);
        self
    }

    /// Get or compile patterns for a slot and language
    fn get_compiled_patterns(&self, slot_name: &str, language: &str) -> Vec<regex::Regex> {
        // Check cache first
        {
            let cache = self.compiled_patterns.read().unwrap();
            if let Some(slot_patterns) = cache.get(slot_name) {
                if let Some(lang_patterns) = slot_patterns.get(language) {
                    return lang_patterns.clone();
                }
            }
        }

        // Compile patterns
        let patterns = if let Some(slot) = self.slots.get(slot_name) {
            slot.extraction_patterns(language)
                .iter()
                .filter_map(|p| {
                    regex::RegexBuilder::new(p)
                        .case_insensitive(true)
                        .build()
                        .ok()
                })
                .collect()
        } else {
            Vec::new()
        };

        // Cache compiled patterns
        {
            let mut cache = self.compiled_patterns.write().unwrap();
            cache
                .entry(slot_name.to_string())
                .or_insert_with(HashMap::new)
                .insert(language.to_string(), patterns.clone());
        }

        patterns
    }

    /// Extract a slot value using compiled patterns
    fn extract_with_patterns(
        &self,
        slot: &ConfigSlotDefinition,
        text: &str,
        language: &str,
    ) -> Option<ExtractedSlot> {
        let patterns = self.get_compiled_patterns(slot.name(), language);

        for pattern in &patterns {
            if let Some(m) = pattern.find(text) {
                let raw_value = m.as_str().to_string();
                let normalized = slot.normalize(&raw_value).unwrap_or_else(|_| raw_value.clone());
                let enum_id = slot.match_enum(&raw_value).map(|e| e.id.clone());

                return Some(ExtractedSlot {
                    slot_name: slot.name().to_string(),
                    raw_value,
                    normalized_value: normalized,
                    confidence: 0.9, // Pattern match gives high confidence
                    span: Some((m.start(), m.end())),
                    enum_id,
                });
            }
        }

        // Try enum matching directly for Enum types
        if let Some(enum_val) = slot.match_enum(text) {
            return Some(ExtractedSlot {
                slot_name: slot.name().to_string(),
                raw_value: enum_val.display.clone(),
                normalized_value: enum_val.id.clone(),
                confidence: 0.85,
                span: None,
                enum_id: Some(enum_val.id.clone()),
            });
        }

        None
    }
}

impl SlotSchema for ConfigSlotSchema {
    fn get_slot(&self, name: &str) -> Option<&dyn SlotDefinition> {
        self.slots.get(name).map(|s| s as &dyn SlotDefinition)
    }

    fn slot_names(&self) -> Vec<&str> {
        self.slots.keys().map(|s| s.as_str()).collect()
    }

    fn all_slots(&self) -> Vec<&dyn SlotDefinition> {
        self.slots.values().map(|s| s as &dyn SlotDefinition).collect()
    }

    fn extract_slots(&self, text: &str, language: &str) -> HashMap<String, ExtractedSlot> {
        let mut result = HashMap::new();

        for slot in self.slots.values() {
            if let Some(extracted) = self.extract_with_patterns(slot, text, language) {
                result.insert(slot.name().to_string(), extracted);
            }
        }

        result
    }

    fn extract_slot(
        &self,
        slot_name: &str,
        text: &str,
        language: &str,
    ) -> Option<ExtractedSlot> {
        self.slots
            .get(slot_name)
            .and_then(|slot| self.extract_with_patterns(slot, text, language))
    }

    fn validate_slots(&self, slots: &HashMap<String, String>) -> Vec<SlotValidationError> {
        let mut errors = Vec::new();

        for (name, value) in slots {
            if let Some(slot) = self.slots.get(name) {
                if let Err(e) = slot.validate(value) {
                    errors.push(e);
                }
            }
        }

        errors
    }

    fn normalize_slots(&self, slots: &HashMap<String, String>) -> HashMap<String, String> {
        slots
            .iter()
            .map(|(name, value)| {
                let normalized = self
                    .slots
                    .get(name)
                    .and_then(|slot| slot.normalize(value).ok())
                    .unwrap_or_else(|| value.clone());
                (name.clone(), normalized)
            })
            .collect()
    }

    fn required_slots_for_goal(&self, goal_id: &str) -> Vec<&str> {
        self.goal_slots
            .get(goal_id)
            .map(|slots| slots.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    fn has_required_slots(&self, goal_id: &str, filled_slots: &HashMap<String, String>) -> bool {
        self.required_slots_for_goal(goal_id)
            .iter()
            .all(|slot| filled_slots.contains_key(*slot))
    }

    fn missing_slots_for_goal(
        &self,
        goal_id: &str,
        filled_slots: &HashMap<String, String>,
    ) -> Vec<&str> {
        self.required_slots_for_goal(goal_id)
            .into_iter()
            .filter(|slot| !filled_slots.contains_key(*slot))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_definition_builder() {
        let slot = ConfigSlotDefinition::new(
            "test_slot",
            "Test Slot",
            "A test slot for unit tests",
            SlotType::String,
        );
        assert_eq!(slot.name(), "test_slot");
        assert_eq!(slot.display_name(), "Test Slot");
        assert!(!slot.is_required());
    }

    #[test]
    fn test_number_slot_validation() {
        let slot = ConfigSlotDefinition::new(
            "quantity",
            "Quantity",
            "Numeric quantity",
            SlotType::Number { min: Some(1.0), max: Some(1000.0) },
        );

        assert!(slot.validate("100").is_ok());
        assert!(slot.validate("0.5").is_err()); // Below min
        assert!(slot.validate("2000").is_err()); // Above max
        assert!(slot.validate("invalid").is_err()); // Not a number
    }

    #[test]
    fn test_enum_slot() {
        let slot = ConfigSlotDefinition::new(
            "quality",
            "Quality",
            "Quality enumeration",
            SlotType::Enum {
                values: vec![
                    EnumValue {
                        id: "high".to_string(),
                        display: "High Quality".to_string(),
                        patterns: vec!["high".to_string(), "premium".to_string()],
                        metadata: HashMap::new(),
                    },
                    EnumValue {
                        id: "low".to_string(),
                        display: "Low Quality".to_string(),
                        patterns: vec!["low".to_string(), "basic".to_string()],
                        metadata: HashMap::new(),
                    },
                ],
            },
        );

        assert!(matches!(slot.slot_type(), SlotType::Enum { .. }));
        let enum_values = slot.enum_values().unwrap();
        assert_eq!(enum_values.len(), 2);
        assert_eq!(enum_values[0].id, "high");
    }

    #[test]
    fn test_unit_conversion() {
        let slot = ConfigSlotDefinition::new(
            "weight",
            "Weight",
            "Weight in grams",
            SlotType::Number { min: Some(0.0), max: None },
        )
        .with_unit_conversions(vec![
            UnitConversion { unit: "kg".to_string(), factor: 1000.0 },
            UnitConversion { unit: "tola".to_string(), factor: 11.66 },
        ]);

        // 10 tola = 116.6 grams
        let grams = slot.convert_unit(10.0, "tola");
        assert_eq!(grams, Some(116.6));

        // 2 kg = 2000 grams
        let grams_from_kg = slot.convert_unit(2.0, "kg");
        assert_eq!(grams_from_kg, Some(2000.0));
    }

    #[test]
    fn test_enum_matching() {
        let slot = ConfigSlotDefinition::new(
            "quality",
            "Quality",
            "Quality enumeration",
            SlotType::Enum {
                values: vec![
                    EnumValue {
                        id: "K22".to_string(),
                        display: "22 Karat".to_string(),
                        patterns: vec!["22k".to_string(), "22 karat".to_string()],
                        metadata: HashMap::new(),
                    },
                ],
            },
        );

        let matched = slot.match_enum("I have 22k gold");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().id, "K22");
    }

    #[test]
    fn test_extraction_patterns() {
        let slot = ConfigSlotDefinition::new(
            "amount",
            "Amount",
            "Loan amount",
            SlotType::Number { min: None, max: None },
        )
        .with_patterns("en", vec![
            r"\d+\s*(?:crore|cr)".to_string(),
            r"\d+\s*(?:lakh|lac)".to_string(),
        ])
        .with_patterns("hi", vec![
            r"\d+\s*(?:करोड़|लाख)".to_string(),
        ]);

        let en_patterns = slot.extraction_patterns("en");
        assert_eq!(en_patterns.len(), 2);

        let hi_patterns = slot.extraction_patterns("hi");
        assert_eq!(hi_patterns.len(), 1);
    }

    #[test]
    fn test_string_validation() {
        let slot = ConfigSlotDefinition::new(
            "phone",
            "Phone Number",
            "Customer phone",
            SlotType::String,
        );

        // String type accepts any value
        assert!(slot.validate("9876543210").is_ok());
        assert!(slot.validate("anything").is_ok());
    }

    #[test]
    fn test_required_slot() {
        let slot = ConfigSlotDefinition::new(
            "required_field",
            "Required Field",
            "A required slot",
            SlotType::String,
        )
        .required();

        assert!(slot.is_required());
    }

    #[test]
    fn test_default_value() {
        let slot = ConfigSlotDefinition::new(
            "optional_field",
            "Optional Field",
            "An optional slot",
            SlotType::String,
        )
        .with_default("default_value");

        assert_eq!(slot.default_value(), Some("default_value"));
    }

    // ============= ConfigSlotSchema Tests =============

    fn create_test_schema() -> ConfigSlotSchema {
        let amount_slot = ConfigSlotDefinition::new(
            "amount",
            "Amount",
            "Loan amount",
            SlotType::Number { min: Some(10000.0), max: Some(10000000.0) },
        )
        .with_patterns("en", vec![
            r"(\d+)\s*(?:lakh|lac)".to_string(),
            r"(\d+)\s*(?:crore|cr)".to_string(),
        ]);

        let quality_slot = ConfigSlotDefinition::new(
            "quality",
            "Quality",
            "Gold quality",
            SlotType::Enum {
                values: vec![
                    EnumValue {
                        id: "K24".to_string(),
                        display: "24 Karat".to_string(),
                        patterns: vec!["24k".to_string(), "24 karat".to_string(), "pure".to_string()],
                        metadata: HashMap::new(),
                    },
                    EnumValue {
                        id: "K22".to_string(),
                        display: "22 Karat".to_string(),
                        patterns: vec!["22k".to_string(), "22 karat".to_string()],
                        metadata: HashMap::new(),
                    },
                ],
            },
        );

        let name_slot = ConfigSlotDefinition::new(
            "name",
            "Customer Name",
            "Customer name",
            SlotType::String,
        );

        ConfigSlotSchema::new(vec![amount_slot, quality_slot, name_slot])
            .with_goal_slots("loan_application", vec!["amount".to_string(), "quality".to_string()])
            .with_goal_slots("contact", vec!["name".to_string()])
    }

    #[test]
    fn test_schema_get_slot() {
        let schema = create_test_schema();

        assert!(schema.get_slot("amount").is_some());
        assert!(schema.get_slot("quality").is_some());
        assert!(schema.get_slot("nonexistent").is_none());
    }

    #[test]
    fn test_schema_slot_names() {
        let schema = create_test_schema();
        let names = schema.slot_names();

        assert_eq!(names.len(), 3);
        assert!(names.contains(&"amount"));
        assert!(names.contains(&"quality"));
        assert!(names.contains(&"name"));
    }

    #[test]
    fn test_schema_extract_slots() {
        let schema = create_test_schema();
        let extracted = schema.extract_slots("I need 5 lakh loan for 22k gold", "en");

        assert!(extracted.contains_key("amount"));
        assert!(extracted.contains_key("quality"));

        let quality = &extracted["quality"];
        assert_eq!(quality.normalized_value, "K22");
    }

    #[test]
    fn test_schema_extract_single_slot() {
        let schema = create_test_schema();

        let amount = schema.extract_slot("amount", "I need 10 lakh", "en");
        assert!(amount.is_some());

        let nonexistent = schema.extract_slot("nonexistent", "test", "en");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_schema_validate_slots() {
        let schema = create_test_schema();

        let mut slots = HashMap::new();
        slots.insert("amount".to_string(), "50000".to_string());
        slots.insert("name".to_string(), "John".to_string());

        let errors = schema.validate_slots(&slots);
        assert!(errors.is_empty());

        // Invalid amount (out of range)
        slots.insert("amount".to_string(), "1000".to_string()); // Below min
        let errors = schema.validate_slots(&slots);
        assert!(!errors.is_empty());
    }

    #[test]
    fn test_schema_normalize_slots() {
        let schema = create_test_schema();

        let mut slots = HashMap::new();
        slots.insert("quality".to_string(), "22k".to_string());

        let normalized = schema.normalize_slots(&slots);
        assert_eq!(normalized.get("quality"), Some(&"K22".to_string()));
    }

    #[test]
    fn test_schema_goal_slots() {
        let schema = create_test_schema();

        let loan_slots = schema.required_slots_for_goal("loan_application");
        assert_eq!(loan_slots.len(), 2);
        assert!(loan_slots.contains(&"amount"));
        assert!(loan_slots.contains(&"quality"));

        let contact_slots = schema.required_slots_for_goal("contact");
        assert_eq!(contact_slots.len(), 1);
        assert!(contact_slots.contains(&"name"));
    }

    #[test]
    fn test_schema_has_required_slots() {
        let schema = create_test_schema();

        let mut filled = HashMap::new();
        assert!(!schema.has_required_slots("loan_application", &filled));

        filled.insert("amount".to_string(), "50000".to_string());
        assert!(!schema.has_required_slots("loan_application", &filled));

        filled.insert("quality".to_string(), "K22".to_string());
        assert!(schema.has_required_slots("loan_application", &filled));
    }

    #[test]
    fn test_schema_missing_slots() {
        let schema = create_test_schema();

        let mut filled = HashMap::new();
        filled.insert("amount".to_string(), "50000".to_string());

        let missing = schema.missing_slots_for_goal("loan_application", &filled);
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&"quality"));
    }
}
