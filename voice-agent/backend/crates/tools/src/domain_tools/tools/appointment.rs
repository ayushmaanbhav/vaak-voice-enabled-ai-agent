//! Appointment Scheduler Tool
//!
//! Schedule branch visit appointments.
//! P16 FIX: Purposes and time slots are now config-driven via ToolsDomainView.

use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use serde_json::{json, Value};
use std::sync::Arc;

use voice_agent_config::ToolsDomainView;

use crate::integrations::{
    Appointment, AppointmentPurpose, AppointmentStatus, CalendarIntegration,
};
use crate::mcp::{InputSchema, PropertySchema, Tool, ToolError, ToolOutput, ToolSchema};

/// Appointment scheduler tool
///
/// P16 FIX: Now uses ToolsDomainView for:
/// - Time slots from config
/// - Purposes from config (no hardcoded domain-specific terms)
pub struct AppointmentSchedulerTool {
    calendar: Option<Arc<dyn CalendarIntegration>>,
    /// P16 FIX: Domain view for config-driven values
    view: Option<Arc<ToolsDomainView>>,
}

impl AppointmentSchedulerTool {
    pub fn new() -> Self {
        Self {
            calendar: None,
            view: None,
        }
    }

    /// P16 FIX: Create with domain view for config-driven values
    pub fn with_view(view: Arc<ToolsDomainView>) -> Self {
        Self {
            calendar: None,
            view: Some(view),
        }
    }

    pub fn with_calendar(calendar: Arc<dyn CalendarIntegration>) -> Self {
        Self {
            calendar: Some(calendar),
            view: None,
        }
    }

    /// P16 FIX: Create with both calendar and domain view
    pub fn with_calendar_and_view(
        calendar: Arc<dyn CalendarIntegration>,
        view: Arc<ToolsDomainView>,
    ) -> Self {
        Self {
            calendar: Some(calendar),
            view: Some(view),
        }
    }

    /// Get time slots from config or defaults
    fn time_slots(&self) -> Vec<String> {
        if let Some(ref view) = self.view {
            if let Some(tool) = view.get_tool("schedule_appointment") {
                // Find the preferred_time parameter and get its enum values
                for param in &tool.parameters {
                    if param.name == "preferred_time" {
                        if let Some(ref values) = param.enum_values {
                            if !values.is_empty() {
                                return values.clone();
                            }
                        }
                    }
                }
            }
        }
        // Default time slots (generic)
        vec![
            "10:00 AM".to_string(),
            "11:00 AM".to_string(),
            "12:00 PM".to_string(),
            "2:00 PM".to_string(),
            "3:00 PM".to_string(),
            "4:00 PM".to_string(),
            "5:00 PM".to_string(),
        ]
    }

    /// Get appointment purposes from config or defaults
    fn purposes(&self) -> Vec<String> {
        if let Some(ref view) = self.view {
            if let Some(tool) = view.get_tool("schedule_appointment") {
                // Find the purpose parameter and get its enum values
                for param in &tool.parameters {
                    if param.name == "purpose" {
                        if let Some(ref values) = param.enum_values {
                            if !values.is_empty() {
                                return values.clone();
                            }
                        }
                    }
                }
            }
        }
        // Default purposes (generic, not domain-specific)
        vec![
            "New Application".to_string(),
            "Transfer".to_string(),
            "Top-up".to_string(),
            "Closure".to_string(),
            "Consultation".to_string(),
        ]
    }

    /// Get default purpose
    fn default_purpose(&self) -> String {
        self.purposes().first().cloned().unwrap_or_else(|| "Consultation".to_string())
    }

    /// Get tool description from config or default
    fn tool_description(&self) -> &str {
        // Can't return borrowed &str from config, so use static description
        "Schedule a branch visit appointment"
    }

    /// Get product name from config or default
    fn product_name(&self) -> String {
        self.view.as_ref()
            .map(|v| v.product_name().to_string())
            .unwrap_or_else(|| "Service".to_string())
    }
}

#[async_trait]
impl Tool for AppointmentSchedulerTool {
    fn name(&self) -> &str {
        "schedule_appointment"
    }

    fn description(&self) -> &str {
        self.tool_description()
    }

