//! Send SMS Tool
//!
//! Send SMS messages to customers for appointment confirmations, follow-ups, etc.
//! P16 FIX: Templates are now config-driven via ToolsDomainView.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

use voice_agent_config::ToolsDomainView;

use crate::mcp::{InputSchema, PropertySchema, Tool, ToolError, ToolOutput, ToolSchema};

/// Send SMS tool
///
/// P16 FIX: Now uses ToolsDomainView for:
/// - SMS templates from config (no hardcoded domain-specific text)
/// - Brand placeholders ({brand.company_name}, {brand.helpline})
/// - Dynamic template types from config
pub struct SendSmsTool {
    sms_service: Option<Arc<dyn voice_agent_persistence::SmsService>>,
    /// P16 FIX: Domain view for config-driven templates
    view: Option<Arc<ToolsDomainView>>,
}

impl SendSmsTool {
    /// Create without domain config (uses fallback templates)
    pub fn new() -> Self {
        Self {
            sms_service: None,
            view: None,
        }
    }

    /// P16 FIX: Create with domain view for config-driven templates
    pub fn with_view(view: Arc<ToolsDomainView>) -> Self {
        Self {
            sms_service: None,
            view: Some(view),
        }
    }

    pub fn with_sms_service(service: Arc<dyn voice_agent_persistence::SmsService>) -> Self {
        Self {
            sms_service: Some(service),
            view: None,
        }
    }

    /// P16 FIX: Create with both SMS service and domain view
    pub fn with_service_and_view(
        service: Arc<dyn voice_agent_persistence::SmsService>,
        view: Arc<ToolsDomainView>,
    ) -> Self {
        Self {
            sms_service: Some(service),
            view: Some(view),
        }
    }

    /// P16 FIX: Build SMS message from config templates or fallback
    fn build_message(
        &self,
        msg_type: &str,
        customer_name: &str,
        details: Option<&str>,
        custom_message: Option<&str>,
    ) -> String {
        // Build placeholder map
        let mut placeholders = HashMap::new();
        placeholders.insert("customer_name".to_string(), customer_name.to_string());

        if let Some(d) = details {
            // Parse details into date/time/branch if available
            placeholders.insert("date".to_string(), d.to_string());
            placeholders.insert("time".to_string(), d.to_string());
            placeholders.insert("branch".to_string(), d.to_string());
            placeholders.insert("appointment_details".to_string(), d.to_string());
        }

        // Try to get template from config
        if let Some(ref view) = self.view {
            // Add brand placeholders
            placeholders.insert("brand.company_name".to_string(), view.company_name().to_string());
            placeholders.insert("brand.bank_name".to_string(), view.company_name().to_string());
            placeholders.insert("brand.helpline".to_string(), view.helpline().to_string());
            placeholders.insert("rate".to_string(), format!("{:.1}", view.base_interest_rate()));

            // Try to get template from config (default to English)
            if let Some(message) = view.build_sms_message(msg_type, "en", &placeholders) {
                return message;
            }
        }

        // Fallback to generic templates (no domain-specific content)
        let company = self.view.as_ref()
            .map(|v| v.company_name())
            .unwrap_or("Service Provider");
        let helpline = self.view.as_ref()
            .map(|v| v.helpline())
            .unwrap_or("Customer Support");
        let product = self.view.as_ref()
            .map(|v| v.product_name())
            .unwrap_or("Service");

        match msg_type {
            "appointment_confirmation" => {
                let d = details.unwrap_or("scheduled date and time");
                format!(
                    "Dear {}, your {} appointment is confirmed for {}. Please bring required documents. - {}",
                    customer_name, product, d, company
                )
            }
            "appointment_reminder" => {
                let d = details.unwrap_or("tomorrow");
                format!(
                    "Reminder: Dear {}, your {} appointment is {}. Please bring required documents. - {}",
                    customer_name, product, d, company
                )
            }
            "follow_up" => {
                custom_message.map(|s| s.to_string()).unwrap_or_else(|| {
                    format!(
                        "Dear {}, thank you for your interest in {}. Contact us at {} for more details. - {}",
                        customer_name, product, helpline, company
                    )
                })
            }
            "welcome" => {
                format!(
                    "Welcome, {}! We're excited to help you with your {} needs. - {}",
                    customer_name, product, company
                )
            }
            "promotional" => {
                format!(
                    "Special Offer for {}: Get {} at competitive rates! Contact {}. T&C apply. - {}",
                    customer_name, product, helpline, company
                )
            }
            _ => format!(
                "Dear {}, thank you for contacting us. Call {} for assistance. - {}",
                customer_name, helpline, company
            ),
        }
    }

