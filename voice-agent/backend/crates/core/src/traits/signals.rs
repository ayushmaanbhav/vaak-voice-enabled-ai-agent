//! Signal Provider Trait for Domain-Agnostic Lead Scoring
//!
//! P23 FIX: This module provides a config-driven signal system that replaces
//! the hardcoded LeadSignals trait methods.
//!
//! Instead of 15+ hardcoded methods like `has_urgency_signal()`, `engagement_turns()`, etc.,
//! signals are now defined in config (signals.yaml) and accessed generically.
//!
//! # Example Config (signals.yaml)
//!
//! ```yaml
//! signals:
//!   has_urgency:
//!     display_name: "Urgency Detected"
//!     type: boolean
//!     category: urgency
//!     weight: 10
//!
//!   engagement_turns:
//!     display_name: "Engagement Turns"
//!     type: counter
//!     category: engagement
//!     weight: 3
//!     max: 5
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use voice_agent_core::traits::SignalProvider;
//!
//! fn score_lead(signals: &dyn SignalProvider) -> u32 {
//!     let mut score = 0;
//!     if signals.has_signal("has_urgency") {
//!         score += 10;
//!     }
//!     if let Some(turns) = signals.get_counter("engagement_turns") {
//!         score += turns * 3;
//!     }
//!     score
//! }
//! ```

use std::collections::HashMap;

/// Signal type definition from config
#[derive(Debug, Clone, PartialEq)]
pub enum SignalType {
    /// Boolean signal (present/absent)
    Boolean,
    /// Counter signal (0-N)
    Counter { max: Option<u32> },
    /// String signal (categorical value)
    String { allowed_values: Option<Vec<String>> },
    /// Numeric signal (float value)
    Numeric { min: Option<f64>, max: Option<f64> },
}

impl Default for SignalType {
    fn default() -> Self {
        SignalType::Boolean
    }
}

/// Signal definition from config
#[derive(Debug, Clone)]
pub struct SignalDefinition {
    /// Signal identifier (e.g., "has_urgency")
    pub id: String,
    /// Display name for UI/logging
    pub display_name: String,
    /// Signal type
    pub signal_type: SignalType,
    /// Category for grouping (e.g., "urgency", "engagement", "information")
    pub category: String,
    /// Weight for scoring (0-100)
    pub weight: u32,
    /// Description for documentation
    pub description: Option<String>,
}

/// Signal value that can hold different types
#[derive(Debug, Clone)]
pub enum SignalValue {
    /// Boolean signal value
    Boolean(bool),
    /// Counter signal value
    Counter(u32),
    /// String signal value
    String(String),
    /// Numeric signal value
    Numeric(f64),
}

impl SignalValue {
    /// Check if signal is "active" (true for boolean, > 0 for counter, non-empty for string)
    pub fn is_active(&self) -> bool {
        match self {
            SignalValue::Boolean(v) => *v,
            SignalValue::Counter(v) => *v > 0,
            SignalValue::String(v) => !v.is_empty(),
            SignalValue::Numeric(v) => *v > 0.0,
        }
    }

    /// Get as boolean (true if active)
    pub fn as_bool(&self) -> bool {
        self.is_active()
    }

    /// Get as counter (0 if not a counter)
    pub fn as_counter(&self) -> u32 {
        match self {
            SignalValue::Counter(v) => *v,
            SignalValue::Boolean(true) => 1,
            SignalValue::Boolean(false) => 0,
            _ => 0,
        }
    }

    /// Get as string (empty if not a string)
    pub fn as_string(&self) -> Option<&str> {
        match self {
            SignalValue::String(v) => Some(v.as_str()),
            _ => None,
        }
    }

    /// Get as numeric (0.0 if not numeric)
    pub fn as_numeric(&self) -> f64 {
        match self {
            SignalValue::Numeric(v) => *v,
            SignalValue::Counter(v) => *v as f64,
            SignalValue::Boolean(true) => 1.0,
            SignalValue::Boolean(false) => 0.0,
            _ => 0.0,
        }
    }
}

/// P23 FIX: Generic signal provider trait
///
/// Replaces the hardcoded LeadSignals trait with config-driven signal access.
/// All signal IDs come from signals.yaml config file.
pub trait SignalProvider: Send + Sync {
    /// Check if a boolean signal is active
    fn has_signal(&self, signal_id: &str) -> bool;

    /// Get a counter signal value (returns 0 if not set)
    fn get_counter(&self, signal_id: &str) -> u32;

    /// Get a string signal value
    fn get_string(&self, signal_id: &str) -> Option<String>;

    /// Get a numeric signal value
    fn get_numeric(&self, signal_id: &str) -> Option<f64>;

    /// Get a signal value of any type
    fn get_value(&self, signal_id: &str) -> Option<SignalValue>;

    /// Get all active signal IDs
    fn active_signals(&self) -> Vec<String>;

    /// Get all signal IDs (active and inactive)
    fn all_signals(&self) -> Vec<String>;

    /// Get signals by category
    fn signals_in_category(&self, category: &str) -> Vec<String>;
}

/// Default implementation of SignalProvider using a HashMap
#[derive(Debug, Clone, Default)]
pub struct SignalStore {
    /// Signal values keyed by signal ID
    signals: HashMap<String, SignalValue>,
    /// Signal definitions from config (optional, for validation)
    definitions: Option<HashMap<String, SignalDefinition>>,
}

