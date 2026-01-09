//! Domain Tools
//!
//! Domain-agnostic tools for voice agent.
//! All tool schemas are config-driven via YAML files.
//!
//! This module is organized into:
//! - `utils`: Financial calculations (EMI, interest)
//! - `locations`: Location/branch data management
//! - `tools`: MCP tool implementations

mod locations;
mod tools;
mod utils;

// Re-export utilities
pub use utils::{calculate_emi, calculate_total_interest};

// Re-export location management
pub use locations::{
    get_branches, get_mock_branches, load_branches_from_file, reload_branches, BranchData,
};

// Re-export all tools
pub use tools::{
    AppointmentSchedulerTool, BranchLocatorTool, CompetitorComparisonTool, DocumentChecklistTool,
    EligibilityCheckTool, EscalateToHumanTool, GetGoldPriceTool, LeadCaptureTool,
    SavingsCalculatorTool, SendSmsTool,
};
