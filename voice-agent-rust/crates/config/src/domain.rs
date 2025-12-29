//! Domain configuration loader
//!
//! Unified interface for loading and accessing all domain-specific configuration.

use std::path::Path;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use parking_lot::RwLock;

use crate::{
    GoldLoanConfig, ConfigError,
    branch::BranchConfig,
    product::ProductConfig,
    competitor::CompetitorConfig,
    prompts::PromptTemplates,
};

/// Complete domain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    /// Domain name
    #[serde(default = "default_domain")]
    pub domain: String,
    /// Domain version
    #[serde(default = "default_version")]
    pub version: String,
    /// Gold loan business configuration
    #[serde(default)]
    pub gold_loan: GoldLoanConfig,
    /// Branch configuration
    #[serde(default)]
    pub branches: BranchConfig,
    /// Product configuration
    #[serde(default)]
    pub product: ProductConfig,
    /// Competitor configuration
    #[serde(default)]
    pub competitors: CompetitorConfig,
    /// Prompt templates
    #[serde(default)]
    pub prompts: PromptTemplates,
}

fn default_domain() -> String {
    "gold_loan".to_string()
}

fn default_version() -> String {
    "1.0.0".to_string()
}

impl Default for DomainConfig {
    fn default() -> Self {
        Self {
            domain: default_domain(),
            version: default_version(),
            gold_loan: GoldLoanConfig::default(),
            branches: BranchConfig::default(),
            product: ProductConfig::default(),
            competitors: CompetitorConfig::default(),
            prompts: PromptTemplates::default(),
        }
    }
}

impl DomainConfig {
    /// Create new domain config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from YAML file
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Load from JSON file
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        serde_json::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save to YAML file
    pub fn to_yaml_file(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        std::fs::write(path, content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save to JSON file
    pub fn to_json_file(&self, path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        std::fs::write(path, content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate gold loan config
        if self.gold_loan.kotak_interest_rate <= 0.0 {
            errors.push("Interest rate must be positive".to_string());
        }
        if self.gold_loan.ltv_percent <= 0.0 || self.gold_loan.ltv_percent > 100.0 {
            errors.push("LTV must be between 0 and 100".to_string());
        }

        // Validate product config
        if self.product.variants.is_empty() {
            errors.push("At least one product variant required".to_string());
        }

        // Validate prompts
        if self.prompts.system_prompt.agent_name.is_empty() {
            errors.push("Agent name required in prompts".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Merge with another config (other takes precedence for non-default values)
    pub fn merge(&mut self, other: &DomainConfig) {
        // Merge simple fields
        if other.domain != default_domain() {
            self.domain = other.domain.clone();
        }
        if other.version != default_version() {
            self.version = other.version.clone();
        }

        // Gold loan config - merge rates if different from default
        let default_gold = GoldLoanConfig::default();
        if other.gold_loan.kotak_interest_rate != default_gold.kotak_interest_rate {
            self.gold_loan.kotak_interest_rate = other.gold_loan.kotak_interest_rate;
        }
        if other.gold_loan.gold_price_per_gram != default_gold.gold_price_per_gram {
            self.gold_loan.gold_price_per_gram = other.gold_loan.gold_price_per_gram;
        }

        // Add branches from other
        for branch in &other.branches.branches {
            if !self.branches.branches.iter().any(|b| b.id == branch.id) {
                self.branches.branches.push(branch.clone());
            }
        }

        // Add product variants from other
        for variant in &other.product.variants {
            if !self.product.variants.iter().any(|v| v.id == variant.id) {
                self.product.variants.push(variant.clone());
            }
        }

        // Add competitors from other
        for (id, competitor) in &other.competitors.competitors {
            self.competitors.competitors.entry(id.clone()).or_insert(competitor.clone());
        }
    }
}

/// Domain configuration manager with hot-reload support
pub struct DomainConfigManager {
    /// Current configuration
    config: Arc<RwLock<DomainConfig>>,
    /// Config file path (if loaded from file)
    config_path: Option<String>,
}

impl DomainConfigManager {
    /// Create new manager with default config
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(DomainConfig::default())),
            config_path: None,
        }
    }

    /// Create manager with config
    pub fn with_config(config: DomainConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            config_path: None,
        }
    }

    /// Load from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let config = if path_str.ends_with(".yaml") || path_str.ends_with(".yml") {
            DomainConfig::from_yaml_file(&path)?
        } else {
            DomainConfig::from_json_file(&path)?
        };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path: Some(path_str),
        })
    }

    /// Reload configuration from file
    pub fn reload(&self) -> Result<(), ConfigError> {
        let path = self.config_path.as_ref()
            .ok_or_else(|| ConfigError::FileNotFound("No config path set".to_string()))?;

        let new_config = if path.ends_with(".yaml") || path.ends_with(".yml") {
            DomainConfig::from_yaml_file(path)?
        } else {
            DomainConfig::from_json_file(path)?
        };

        *self.config.write() = new_config;
        Ok(())
    }

    /// Get current configuration
    pub fn get(&self) -> DomainConfig {
        self.config.read().clone()
    }

    /// Get configuration reference
    pub fn config(&self) -> Arc<RwLock<DomainConfig>> {
        Arc::clone(&self.config)
    }

    /// Update configuration
    pub fn update(&self, config: DomainConfig) {
        *self.config.write() = config;
    }

    /// Get gold loan config
    pub fn gold_loan(&self) -> GoldLoanConfig {
        self.config.read().gold_loan.clone()
    }

    /// Get branch config
    pub fn branches(&self) -> BranchConfig {
        self.config.read().branches.clone()
    }

    /// Get product config
    pub fn product(&self) -> ProductConfig {
        self.config.read().product.clone()
    }

    /// Get competitor config
    pub fn competitors(&self) -> CompetitorConfig {
        self.config.read().competitors.clone()
    }

    /// Get prompts
    pub fn prompts(&self) -> PromptTemplates {
        self.config.read().prompts.clone()
    }

    /// Get current gold price
    pub fn gold_price(&self) -> f64 {
        self.config.read().gold_loan.gold_price_per_gram
    }

    /// Update gold price (real-time update)
    pub fn update_gold_price(&self, price: f64) {
        self.config.write().gold_loan.gold_price_per_gram = price;
    }

    /// Get interest rate for loan amount
    pub fn get_interest_rate(&self, loan_amount: f64) -> f64 {
        self.config.read().gold_loan.get_tiered_rate(loan_amount)
    }

    /// Calculate savings vs competitor
    pub fn calculate_competitor_savings(
        &self,
        competitor: &str,
        loan_amount: f64,
    ) -> Option<crate::competitor::MonthlySavings> {
        let config = self.config.read();
        let kotak_rate = config.gold_loan.get_tiered_rate(loan_amount);
        config.competitors.calculate_savings(competitor, loan_amount, kotak_rate)
    }

    /// Find nearby branches
    pub fn find_branches_by_city(&self, city: &str) -> Vec<crate::branch::Branch> {
        self.config.read().branches.find_by_city(city).into_iter().cloned().collect()
    }

    /// Check doorstep service availability
    pub fn doorstep_available(&self, city: &str) -> bool {
        self.config.read().branches.doorstep_available(city)
    }

    /// Get system prompt for stage
    pub fn get_system_prompt(&self, stage: Option<&str>, customer_name: Option<&str>) -> String {
        self.config.read().prompts.build_system_prompt(stage, customer_name)
    }

    /// Get greeting for current time
    pub fn get_greeting(&self, hour: u32, customer_name: Option<&str>) -> String {
        let config = self.config.read();
        let agent_name = &config.prompts.system_prompt.agent_name;
        config.prompts.get_greeting(hour, agent_name, customer_name)
    }
}

