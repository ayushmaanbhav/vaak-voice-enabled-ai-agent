//! Document Checklist Tool
//!
//! Get the list of documents required for application.
//! P16 FIX: Document requirements are now config-driven via ToolsDomainView.

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use voice_agent_config::ToolsDomainView;

use crate::mcp::{InputSchema, PropertySchema, Tool, ToolError, ToolOutput, ToolSchema};

/// Document checklist tool
///
/// P16 FIX: Now uses ToolsDomainView for:
/// - Document requirements from config (no hardcoded domain-specific content)
/// - Service types from config (e.g., new_service, top_up, transfer)
/// - Customer types from config (e.g., individual, business)
/// - Important notes from config
pub struct DocumentChecklistTool {
    /// P16 FIX: Domain view for config-driven document requirements
    view: Option<Arc<ToolsDomainView>>,
}

impl DocumentChecklistTool {
    /// Create without domain config (uses generic fallback)
    pub fn new() -> Self {
        Self { view: None }
    }

    /// P16 FIX: Create with domain view for config-driven documents
    pub fn with_view(view: Arc<ToolsDomainView>) -> Self {
        Self { view: Some(view) }
    }

    /// Get service types from config or defaults
    fn service_types(&self) -> Vec<String> {
        if let Some(ref view) = self.view {
            let types = view.document_service_types();
            if !types.is_empty() {
                return types.iter().map(|s| s.to_string()).collect();
            }
        }
        // Default service types (generic, not domain-specific)
        vec![
            "new_service".to_string(),
            "top_up".to_string(),
            "transfer".to_string(),
            "renewal".to_string(),
        ]
    }

    /// Get customer types from config or defaults
    fn customer_types(&self) -> Vec<String> {
        if let Some(ref view) = self.view {
            let types = view.document_customer_types();
            if !types.is_empty() {
                return types.iter().map(|s| s.to_string()).collect();
            }
        }
        // Default customer types (generic)
        vec![
            "individual".to_string(),
            "self_employed".to_string(),
            "business".to_string(),
        ]
    }

    /// Convert DocumentEntry to JSON Value
    fn entry_to_json(entry: &voice_agent_config::domain::DocumentEntry) -> Value {
        let mut obj = json!({
            "document": entry.document,
        });

        if !entry.accepted.is_empty() {
            obj["accepted"] = json!(entry.accepted);
        }
        if let Some(copies) = entry.copies {
            obj["copies"] = json!(copies);
        }
        if let Some(ref notes) = entry.notes {
            obj["notes"] = json!(notes);
        }

        obj
    }

