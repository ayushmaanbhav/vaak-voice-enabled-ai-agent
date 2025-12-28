//! MCP Tools for Gold Loan Voice Agent
//!
//! Implements MCP (Model Context Protocol) compatible tool interface
//! with domain-specific tools for gold loan operations.

pub mod mcp;
pub mod registry;
pub mod gold_loan;

pub use mcp::{Tool, ToolInput, ToolOutput, ToolSchema, ToolError};
pub use registry::{ToolRegistry, ToolExecutor};
pub use gold_loan::{
    EligibilityCheckTool,
    SavingsCalculatorTool,
    LeadCaptureTool,
    AppointmentSchedulerTool,
    BranchLocatorTool,
};

/// P2 FIX: Removed redundant ToolsError enum.
/// Use mcp::ToolError for tool execution errors instead.
/// This unifies error handling across the tools crate.

impl From<ToolError> for voice_agent_core::Error {
    fn from(err: ToolError) -> Self {
        voice_agent_core::Error::Tool(voice_agent_core::error::ToolError::ExecutionFailed(err.to_string()))
    }
}
