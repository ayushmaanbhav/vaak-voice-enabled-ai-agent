//! Branch Configuration
//!
//! Defines branch data loaded from YAML for the branch locator tool.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Branches configuration loaded from branches.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchesConfig {
    /// List of branch locations
    #[serde(default)]
    pub branches: Vec<BranchEntry>,
    /// Default search settings
    #[serde(default)]
    pub defaults: BranchDefaults,
}

impl Default for BranchesConfig {
    fn default() -> Self {
        Self {
            branches: Vec::new(),
            defaults: BranchDefaults::default(),
        }
    }
}

impl BranchesConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, BranchesConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            BranchesConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| BranchesConfigError::ParseError(e.to_string()))
    }

    /// Get branches by city
    pub fn find_by_city(&self, city: &str) -> Vec<&BranchEntry> {
        let city_lower = city.to_lowercase();
        self.branches
            .iter()
            .filter(|b| b.city.to_lowercase().contains(&city_lower))
            .collect()
    }

    /// Get branches by pincode
    pub fn find_by_pincode(&self, pincode: &str) -> Vec<&BranchEntry> {
        self.branches
            .iter()
            .filter(|b| b.pincode == pincode)
            .collect()
    }

    /// Get branch by ID
    pub fn get_branch(&self, branch_id: &str) -> Option<&BranchEntry> {
        self.branches.iter().find(|b| b.branch_id == branch_id)
    }

    /// Get branches with gold loan service
    pub fn gold_loan_branches(&self) -> Vec<&BranchEntry> {
        self.branches
            .iter()
            .filter(|b| b.gold_loan_available)
            .collect()
    }
}

/// Single branch entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchEntry {
    pub branch_id: String,
    pub name: String,
    pub city: String,
    pub area: String,
    pub address: String,
    #[serde(default)]
    pub pincode: String,
    pub phone: String,
    #[serde(default = "default_true")]
    pub gold_loan_available: bool,
    #[serde(default)]
    pub timing: String,
    #[serde(default)]
    pub facilities: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Default branch search settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchDefaults {
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_true")]
    pub filter_gold_loan_only: bool,
}

fn default_max_results() -> usize {
    5
}

fn default_sort_by() -> String {
    "distance".to_string()
}

impl Default for BranchDefaults {
    fn default() -> Self {
        Self {
            max_results: default_max_results(),
            sort_by: default_sort_by(),
            filter_gold_loan_only: true,
        }
    }
}

/// Errors when loading branches configuration
#[derive(Debug)]
pub enum BranchesConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for BranchesConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Branches config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse branches config: {}", err),
        }
    }
}

impl std::error::Error for BranchesConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branches_deserialization() {
        let yaml = r#"
branches:
  - branch_id: "KMBL001"
    name: "Test Branch"
    city: "Mumbai"
    area: "Andheri"
    address: "Test Address"
    pincode: "400058"
    phone: "022-12345678"
    gold_loan_available: true
    timing: "10:00 AM - 5:00 PM"
    facilities:
      - "Gold Valuation"
defaults:
  max_results: 10
"#;
        let config: BranchesConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.branches.len(), 1);
        assert_eq!(config.branches[0].branch_id, "KMBL001");
        assert_eq!(config.defaults.max_results, 10);
    }

    #[test]
    fn test_find_by_city() {
        let config = BranchesConfig {
            branches: vec![
                BranchEntry {
                    branch_id: "B1".to_string(),
                    name: "Branch 1".to_string(),
                    city: "Mumbai".to_string(),
                    area: "Andheri".to_string(),
                    address: "Address 1".to_string(),
                    pincode: "400001".to_string(),
                    phone: "1234567890".to_string(),
                    gold_loan_available: true,
                    timing: "10-5".to_string(),
                    facilities: vec![],
                },
                BranchEntry {
                    branch_id: "B2".to_string(),
                    name: "Branch 2".to_string(),
                    city: "Delhi".to_string(),
                    area: "CP".to_string(),
                    address: "Address 2".to_string(),
                    pincode: "110001".to_string(),
                    phone: "0987654321".to_string(),
                    gold_loan_available: true,
                    timing: "10-5".to_string(),
                    facilities: vec![],
                },
            ],
            defaults: BranchDefaults::default(),
        };

        let mumbai = config.find_by_city("mumbai");
        assert_eq!(mumbai.len(), 1);
        assert_eq!(mumbai[0].branch_id, "B1");
    }
}
