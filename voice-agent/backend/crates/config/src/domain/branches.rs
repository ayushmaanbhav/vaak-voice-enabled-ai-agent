//! Location Configuration
//!
//! Defines service location data loaded from YAML for the location finder tool.
//! All location data comes from configuration - no hardcoded defaults.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Location configuration loaded from branches.yaml (or locations.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchesConfig {
    /// List of service locations
    #[serde(default, alias = "locations")]
    pub branches: Vec<BranchEntry>,
    /// Default search settings
    #[serde(default)]
    pub defaults: BranchDefaults,
    /// Mobile/doorstep service configuration
    #[serde(default)]
    pub doorstep_service: DoorstepServiceConfig,
}

impl Default for BranchesConfig {
    fn default() -> Self {
        Self {
            branches: Vec::new(),
            defaults: BranchDefaults::default(),
            doorstep_service: DoorstepServiceConfig::default(),
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

    /// Get locations by city
    pub fn find_by_city(&self, city: &str) -> Vec<&BranchEntry> {
        let city_lower = city.to_lowercase();
        self.branches
            .iter()
            .filter(|b| b.city.to_lowercase().contains(&city_lower))
            .collect()
    }

    /// Get locations by pincode
    pub fn find_by_pincode(&self, pincode: &str) -> Vec<&BranchEntry> {
        self.branches
            .iter()
            .filter(|b| b.pincode == pincode)
            .collect()
    }

    /// Get location by ID
    pub fn get_branch(&self, branch_id: &str) -> Option<&BranchEntry> {
        self.branches.iter().find(|b| b.branch_id == branch_id)
    }

    /// Get locations where service is available
    pub fn service_locations(&self) -> Vec<&BranchEntry> {
        self.branches
            .iter()
            .filter(|b| b.service_available)
            .collect()
    }

    /// Backwards compatibility alias
    pub fn gold_loan_branches(&self) -> Vec<&BranchEntry> {
        self.service_locations()
    }

    /// Check if mobile/doorstep service is available in a city
    pub fn doorstep_available(&self, city: &str) -> bool {
        if !self.doorstep_service.enabled {
            return false;
        }
        let city_lower = city.to_lowercase();
        self.doorstep_service
            .available_cities
            .iter()
            .any(|c| c.to_lowercase() == city_lower)
    }
}

/// Single location entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchEntry {
    /// Unique location identifier
    #[serde(alias = "location_id")]
    pub branch_id: String,
    /// Display name
    pub name: String,
    /// City
    pub city: String,
    /// Area/locality
    pub area: String,
    /// Full address
    pub address: String,
    /// Postal/PIN code
    #[serde(default)]
    pub pincode: String,
    /// Contact phone
    pub phone: String,
    /// Whether the primary service is available at this location
    #[serde(default = "default_true", alias = "gold_loan_available")]
    pub service_available: bool,
    /// Operating hours
    #[serde(default)]
    pub timing: String,
    /// List of available facilities/services
    #[serde(default)]
    pub facilities: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Default location search settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchDefaults {
    /// Maximum results to return
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    /// Sort order
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    /// Whether to filter to only show locations with service available
    #[serde(default = "default_true", alias = "filter_gold_loan_only")]
    pub filter_service_only: bool,
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
            filter_service_only: true,
        }
    }
}

/// Mobile/doorstep service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorstepServiceConfig {
    /// Whether mobile service is enabled
    #[serde(default)]
    pub enabled: bool,
    /// List of cities where mobile service is available (from config)
    #[serde(default)]
    pub available_cities: Vec<String>,
    /// Timing description for mobile service
    #[serde(default)]
    pub timing: String,
}

impl Default for DoorstepServiceConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            available_cities: Vec::new(),
            timing: String::new(),
        }
    }
}

/// Errors when loading location configuration
#[derive(Debug)]
pub enum BranchesConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for BranchesConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Location config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse location config: {}", err),
        }
    }
}

impl std::error::Error for BranchesConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locations_deserialization() {
        let yaml = r#"
branches:
  - branch_id: "LOC001"
    name: "Test Location"
    city: "Mumbai"
    area: "Andheri"
    address: "Test Address"
    pincode: "400058"
    phone: "022-12345678"
    service_available: true
    timing: "10:00 AM - 5:00 PM"
    facilities:
      - "Service A"
defaults:
  max_results: 10
"#;
        let config: BranchesConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.branches.len(), 1);
        assert_eq!(config.branches[0].branch_id, "LOC001");
        assert!(config.branches[0].service_available);
        assert_eq!(config.defaults.max_results, 10);
    }

    #[test]
    fn test_legacy_gold_loan_available_alias() {
        let yaml = r#"
branches:
  - branch_id: "LOC001"
    name: "Test Location"
    city: "Mumbai"
    area: "Andheri"
    address: "Test Address"
    phone: "022-12345678"
    gold_loan_available: true
"#;
        let config: BranchesConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.branches[0].service_available);
    }

    #[test]
    fn test_find_by_city() {
        let config = BranchesConfig {
            branches: vec![
                BranchEntry {
                    branch_id: "L1".to_string(),
                    name: "Location 1".to_string(),
                    city: "Mumbai".to_string(),
                    area: "Andheri".to_string(),
                    address: "Address 1".to_string(),
                    pincode: "400001".to_string(),
                    phone: "1234567890".to_string(),
                    service_available: true,
                    timing: "10-5".to_string(),
                    facilities: vec![],
                },
                BranchEntry {
                    branch_id: "L2".to_string(),
                    name: "Location 2".to_string(),
                    city: "Delhi".to_string(),
                    area: "CP".to_string(),
                    address: "Address 2".to_string(),
                    pincode: "110001".to_string(),
                    phone: "0987654321".to_string(),
                    service_available: true,
                    timing: "10-5".to_string(),
                    facilities: vec![],
                },
            ],
            defaults: BranchDefaults::default(),
            doorstep_service: DoorstepServiceConfig::default(),
        };

        let mumbai = config.find_by_city("mumbai");
        assert_eq!(mumbai.len(), 1);
        assert_eq!(mumbai[0].branch_id, "L1");
    }

    #[test]
    fn test_service_locations() {
        let config = BranchesConfig {
            branches: vec![
                BranchEntry {
                    branch_id: "L1".to_string(),
                    name: "Location 1".to_string(),
                    city: "Mumbai".to_string(),
                    area: "Andheri".to_string(),
                    address: "Address 1".to_string(),
                    pincode: "400001".to_string(),
                    phone: "1234567890".to_string(),
                    service_available: true,
                    timing: "10-5".to_string(),
                    facilities: vec![],
                },
                BranchEntry {
                    branch_id: "L2".to_string(),
                    name: "Location 2".to_string(),
                    city: "Delhi".to_string(),
                    area: "CP".to_string(),
                    address: "Address 2".to_string(),
                    pincode: "110001".to_string(),
                    phone: "0987654321".to_string(),
                    service_available: false, // Service not available
                    timing: "10-5".to_string(),
                    facilities: vec![],
                },
            ],
            defaults: BranchDefaults::default(),
            doorstep_service: DoorstepServiceConfig::default(),
        };

        let service_locs = config.service_locations();
        assert_eq!(service_locs.len(), 1);
        assert_eq!(service_locs[0].branch_id, "L1");
    }
}
