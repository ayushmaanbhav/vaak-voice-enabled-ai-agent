//! Config Validator for Domain Configuration
//!
//! Validates domain configuration at startup to catch errors early.
//! Performs:
//! - Required files check
//! - Cross-reference validation (e.g., goals reference valid slots)
//! - Value range validation
//! - Schema completeness checks
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_config::domain::ConfigValidator;
//!
//! let validator = ConfigValidator::new();
//! let result = validator.validate_domain("gold_loan", &config)?;
//! ```

use std::collections::HashSet;
use super::MasterDomainConfig;
use super::slots::SlotType;

/// Validation error with context
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Category of error
    pub category: ValidationCategory,
    /// Source file or config section
    pub source: String,
    /// Specific field or reference
    pub field: Option<String>,
    /// Error message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let field_str = self.field.as_deref().unwrap_or("(root)");
        write!(
            f,
            "[{:?}] {}/{}: {}",
            self.severity, self.source, field_str, self.message
        )
    }
}

impl std::error::Error for ValidationError {}

/// Category of validation error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationCategory {
    /// Missing required configuration
    MissingRequired,
    /// Invalid cross-reference
    InvalidReference,
    /// Value out of expected range
    ValueOutOfRange,
    /// Duplicate definition
    Duplicate,
    /// Schema mismatch
    SchemaMismatch,
    /// Unused definition (warning)
    Unused,
}

/// Severity of validation error
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Informational warning
    Warning,
    /// Potential issue
    Error,
    /// Critical - will prevent startup
    Critical,
}