    /// Get available message types from config or defaults
    fn message_types(&self) -> Vec<String> {
        if let Some(ref view) = self.view {
            let types = view.sms_template_types();
            if !types.is_empty() {
                return types.iter().map(|s| s.to_string()).collect();
            }
        }
        // Default message types
        vec![
            "appointment_confirmation".to_string(),
            "appointment_reminder".to_string(),
            "follow_up".to_string(),
            "welcome".to_string(),
            "promotional".to_string(),
        ]
    }
}

#[async_trait]
impl Tool for SendSmsTool {
    fn name(&self) -> &str {
        "send_sms"
    }

    fn description(&self) -> &str {
        "Send an SMS message to the customer for appointment confirmations, follow-ups, or information sharing"
    }

    fn schema(&self) -> ToolSchema {
        // P16 FIX: Get message types from config
        let msg_types = self.message_types();

        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: InputSchema::object()
                .property(
                    "phone_number",
                    PropertySchema::string("10-digit mobile number"),
                    true,
                )
                .property(
                    "message_type",
                    PropertySchema::enum_type(
                        "Type of SMS message",
                        msg_types,
                    ),
                    true,
                )
                .property(
                    "customer_name",
                    PropertySchema::string("Customer name for personalization"),
                    false,
                )
                .property(
                    "custom_message",
                    PropertySchema::string("Custom message text (for follow_up type)"),
                    false,
                )
                .property(
                    "appointment_details",
                    PropertySchema::string("Appointment details (date, time, branch)"),
                    false,
                )
                .property(
                    "session_id",
                    PropertySchema::string("Session ID for tracking"),
                    false,
                ),
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput, ToolError> {
        let phone = input
            .get("phone_number")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("phone_number is required"))?;

        if phone.len() != 10 || !phone.chars().all(|c| c.is_ascii_digit()) {
            return Err(ToolError::invalid_params("phone_number must be 10 digits"));
        }

        let msg_type_str = input
            .get("message_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("message_type is required"))?;

        let customer_name = input
            .get("customer_name")
            .and_then(|v| v.as_str())
            .unwrap_or("Customer");

        let session_id = input.get("session_id").and_then(|v| v.as_str());

        let details = input.get("appointment_details").and_then(|v| v.as_str());
        let custom_message = input.get("custom_message").and_then(|v| v.as_str());

        let msg_type = match msg_type_str {
            "appointment_confirmation" => voice_agent_persistence::SmsType::AppointmentConfirmation,
            "appointment_reminder" => voice_agent_persistence::SmsType::AppointmentReminder,
            "follow_up" => voice_agent_persistence::SmsType::FollowUp,
            "welcome" => voice_agent_persistence::SmsType::Welcome,
            "promotional" => voice_agent_persistence::SmsType::Promotional,
            _ => voice_agent_persistence::SmsType::FollowUp,
        };

        // P16 FIX: Build message from config templates
        let message_text = self.build_message(msg_type_str, customer_name, details, custom_message);

        let (message_id, status, simulated) = if let Some(ref service) = self.sms_service {
            match service
                .send_sms(phone, &message_text, msg_type, session_id)
                .await
            {
                Ok(result) => (
                    result.message_id.to_string(),
                    result.status.as_str().to_string(),
                    result.simulated,
                ),
                Err(e) => {
                    tracing::warn!("SMS service failed: {}", e);
                    let id = format!(
                        "SMS{}",
                        uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
                    );
                    (id, "failed".to_string(), false)
                }
            }
        } else {
            let id = format!(
                "SMS{}",
                uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
            );
            (id, "simulated_not_sent".to_string(), true)
        };

        let success = status != "failed";

        let result = json!({
            "success": success,
            "message_id": message_id,
            "phone_number": phone,
            "message_type": msg_type_str,
            "message_text": message_text,
            "status": status,
            "simulated": simulated,
            "sent_at": if success { Some(Utc::now().to_rfc3339()) } else { None },
            "message": if success {
                format!("SMS {} to {}.", if simulated { "simulated" } else { "sent" }, phone)
            } else {
                "Failed to send SMS. Please try again.".to_string()
            }
        });

        Ok(ToolOutput::json(result))
    }

    fn timeout_secs(&self) -> u64 {
        30
    }
}

impl Default for SendSmsTool {
    fn default() -> Self {
        Self::new()
    }
}
