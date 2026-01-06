//! Gold Loan Domain Tools
//!
//! Specific tools for the gold loan voice agent.
//!
//! This module is organized into:
//! - `utils`: Financial calculations (EMI, interest)
//! - `branches`: Branch data management
//! - `tools`: MCP tool implementations

mod branches;
mod tools;
mod utils;

// Re-export utilities
pub use utils::{calculate_emi, calculate_total_interest};

// Re-export branch management
pub use branches::{
    get_branches, get_mock_branches, load_branches_from_file, reload_branches, BranchData,
};

// Re-export all tools
pub use tools::{
    AppointmentSchedulerTool, BranchLocatorTool, CompetitorComparisonTool, DocumentChecklistTool,
    EligibilityCheckTool, EscalateToHumanTool, GetGoldPriceTool, LeadCaptureTool,
    SavingsCalculatorTool, SendSmsTool,
};