/// Validation result
#[derive(Debug, Default)]
pub struct ValidationResult {
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// Domain name being validated
    pub domain: String,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            errors: Vec::new(),
            domain: domain.into(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Add a critical error
    pub fn add_critical(&mut self, source: &str, message: &str) {
        self.errors.push(ValidationError {
            category: ValidationCategory::MissingRequired,
            source: source.to_string(),
            field: None,
            message: message.to_string(),
            severity: ValidationSeverity::Critical,
        });
    }

    /// Add a reference error
    pub fn add_reference_error(&mut self, source: &str, field: &str, message: &str) {
        self.errors.push(ValidationError {
            category: ValidationCategory::InvalidReference,
            source: source.to_string(),
            field: Some(field.to_string()),
            message: message.to_string(),
            severity: ValidationSeverity::Error,
        });
    }

    /// Add a warning
    pub fn add_warning(&mut self, source: &str, field: &str, message: &str) {
        self.errors.push(ValidationError {
            category: ValidationCategory::Unused,
            source: source.to_string(),
            field: Some(field.to_string()),
            message: message.to_string(),
            severity: ValidationSeverity::Warning,
        });
    }

    /// Check if validation passed (no critical errors)
    pub fn is_ok(&self) -> bool {
        !self.errors.iter().any(|e| e.severity == ValidationSeverity::Critical)
    }

    /// Get only critical errors
    pub fn critical_errors(&self) -> Vec<&ValidationError> {
        self.errors
            .iter()
            .filter(|e| e.severity == ValidationSeverity::Critical)
            .collect()
    }

    /// Get errors and critical errors (not warnings)
    pub fn errors_and_critical(&self) -> Vec<&ValidationError> {
        self.errors
            .iter()
            .filter(|e| e.severity >= ValidationSeverity::Error)
            .collect()
    }

    /// Summary string
    pub fn summary(&self) -> String {
        let critical = self.errors.iter().filter(|e| e.severity == ValidationSeverity::Critical).count();
        let errors = self.errors.iter().filter(|e| e.severity == ValidationSeverity::Error).count();
        let warnings = self.errors.iter().filter(|e| e.severity == ValidationSeverity::Warning).count();

        if self.errors.is_empty() {
            format!("Domain '{}': All validations passed", self.domain)
        } else {
            format!(
                "Domain '{}': {} critical, {} errors, {} warnings",
                self.domain, critical, errors, warnings
            )
        }
    }
}

/// Config validator
pub struct ConfigValidator {
    /// Whether to include warnings
    include_warnings: bool,
}

impl Default for ConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            include_warnings: true,
        }
    }

    /// Set whether to include warnings
    pub fn with_warnings(mut self, include: bool) -> Self {
        self.include_warnings = include;
        self
    }

    /// Validate a domain configuration
    pub fn validate(&self, domain: &str, config: &MasterDomainConfig) -> ValidationResult {
        let mut result = ValidationResult::new(domain);

        // 1. Validate slots config
        self.validate_slots(config, &mut result);

        // 2. Validate goals config
        self.validate_goals(config, &mut result);

        // 3. Validate stages config
        self.validate_stages(config, &mut result);

        // 4. Validate competitors config
        self.validate_competitors(config, &mut result);

        // 5. Validate objections config
        self.validate_objections(config, &mut result);

        // 6. Validate scoring config
        self.validate_scoring(config, &mut result);

        // 7. Cross-validate references
        self.validate_cross_references(config, &mut result);

        result
    }

    /// Validate slots configuration
    fn validate_slots(&self, config: &MasterDomainConfig, result: &mut ValidationResult) {
        let slots = &config.slots;

        // Check slots is not empty
        if slots.slots.is_empty() {
            result.add_critical("slots.yaml", "No slots defined");
            return;
        }

        // Check each slot has required fields
        for (id, slot) in &slots.slots {
            if slot.description.is_empty() {
                result.add_reference_error("slots.yaml", id, "Slot missing description");
            }

            // Check enum slots have values
            if slot.slot_type == SlotType::Enum {
                if slot.values.as_ref().map(|v| v.is_empty()).unwrap_or(true) {
                    result.add_reference_error(
                        "slots.yaml",
                        id,
                        "Enum slot must have at least one value",
                    );
                }
            }

            // Check numeric slots have valid ranges
            if slot.slot_type == SlotType::Number {
                if let (Some(min), Some(max)) = (slot.min, slot.max) {
                    if min > max {
                        result.add_reference_error(
                            "slots.yaml",
                            id,
                            &format!("Invalid range: min ({}) > max ({})", min, max),
                        );
                    }
                }
            }
        }
    }

    /// Validate goals configuration
    fn validate_goals(&self, config: &MasterDomainConfig, result: &mut ValidationResult) {
        let goals = &config.goals;

        if goals.goals.is_empty() {
            result.add_critical("goals.yaml", "No goals defined");
            return;
        }

        for (id, goal) in &goals.goals {
            if goal.display_name.is_empty() {
                result.add_reference_error("goals.yaml", id, "Goal missing display_name");
            }
        }
    }

    /// Validate stages configuration
    fn validate_stages(&self, config: &MasterDomainConfig, result: &mut ValidationResult) {
        let stages = &config.stages;

        if stages.stages.is_empty() {
            result.add_critical("stages.yaml", "No stages defined");
            return;
        }

        // Validate stage transitions reference valid stages
        let stage_ids: HashSet<_> = stages.stages.keys().cloned().collect();
        for (id, stage) in &stages.stages {
            // Stage.transitions is Vec<String> of target stage IDs
            for target_stage in &stage.transitions {
                if !stage_ids.contains(target_stage) {
                    result.add_reference_error(
                        "stages.yaml",
                        id,
                        &format!("Transition references unknown stage: {}", target_stage),
                    );
                }
            }
        }
    }

    /// Validate competitors configuration
    fn validate_competitors(&self, config: &MasterDomainConfig, result: &mut ValidationResult) {
        // Use competitors_config which is loaded from competitors.yaml
        let competitors = &config.competitors_config;

        for (id, comp) in &competitors.competitors {
            if comp.display_name.is_empty() {
                result.add_reference_error("competitors.yaml", id, "Competitor missing display_name");
            }

            // Check rate ranges
            if let Some(rate_range) = &comp.rate_range {
                if rate_range.min > rate_range.max {
                    result.add_reference_error(
                        "competitors.yaml",
                        id,
                        &format!(
                            "Invalid rate range: min ({}) > max ({})",
                            rate_range.min, rate_range.max
                        ),
                    );
                }

                // Check typical_rate is within range
                if comp.typical_rate < rate_range.min || comp.typical_rate > rate_range.max {
                    if self.include_warnings {
                        result.add_warning(
                            "competitors.yaml",
                            id,
                            &format!(
                                "typical_rate ({}) is outside rate_range ({}-{})",
                                comp.typical_rate, rate_range.min, rate_range.max
                            ),
                        );
                    }
                }
            }
        }
    }

    /// Validate objections configuration
    fn validate_objections(&self, config: &MasterDomainConfig, result: &mut ValidationResult) {
        let objections = &config.objections;

        if objections.objections.is_empty() {
            if self.include_warnings {
                result.add_warning("objections.yaml", "objections", "No objections defined");
            }
            return;
        }

        for (id, objection) in &objections.objections {
            if objection.patterns.is_empty() {
                result.add_reference_error("objections.yaml", id, "Objection has no detection patterns");
            }

            if objection.responses.is_empty() {
                result.add_reference_error("objections.yaml", id, "Objection has no responses");
            }
        }
    }

    /// Validate scoring configuration
    fn validate_scoring(&self, config: &MasterDomainConfig, result: &mut ValidationResult) {
        let scoring = &config.scoring;

        // Validate thresholds are in order
        // QualificationThresholds has: cold, warm, hot, qualified (where each is the minimum score for that level)
        let thresholds = &scoring.qualification_thresholds;

        // cold < warm < hot < qualified
        if thresholds.cold >= thresholds.warm {
            result.add_reference_error(
                "scoring.yaml",
                "qualification_thresholds",
                &format!(
                    "cold ({}) should be < warm ({})",
                    thresholds.cold, thresholds.warm
                ),
            );
        }
        if thresholds.warm >= thresholds.hot {
            result.add_reference_error(
                "scoring.yaml",
                "qualification_thresholds",
                &format!(
                    "warm ({}) should be < hot ({})",
                    thresholds.warm, thresholds.hot
                ),
            );
        }
        if thresholds.hot >= thresholds.qualified {
            result.add_reference_error(
                "scoring.yaml",
                "qualification_thresholds",
                &format!(
                    "hot ({}) should be < qualified ({})",
                    thresholds.hot, thresholds.qualified
                ),
            );
        }
    }

    /// Validate cross-references between config files
    fn validate_cross_references(&self, config: &MasterDomainConfig, result: &mut ValidationResult) {
        // Collect all slot IDs
        let slot_ids: HashSet<_> = config.slots.slots.keys().cloned().collect();

        // Check goals reference valid slots
        for (goal_id, goal) in &config.goals.goals {
            for slot in &goal.required_slots {
                if !slot_ids.contains(slot) {
                    result.add_reference_error(
                        "goals.yaml",
                        goal_id,
                        &format!("Goal references unknown required slot: {}", slot),
                    );
                }
            }

            for slot in &goal.optional_slots {
                if !slot_ids.contains(slot) {
                    result.add_reference_error(
                        "goals.yaml",
                        goal_id,
                        &format!("Goal references unknown optional slot: {}", slot),
                    );
                }
            }
        }

        // Check for unused slots (warning only)
        if self.include_warnings {
            let mut used_slots = HashSet::new();
            for goal in config.goals.goals.values() {
                used_slots.extend(goal.required_slots.iter().cloned());
                used_slots.extend(goal.optional_slots.iter().cloned());
            }

            for slot_id in &slot_ids {
                if !used_slots.contains(slot_id) {
                    result.add_warning(
                        "slots.yaml",
                        slot_id,
                        "Slot is defined but not referenced by any goal",
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_summary() {
        let mut result = ValidationResult::new("test_domain");
        assert!(result.is_ok());
        assert!(result.summary().contains("All validations passed"));

        result.add_warning("test", "field", "warning message");
        assert!(result.is_ok()); // Warnings don't fail

        result.add_critical("test", "critical error");
        assert!(!result.is_ok());
        assert!(result.summary().contains("1 critical"));
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError {
            category: ValidationCategory::InvalidReference,
            source: "goals.yaml".to_string(),
            field: Some("balance_transfer".to_string()),
            message: "References unknown slot".to_string(),
            severity: ValidationSeverity::Error,
        };

        let display = format!("{}", error);
        assert!(display.contains("goals.yaml"));
        assert!(display.contains("balance_transfer"));
        assert!(display.contains("References unknown slot"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ValidationSeverity::Warning < ValidationSeverity::Error);
        assert!(ValidationSeverity::Error < ValidationSeverity::Critical);
    }
}
