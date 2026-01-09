//! Document Requirements Configuration
//!
//! P16 FIX: Config-driven document checklist for domain-agnostic support.
//! Defines documents required for different service types and customer categories.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Document requirements configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentsConfig {
    /// Available service types (e.g., new_loan, top_up)
    #[serde(default)]
    pub service_types: Vec<ServiceTypeEntry>,

    /// Available customer types (e.g., individual, business)
    #[serde(default)]
    pub customer_types: Vec<CustomerTypeEntry>,

    /// Mandatory documents for all applications
    #[serde(default)]
    pub mandatory_documents: Vec<DocumentEntry>,

    /// Domain-specific documents (e.g., gold items for gold loan)
    #[serde(default)]
    pub domain_specific_documents: Vec<DocumentEntry>,

    /// Additional documents by service type
    #[serde(default)]
    pub service_type_documents: HashMap<String, Vec<DocumentEntry>>,

    /// Additional documents by customer type
    #[serde(default)]
    pub customer_type_documents: HashMap<String, Vec<DocumentEntry>>,

    /// Important notes
    #[serde(default)]
    pub important_notes: ImportantNotes,

    /// Tool configuration
    #[serde(default)]
    pub tool_config: DocumentToolConfig,
}

/// Service type entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTypeEntry {
    pub id: String,
    pub display_name: String,
}

/// Customer type entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerTypeEntry {
    pub id: String,
    pub display_name: String,
}

/// Document entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEntry {
    pub document: String,
    #[serde(default)]
    pub accepted: Vec<String>,
    #[serde(default)]
    pub copies: Option<u32>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Important notes configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImportantNotes {
    #[serde(default)]
    pub existing_customer: String,
    #[serde(default)]
    pub new_customer: String,
    #[serde(default)]
    pub general: Vec<String>,
}

/// Tool schema configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentToolConfig {
    pub name: String,
    pub description: String,
}

impl Default for DocumentToolConfig {
    fn default() -> Self {
        Self {
            name: "get_document_checklist".to_string(),
            description: "Get the list of documents required based on service type and customer category".to_string(),
        }
    }
}

impl DocumentsConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, DocumentsConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentsConfigError::FileNotFound(path.as_ref().display().to_string(), e.to_string())
        })?;

        serde_yaml::from_str(&content).map_err(|e| DocumentsConfigError::ParseError(e.to_string()))
    }

    /// Get service type IDs for tool schema enum
    pub fn service_type_ids(&self) -> Vec<&str> {
        self.service_types.iter().map(|s| s.id.as_str()).collect()
    }

    /// Get customer type IDs for tool schema enum
    pub fn customer_type_ids(&self) -> Vec<&str> {
        self.customer_types.iter().map(|c| c.id.as_str()).collect()
    }

    /// Get additional documents for a service type
    pub fn documents_for_service_type(&self, service_type: &str) -> &[DocumentEntry] {
        self.service_type_documents
            .get(service_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get additional documents for a customer type
    pub fn documents_for_customer_type(&self, customer_type: &str) -> &[DocumentEntry] {
        self.customer_type_documents
            .get(customer_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Get existing customer note
    pub fn existing_customer_note(&self) -> &str {
        if self.important_notes.existing_customer.is_empty() {
            "As an existing customer, some documents may already be on file."
        } else {
            &self.important_notes.existing_customer
        }
    }

    /// Get new customer note
    pub fn new_customer_note(&self) -> &str {
        if self.important_notes.new_customer.is_empty() {
            "Please bring original documents along with photocopies."
        } else {
            &self.important_notes.new_customer
        }
    }

    /// Get general notes
    pub fn general_notes(&self) -> &[String] {
        &self.important_notes.general
    }

    /// Get tool description
    pub fn tool_description(&self) -> &str {
        &self.tool_config.description
    }

    /// Check if config has any document definitions
    pub fn has_documents(&self) -> bool {
        !self.mandatory_documents.is_empty() || !self.domain_specific_documents.is_empty()
    }
}

/// Configuration error types
#[derive(Debug)]
pub enum DocumentsConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for DocumentsConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Documents config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse documents config: {}", err),
        }
    }
}

impl std::error::Error for DocumentsConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_document_entry() {
        let yaml = r#"
document: "Valid Photo ID"
accepted:
  - "Aadhaar Card"
  - "PAN Card"
copies: 1
notes: "Original required"
"#;
        let entry: DocumentEntry = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(entry.document, "Valid Photo ID");
        assert_eq!(entry.accepted.len(), 2);
        assert_eq!(entry.copies, Some(1));
    }

    #[test]
    fn test_default_documents_config() {
        let config = DocumentsConfig::default();
        assert!(config.mandatory_documents.is_empty());
        assert!(!config.has_documents());
    }

    #[test]
    fn test_service_type_ids() {
        let config = DocumentsConfig {
            service_types: vec![
                ServiceTypeEntry {
                    id: "new_loan".to_string(),
                    display_name: "New Loan".to_string(),
                },
                ServiceTypeEntry {
                    id: "top_up".to_string(),
                    display_name: "Top-up".to_string(),
                },
            ],
            ..Default::default()
        };
        let ids = config.service_type_ids();
        assert_eq!(ids, vec!["new_loan", "top_up"]);
    }
}