impl SignalStore {
    /// Create a new empty signal store
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with signal definitions for validation
    pub fn with_definitions(definitions: Vec<SignalDefinition>) -> Self {
        let defs = definitions
            .into_iter()
            .map(|d| (d.id.clone(), d))
            .collect();
        Self {
            signals: HashMap::new(),
            definitions: Some(defs),
        }
    }

    /// Set a boolean signal
    pub fn set_boolean(&mut self, signal_id: &str, value: bool) {
        self.signals
            .insert(signal_id.to_string(), SignalValue::Boolean(value));
    }

    /// Set a counter signal
    pub fn set_counter(&mut self, signal_id: &str, value: u32) {
        // Apply max constraint if definition exists
        let capped_value = if let Some(defs) = &self.definitions {
            if let Some(def) = defs.get(signal_id) {
                if let SignalType::Counter { max: Some(max) } = def.signal_type {
                    value.min(max)
                } else {
                    value
                }
            } else {
                value
            }
        } else {
            value
        };
        self.signals
            .insert(signal_id.to_string(), SignalValue::Counter(capped_value));
    }

    /// Increment a counter signal
    pub fn increment_counter(&mut self, signal_id: &str) {
        let current = self.get_counter(signal_id);
        self.set_counter(signal_id, current + 1);
    }

    /// Set a string signal
    pub fn set_string(&mut self, signal_id: &str, value: impl Into<String>) {
        self.signals
            .insert(signal_id.to_string(), SignalValue::String(value.into()));
    }

    /// Set a numeric signal
    pub fn set_numeric(&mut self, signal_id: &str, value: f64) {
        self.signals
            .insert(signal_id.to_string(), SignalValue::Numeric(value));
    }

    /// Remove a signal
    pub fn clear(&mut self, signal_id: &str) {
        self.signals.remove(signal_id);
    }

    /// Clear all signals
    pub fn clear_all(&mut self) {
        self.signals.clear();
    }
}

impl SignalProvider for SignalStore {
    fn has_signal(&self, signal_id: &str) -> bool {
        self.signals
            .get(signal_id)
            .map(|v| v.is_active())
            .unwrap_or(false)
    }

    fn get_counter(&self, signal_id: &str) -> u32 {
        self.signals
            .get(signal_id)
            .map(|v| v.as_counter())
            .unwrap_or(0)
    }

    fn get_string(&self, signal_id: &str) -> Option<String> {
        self.signals
            .get(signal_id)
            .and_then(|v| v.as_string().map(String::from))
    }

    fn get_numeric(&self, signal_id: &str) -> Option<f64> {
        self.signals.get(signal_id).map(|v| v.as_numeric())
    }

    fn get_value(&self, signal_id: &str) -> Option<SignalValue> {
        self.signals.get(signal_id).cloned()
    }

    fn active_signals(&self) -> Vec<String> {
        self.signals
            .iter()
            .filter(|(_, v)| v.is_active())
            .map(|(k, _)| k.clone())
            .collect()
    }

    fn all_signals(&self) -> Vec<String> {
        self.signals.keys().cloned().collect()
    }

    fn signals_in_category(&self, category: &str) -> Vec<String> {
        if let Some(defs) = &self.definitions {
            defs.values()
                .filter(|d| d.category == category)
                .filter(|d| self.has_signal(&d.id))
                .map(|d| d.id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_store_boolean() {
        let mut store = SignalStore::new();

        assert!(!store.has_signal("test_signal"));

        store.set_boolean("test_signal", true);
        assert!(store.has_signal("test_signal"));

        store.set_boolean("test_signal", false);
        assert!(!store.has_signal("test_signal"));
    }

    #[test]
    fn test_signal_store_counter() {
        let mut store = SignalStore::new();

        assert_eq!(store.get_counter("turns"), 0);

        store.set_counter("turns", 5);
        assert_eq!(store.get_counter("turns"), 5);
        assert!(store.has_signal("turns")); // Counter > 0 is active

        store.increment_counter("turns");
        assert_eq!(store.get_counter("turns"), 6);
    }

    #[test]
    fn test_signal_store_with_max() {
        let definitions = vec![SignalDefinition {
            id: "limited".to_string(),
            display_name: "Limited Counter".to_string(),
            signal_type: SignalType::Counter { max: Some(3) },
            category: "test".to_string(),
            weight: 5,
            description: None,
        }];

        let mut store = SignalStore::with_definitions(definitions);

        store.set_counter("limited", 10);
        assert_eq!(store.get_counter("limited"), 3); // Capped at max
    }

    #[test]
    fn test_active_signals() {
        let mut store = SignalStore::new();

        store.set_boolean("signal_a", true);
        store.set_boolean("signal_b", false);
        store.set_counter("signal_c", 5);
        store.set_counter("signal_d", 0);

        let active = store.active_signals();
        assert!(active.contains(&"signal_a".to_string()));
        assert!(!active.contains(&"signal_b".to_string()));
        assert!(active.contains(&"signal_c".to_string()));
        assert!(!active.contains(&"signal_d".to_string()));
    }

    #[test]
    fn test_signal_provider_methods() {
        let mut store = SignalStore::new();

        store.set_boolean("has_urgency", true);
        store.set_counter("engagement_turns", 3);
        store.set_string("current_stage", "discovery");

        // Use SignalProvider trait methods directly
        assert!(store.has_signal("has_urgency"));
        assert_eq!(store.get_counter("engagement_turns"), 3);
        assert_eq!(store.get_string("current_stage"), Some("discovery".to_string()));
    }
}
