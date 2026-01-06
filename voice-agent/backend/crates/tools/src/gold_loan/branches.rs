//! Branch Data Management
//!
//! Handles loading and managing bank branch data for the gold loan tools.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::RwLock;

/// P0 FIX: Branch data structure for JSON loading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchData {
    pub branch_id: String,
    pub name: String,
    pub city: String,
    pub area: String,
    pub address: String,
    #[serde(default)]
    pub pincode: String,
    pub phone: String,
    pub gold_loan_available: bool,
    pub timing: String,
    #[serde(default)]
    pub facilities: Vec<String>,
}

/// Branch data file structure
#[derive(Debug, Deserialize)]
struct BranchDataFile {
    branches: Vec<BranchData>,
}

/// P0 FIX: Global branch data loaded from JSON
static BRANCH_DATA: Lazy<RwLock<Vec<BranchData>>> = Lazy::new(|| {
    // Try to load from default paths
    let default_paths = [
        "data/branches.json",
        "../data/branches.json",
        "../../data/branches.json",
        "./branches.json",
    ];

    for path in &default_paths {
        if let Ok(data) = load_branches_from_file(path) {
            tracing::info!("Loaded {} branches from {}", data.len(), path);
            return RwLock::new(data);
        }
    }

    // Fall back to embedded default data
    tracing::warn!("Could not load branches from file, using embedded defaults");
    RwLock::new(get_default_branches())
});

/// Load branches from a JSON file
pub fn load_branches_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<BranchData>, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let file: BranchDataFile = serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(file.branches)
}

/// Reload branches from a file (for runtime updates)
pub fn reload_branches<P: AsRef<Path>>(path: P) -> Result<usize, std::io::Error> {
    let branches = load_branches_from_file(path)?;
    let count = branches.len();
    *BRANCH_DATA.write().unwrap() = branches;
    Ok(count)
}

/// Get all loaded branches
pub fn get_branches() -> Vec<BranchData> {
    BRANCH_DATA.read().unwrap().clone()
}

/// Get default embedded branches (fallback)
fn get_default_branches() -> Vec<BranchData> {
    vec![
        BranchData {
            branch_id: "KMBL001".to_string(),
            name: "Kotak Mahindra Bank - Andheri West".to_string(),
            city: "Mumbai".to_string(),
            area: "Andheri West".to_string(),
            address: "Ground Floor, Kora Kendra, S.V. Road, Andheri West, Mumbai - 400058"
                .to_string(),
            pincode: "400058".to_string(),
            phone: "022-66006060".to_string(),
            gold_loan_available: true,
            timing: "10:00 AM - 5:00 PM (Mon-Sat)".to_string(),
            facilities: vec![
                "Gold Valuation".to_string(),
                "Same Day Disbursement".to_string(),
            ],
        },
        BranchData {
            branch_id: "KMBL101".to_string(),
            name: "Kotak Mahindra Bank - Connaught Place".to_string(),
            city: "Delhi".to_string(),
            area: "Connaught Place".to_string(),
            address: "M-Block, Connaught Place, New Delhi - 110001".to_string(),
            pincode: "110001".to_string(),
            phone: "011-66006060".to_string(),
            gold_loan_available: true,
            timing: "10:00 AM - 5:00 PM (Mon-Sat)".to_string(),
            facilities: vec![
                "Gold Valuation".to_string(),
                "Same Day Disbursement".to_string(),
            ],
        },
        BranchData {
            branch_id: "KMBL201".to_string(),
            name: "Kotak Mahindra Bank - MG Road".to_string(),
            city: "Bangalore".to_string(),
            area: "MG Road".to_string(),
            address: "Church Street, MG Road, Bangalore - 560001".to_string(),
            pincode: "560001".to_string(),
            phone: "080-66006060".to_string(),
            gold_loan_available: true,
            timing: "10:00 AM - 5:00 PM (Mon-Sat)".to_string(),
            facilities: vec![
                "Gold Valuation".to_string(),
                "Same Day Disbursement".to_string(),
            ],
        },
        BranchData {
            branch_id: "KMBL301".to_string(),
            name: "Kotak Mahindra Bank - T Nagar".to_string(),
            city: "Chennai".to_string(),
            area: "T Nagar".to_string(),
            address: "Usman Road, T Nagar, Chennai - 600017".to_string(),
            pincode: "600017".to_string(),
            phone: "044-66006060".to_string(),
            gold_loan_available: true,
            timing: "10:00 AM - 5:00 PM (Mon-Sat)".to_string(),
            facilities: vec![
                "Gold Valuation".to_string(),
                "Same Day Disbursement".to_string(),
            ],
        },
    ]
}

/// Get mock branches for testing purposes (used by tools)
pub fn get_mock_branches(
    city: Option<&str>,
    pincode: Option<&str>,
    area: Option<&str>,
) -> Vec<BranchData> {
    let all_branches = get_branches();

    all_branches
        .into_iter()
        .filter(|b| {
            if !b.gold_loan_available {
                return false;
            }

            // Filter by city
            if let Some(c) = city {
                if !b.city.to_lowercase().contains(&c.to_lowercase()) {
                    return false;
                }
            }

            // Filter by pincode
            if let Some(p) = pincode {
                if !b.pincode.starts_with(p) && !b.address.contains(p) {
                    return false;
                }
            }

            // Filter by area
            if let Some(a) = area {
                if !b.area.to_lowercase().contains(&a.to_lowercase())
                    && !b.address.to_lowercase().contains(&a.to_lowercase())
                {
                    return false;
                }
            }

            true
        })
        .collect()
}