    fn schema(&self) -> ToolSchema {
        // P16 FIX: Get time slots and purposes from config
        let time_slots = self.time_slots();
        let purposes = self.purposes();

        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: InputSchema::object()
                .property(
                    "customer_name",
                    PropertySchema::string("Customer's name"),
                    true,
                )
                .property(
                    "phone_number",
                    PropertySchema::string("Contact number"),
                    true,
                )
                .property(
                    "branch_id",
                    PropertySchema::string("Branch ID or location"),
                    true,
                )
                .property(
                    "preferred_date",
                    PropertySchema::string("Preferred date (YYYY-MM-DD)"),
                    true,
                )
                .property(
                    "preferred_time",
                    PropertySchema::enum_type("Preferred time slot", time_slots),
                    true,
                )
                .property(
                    "purpose",
                    PropertySchema::enum_type("Purpose of visit", purposes),
                    false,
                ),
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput, ToolError> {
        let name = input
            .get("customer_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("customer_name is required"))?;

        let phone = input
            .get("phone_number")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("phone_number is required"))?;

        let branch = input
            .get("branch_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("branch_id is required"))?;

        let date_str = input
            .get("preferred_date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("preferred_date is required"))?;

        let parsed_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(date_str, "%d-%m-%Y"))
            .or_else(|_| NaiveDate::parse_from_str(date_str, "%d/%m/%Y"))
            .map_err(|_| {
                ToolError::invalid_params(
                    "preferred_date must be in format YYYY-MM-DD, DD-MM-YYYY, or DD/MM/YYYY",
                )
            })?;

        let today = Utc::now().date_naive();
        if parsed_date < today {
            return Err(ToolError::invalid_params(
                "preferred_date cannot be in the past",
            ));
        }

        let date = parsed_date.format("%Y-%m-%d").to_string();

        let time = input
            .get("preferred_time")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::invalid_params("preferred_time is required"))?;

        let default_purpose = self.default_purpose();
        let purpose_str = input
            .get("purpose")
            .and_then(|v| v.as_str())
            .unwrap_or(&default_purpose);

        // P16 FIX: Use string-based purpose directly (config-driven)
        let purpose = AppointmentPurpose::new(purpose_str);

        let product = self.product_name();

        if let Some(ref calendar) = self.calendar {
            let appointment = Appointment {
                id: None,
                customer_name: name.to_string(),
                customer_phone: phone.to_string(),
                branch_id: branch.to_string(),
                date: date.clone(),
                time_slot: time.to_string(),
                purpose,
                notes: None,
                status: AppointmentStatus::Scheduled,
                confirmation_sent: false,
            };

            match calendar.schedule_appointment(appointment).await {
                Ok(appointment_id) => {
                    let confirmation_sent =
                        calendar.send_confirmation(&appointment_id).await.is_ok();

                    let result = json!({
                        "success": true,
                        "appointment_id": appointment_id,
                        "customer_name": name,
                        "phone_number": phone,
                        "branch_id": branch,
                        "date": date,
                        "time": time,
                        "purpose": purpose_str,
                        "confirmation_sent": confirmation_sent,
                        "calendar_integrated": true,
                        "status": "pending_confirmation",
                        "confirmation_method": "agent_will_call_to_confirm",
                        "next_action": "Agent will call customer to confirm appointment",
                        "message": if confirmation_sent {
                            format!(
                                "{} appointment scheduled for {} on {} at {}. Confirmation sent to {}.",
                                product, name, date, time, phone
                            )
                        } else {
                            format!(
                                "{} appointment scheduled for {} on {} at {}. Our team will call to confirm.",
                                product, name, date, time
                            )
                        }
                    });
                    return Ok(ToolOutput::json(result));
                }
                Err(e) => {
                    tracing::warn!("Calendar integration failed, falling back to local: {}", e);
                }
            }
        }

        let appointment_id = format!(
            "APT{}",
            uuid::Uuid::new_v4().to_string()[..8].to_uppercase()
        );

        let result = json!({
            "success": true,
            "appointment_id": appointment_id,
            "customer_name": name,
            "phone_number": phone,
            "branch_id": branch,
            "date": date,
            "time": time,
            "purpose": purpose_str,
            "confirmation_sent": false,
            "calendar_integrated": false,
            "status": "pending_confirmation",
            "confirmation_method": "agent_will_call_to_confirm",
            "next_action": "Agent will call customer to confirm appointment",
            "message": format!(
                "{} appointment scheduled for {} on {} at {}. Our team will call to confirm.",
                product, name, date, time
            )
        });

        Ok(ToolOutput::json(result))
    }

    fn timeout_secs(&self) -> u64 {
        60
    }
}

impl Default for AppointmentSchedulerTool {
    fn default() -> Self {
        Self::new()
    }
}