impl Default for DomainConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global domain configuration instance
static DOMAIN_CONFIG: once_cell::sync::Lazy<DomainConfigManager> =
    once_cell::sync::Lazy::new(|| DomainConfigManager::new());

/// Get global domain configuration
pub fn domain_config() -> &'static DomainConfigManager {
    &DOMAIN_CONFIG
}

/// Initialize global domain configuration from file
pub fn init_domain_config(path: impl AsRef<Path>) -> Result<(), ConfigError> {
    let manager = DomainConfigManager::from_file(path)?;
    DOMAIN_CONFIG.update(manager.get());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DomainConfig::default();
        assert_eq!(config.domain, "gold_loan");
        assert!(!config.product.variants.is_empty());
    }

    #[test]
    fn test_validation() {
        let config = DomainConfig::default();
        assert!(config.validate().is_ok());

        let mut bad_config = DomainConfig::default();
        bad_config.gold_loan.kotak_interest_rate = -1.0;
        assert!(bad_config.validate().is_err());
    }

    #[test]
    fn test_manager() {
        let manager = DomainConfigManager::new();

        assert!(manager.gold_price() > 0.0);
        assert!(!manager.product().variants.is_empty());
    }

    #[test]
    fn test_update_gold_price() {
        let manager = DomainConfigManager::new();
        let original = manager.gold_price();

        manager.update_gold_price(8000.0);
        assert_eq!(manager.gold_price(), 8000.0);

        manager.update_gold_price(original);
    }

    #[test]
    fn test_get_interest_rate() {
        let manager = DomainConfigManager::new();

        // Small loan gets tier 1 rate
        let rate1 = manager.get_interest_rate(50_000.0);
        // Large loan gets tier 3 rate
        let rate3 = manager.get_interest_rate(1_000_000.0);

        assert!(rate3 < rate1);
    }

    #[test]
    fn test_competitor_savings() {
        let manager = DomainConfigManager::new();
        let savings = manager.calculate_competitor_savings("muthoot", 100_000.0);

        assert!(savings.is_some());
        let savings = savings.unwrap();
        assert!(savings.monthly_savings > 0.0);
    }

    #[test]
    fn test_doorstep_availability() {
        let manager = DomainConfigManager::new();

        assert!(manager.doorstep_available("Mumbai"));
        assert!(!manager.doorstep_available("SmallVillage"));
    }

    #[test]
    fn test_system_prompt() {
        let manager = DomainConfigManager::new();
        let prompt = manager.get_system_prompt(Some("discovery"), Some("Raj"));

        assert!(prompt.contains("discovery"));
        assert!(prompt.contains("Raj"));
    }

    #[test]
    fn test_greeting() {
        let manager = DomainConfigManager::new();

        let morning = manager.get_greeting(9, Some("Raj"));
        assert!(morning.contains("morning"));

        let evening = manager.get_greeting(19, None);
        assert!(evening.contains("evening"));
    }

    #[test]
    fn test_merge() {
        let mut base = DomainConfig::default();
        let mut overlay = DomainConfig::default();
        overlay.gold_loan.gold_price_per_gram = 8500.0;
        overlay.version = "2.0.0".to_string();

        base.merge(&overlay);

        assert_eq!(base.gold_loan.gold_price_per_gram, 8500.0);
        assert_eq!(base.version, "2.0.0");
    }
}