    /// Build document list from config or fallback
    fn build_document_response(
        &self,
        service_type: &str,
        customer_type: &str,
        is_existing_customer: bool,
    ) -> Value {
        let product_name = self.view.as_ref()
            .map(|v| v.product_name())
            .unwrap_or("Service");

        // Try to get documents from config
        if let Some(ref view) = self.view {
            if view.has_document_config() {
                // Get documents from config
                let mandatory: Vec<Value> = view.mandatory_documents()
                    .iter()
                    .map(Self::entry_to_json)
                    .collect();

                let domain_specific: Vec<Value> = view.domain_specific_documents()
                    .iter()
                    .map(Self::entry_to_json)
                    .collect();

                let service_specific: Vec<Value> = view.documents_for_service_type(service_type)
                    .iter()
                    .map(Self::entry_to_json)
                    .collect();

                let customer_specific: Vec<Value> = view.documents_for_customer_type(customer_type)
                    .iter()
                    .map(Self::entry_to_json)
                    .collect();

                let existing_note = if is_existing_customer {
                    view.existing_customer_note()
                } else {
                    view.new_customer_note()
                };

                let general_notes = view.document_general_notes();
                let mut all_notes = vec![existing_note.to_string()];
                all_notes.extend(general_notes.iter().cloned());

                let total_docs = mandatory.len() + domain_specific.len()
                    + service_specific.len() + customer_specific.len();

                return json!({
                    "service_type": service_type,
                    "customer_type": customer_type,
                    "existing_customer": is_existing_customer,
                    "mandatory_documents": mandatory,
                    "domain_specific": domain_specific,
                    "service_type_documents": service_specific,
                    "customer_specific_documents": customer_specific,
                    "total_documents": total_docs,
                    "important_notes": all_notes,
                    "message": format!(
                        "For a {} {}, you'll need {} documents. Please bring all required documents and items.",
                        service_type.replace("_", " "),
                        product_name.to_lowercase(),
                        total_docs
                    )
                });
            }
        }

        // Fallback to generic documents (no domain-specific content)
        let mandatory_docs = vec![
            json!({
                "document": "Valid Photo ID",
                "accepted": ["Aadhaar Card", "PAN Card", "Passport", "Voter ID", "Driving License"],
                "copies": 1,
                "notes": "Original required for verification"
            }),
            json!({
                "document": "Address Proof",
                "accepted": ["Aadhaar Card", "Utility Bill (last 3 months)", "Bank Statement", "Rent Agreement"],
                "copies": 1,
                "notes": "Should match current residence"
            }),
            json!({
                "document": "Passport Size Photographs",
                "copies": 2,
                "notes": "Recent photographs (within 6 months)"
            }),
            json!({
                "document": "PAN Card",
                "copies": 1,
                "notes": "Mandatory for transactions above threshold"
            }),
        ];

        let service_specific: Vec<Value> = match service_type {
            "transfer" | "balance_transfer" => vec![
                json!({
                    "document": "Existing Statement",
                    "notes": "From current provider showing outstanding amount"
                }),
                json!({
                    "document": "Account Details",
                    "notes": "Account number and provider details"
                }),
            ],
            "top_up" => vec![json!({
                "document": "Existing Account Details",
                "notes": "Account number for top-up"
            })],
            "renewal" => vec![json!({
                "document": "Previous Account Details",
                "notes": "Account number for renewal"
            })],
            _ => vec![],
        };

        let customer_specific: Vec<Value> = match customer_type {
            "self_employed" | "business" => vec![json!({
                "document": "Business Proof",
                "accepted": ["GST Registration", "Shop & Establishment Certificate", "Trade License"],
                "notes": "Any one document for business verification"
            })],
            _ => vec![],
        };

        let existing_customer_note = if is_existing_customer {
            "As an existing customer, some documents may already be on file. Please bring originals for verification."
        } else {
            "Please bring original documents along with photocopies."
        };

        let total_docs = mandatory_docs.len() + service_specific.len() + customer_specific.len();

        json!({
            "service_type": service_type,
            "customer_type": customer_type,
            "existing_customer": is_existing_customer,
            "mandatory_documents": mandatory_docs,
            "service_type_documents": service_specific,
            "customer_specific_documents": customer_specific,
            "total_documents": total_docs,
            "important_notes": [
                existing_customer_note,
                "Original documents are required for verification.",
                "Processing time varies based on document verification."
            ],
            "message": format!(
                "For a {} {}, you'll need {} documents. Key documents: Valid ID, Address Proof, PAN Card.",
                service_type.replace("_", " "),
                product_name.to_lowercase(),
                total_docs
            )
        })
    }
}

#[async_trait]
impl Tool for DocumentChecklistTool {
    fn name(&self) -> &str {
        "get_document_checklist"
    }

    fn description(&self) -> &str {
        // P16 FIX: Use config description if available
        if let Some(ref view) = self.view {
            let desc = view.document_tool_description();
            if !desc.is_empty() {
                // Can't return borrowed &str from config, so use a generic description
                return "Get the list of documents required based on service type and customer category";
            }
        }
        "Get the list of documents required based on service type and customer category"
    }

    fn schema(&self) -> ToolSchema {
        // P16 FIX: Get service and customer types from config
        let service_types = self.service_types();
        let customer_types = self.customer_types();

        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: InputSchema::object()
                .property(
                    "service_type",
                    PropertySchema::enum_type(
                        "Type of service",
                        service_types,
                    ),
                    true,
                )
                .property(
                    "customer_type",
                    PropertySchema::enum_type(
                        "Customer category",
                        customer_types,
                    ),
                    false,
                )
                .property(
                    "existing_customer",
                    PropertySchema::boolean("Is an existing customer"),
                    false,
                ),
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput, ToolError> {
        let service_type = input
            .get("service_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("service_type is required"))?;

        let customer_type = input
            .get("customer_type")
            .and_then(|v| v.as_str())
            .unwrap_or("individual");

        let existing_customer = input
            .get("existing_customer")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // P16 FIX: Build response from config
        let result = self.build_document_response(service_type, customer_type, existing_customer);

        Ok(ToolOutput::json(result))
    }
}

impl Default for DocumentChecklistTool {
    fn default() -> Self {
        Self::new()
    }
}
