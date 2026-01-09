//! Domain Tool Implementations
//!
//! MCP-compatible tools for voice agent.
//! All tool schemas are config-driven via YAML files.
//!
//! Each tool is in its own module for better maintainability.

mod appointment;
mod branch_locator;
mod competitor;
mod document_checklist;
mod eligibility;
mod escalate;
mod lead_capture;
mod price;
mod savings;
mod sms;

// Re-export all tools
pub use appointment::AppointmentSchedulerTool;
pub use branch_locator::BranchLocatorTool;
pub use competitor::CompetitorComparisonTool;
pub use document_checklist::DocumentChecklistTool;
pub use eligibility::EligibilityCheckTool;
pub use escalate::EscalateToHumanTool;
pub use lead_capture::LeadCaptureTool;
pub use price::GetPriceTool;
/// Legacy alias for backwards compatibility
pub type GetGoldPriceTool = GetPriceTool;
pub use savings::SavingsCalculatorTool;
pub use sms::SendSmsTool;
