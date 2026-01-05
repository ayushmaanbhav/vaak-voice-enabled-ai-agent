//! Tool Registry
//!
//! Manages tool registration, discovery, and execution.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use crate::mcp::{Tool, ToolError, ToolOutput, ToolSchema};

/// Default timeout for tool execution (30 seconds)
const DEFAULT_TOOL_TIMEOUT_SECS: u64 = 30;

/// Tool executor trait
#[async_trait]
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool by name
    async fn execute(&self, name: &str, arguments: Value) -> Result<ToolOutput, ToolError>;

    /// List available tools
    fn list_tools(&self) -> Vec<ToolSchema>;

    /// Get tool schema by name
    fn get_tool(&self, name: &str) -> Option<ToolSchema>;
}

/// Tool registry
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        self.tools.insert(name, Arc::new(tool));
    }

    /// Register a boxed tool
    pub fn register_boxed(&mut self, tool: Arc<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    /// Get tool by name
    pub fn get(&self, name: &str) -> Option<&Arc<dyn Tool>> {
        self.tools.get(name)
    }

    /// Check if tool exists
    pub fn has(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Remove a tool
    pub fn remove(&mut self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.remove(name)
    }

    /// Get number of registered tools
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Get all tool names
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ToolExecutor for ToolRegistry {
    /// Execute a tool with timeout protection
    ///
    /// P1 FIX: Wraps tool execution in a timeout to prevent indefinite blocking.
    /// P5 FIX: Uses per-tool timeout instead of global default.
    async fn execute(&self, name: &str, arguments: Value) -> Result<ToolOutput, ToolError> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| ToolError::not_found(format!("Tool not found: {}", name)))?;

        // Validate input
        tool.validate(&arguments)?;

        // P5 FIX: Use per-tool timeout, falling back to default
        let timeout_secs = tool.timeout_secs();
        let timeout_duration = Duration::from_secs(timeout_secs);

        tracing::trace!(
            tool = name,
            timeout_secs = timeout_secs,
            "Executing tool with timeout"
        );

        match tokio::time::timeout(timeout_duration, tool.execute(arguments)).await {
            Ok(result) => result,
            Err(_elapsed) => Err(ToolError::timeout(name, timeout_secs)),
        }
    }

    fn list_tools(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|t| t.schema()).collect()
    }

    fn get_tool(&self, name: &str) -> Option<ToolSchema> {
        self.tools.get(name).map(|t| t.schema())
    }
}

/// Tool call result for conversation tracking
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// Tool name
    pub name: String,
    /// Input arguments
    pub arguments: Value,
    /// Output result
    pub output: ToolOutput,
    /// Execution duration (ms)
    pub duration_ms: u64,
    /// Timestamp
    pub timestamp: std::time::Instant,
}

/// Tool call tracker
///
/// P2 FIX: Uses VecDeque for O(1) removal from front.
pub struct ToolCallTracker {
    calls: VecDeque<ToolCall>,
    max_history: usize,
}

impl ToolCallTracker {
    pub fn new(max_history: usize) -> Self {
        Self {
            calls: VecDeque::with_capacity(max_history),
            max_history,
        }
    }

    /// Record a tool call
    pub fn record(&mut self, call: ToolCall) {
        if self.calls.len() >= self.max_history {
            self.calls.pop_front(); // P2 FIX: O(1) instead of O(n)
        }
        self.calls.push_back(call);
    }

    /// Get recent calls as a slice
    ///
    /// P2 FIX: Returns contiguous slice by ensuring make_contiguous.
    pub fn recent(&mut self, n: usize) -> &[ToolCall] {
        self.calls.make_contiguous();
        let (slice, _) = self.calls.as_slices();
        let start = slice.len().saturating_sub(n);
        &slice[start..]
    }

    /// Get all calls as a slice
    ///
    /// P2 FIX: Returns contiguous slice by ensuring make_contiguous.
    pub fn all(&mut self) -> &[ToolCall] {
        self.calls.make_contiguous();
        let (slice, _) = self.calls.as_slices();
        slice
    }

    /// Get calls by tool name
    pub fn by_name(&self, name: &str) -> Vec<&ToolCall> {
        self.calls.iter().filter(|c| c.name == name).collect()
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.calls.clear();
    }
}

/// Create default registry with gold loan tools
pub fn create_default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // Register gold loan tools
    registry.register(crate::gold_loan::EligibilityCheckTool::new());
    registry.register(crate::gold_loan::SavingsCalculatorTool::new());
    registry.register(crate::gold_loan::LeadCaptureTool::new());
    registry.register(crate::gold_loan::AppointmentSchedulerTool::new());
    registry.register(crate::gold_loan::BranchLocatorTool::new());

    // P0 FIX: Register missing tools that were implemented but not registered
    registry.register(crate::gold_loan::GetGoldPriceTool::new());
    registry.register(crate::gold_loan::EscalateToHumanTool::new());
    registry.register(crate::gold_loan::SendSmsTool::new());

    // Phase 6: Additional gold loan tools
    registry.register(crate::gold_loan::DocumentChecklistTool::new());
    registry.register(crate::gold_loan::CompetitorComparisonTool::new());

    registry
}

// =============================================================================
// P0-4 FIX: Domain Config Wiring with Hot-Reload Support
// =============================================================================

/// P0-4 FIX: Create registry with domain configuration injected
///
/// Uses the provided GoldLoanConfig instead of defaults, allowing
/// for configurable interest rates, LTV, competitor rates, etc.
pub fn create_registry_with_config(
    gold_loan_config: &voice_agent_config::GoldLoanConfig,
) -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // P0-4: Register gold loan tools with injected config
    registry.register(crate::gold_loan::EligibilityCheckTool::with_config(
        gold_loan_config.clone(),
    ));
    registry.register(crate::gold_loan::SavingsCalculatorTool::with_config(
        gold_loan_config.clone(),
    ));
    registry.register(crate::gold_loan::LeadCaptureTool::new());
    registry.register(crate::gold_loan::AppointmentSchedulerTool::new());
    registry.register(crate::gold_loan::BranchLocatorTool::new());

    // Tools that don't need config
    registry.register(crate::gold_loan::GetGoldPriceTool::new());
    registry.register(crate::gold_loan::EscalateToHumanTool::new());
    registry.register(crate::gold_loan::SendSmsTool::new());

    // Phase 6: Additional gold loan tools with config
    registry.register(crate::gold_loan::DocumentChecklistTool::new());
    registry.register(crate::gold_loan::CompetitorComparisonTool::with_config(
        gold_loan_config.clone(),
    ));

    tracing::info!(
        kotak_rate = gold_loan_config.kotak_interest_rate,
        ltv = gold_loan_config.ltv_percent,
        "Created tool registry with domain config"
    );

    registry
}

/// P0-4 FIX: Create registry with full domain configuration
///
/// Takes the complete DomainConfig and extracts relevant parts for each tool.
pub fn create_registry_with_domain_config(
    domain_config: &voice_agent_config::DomainConfig,
) -> ToolRegistry {
    create_registry_with_config(&domain_config.gold_loan)
}

/// P0-4 FIX: Configurable tool registry with hot-reload support
///
/// Wraps a ToolRegistry with config management, allowing tools to be
/// recreated when configuration changes.
pub struct ConfigurableToolRegistry {
    inner: parking_lot::RwLock<ToolRegistry>,
    config: parking_lot::RwLock<voice_agent_config::GoldLoanConfig>,
}

impl ConfigurableToolRegistry {
    /// Create with initial config
    pub fn new(config: voice_agent_config::GoldLoanConfig) -> Self {
        let registry = create_registry_with_config(&config);
        Self {
            inner: parking_lot::RwLock::new(registry),
            config: parking_lot::RwLock::new(config),
        }
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::new(voice_agent_config::GoldLoanConfig::default())
    }

    /// Reload configuration and recreate tools
    ///
    /// This is the hot-reload entry point. Call this when config file changes.
    pub fn reload(&self, new_config: voice_agent_config::GoldLoanConfig) {
        tracing::info!(
            old_rate = %self.config.read().kotak_interest_rate,
            new_rate = %new_config.kotak_interest_rate,
            "Hot-reloading tool configuration"
        );

        // Update config
        *self.config.write() = new_config.clone();

        // Recreate registry with new config
        let new_registry = create_registry_with_config(&new_config);
        *self.inner.write() = new_registry;

        tracing::info!("Tool registry reloaded with new configuration");
    }

    /// Get current config
    pub fn config(&self) -> voice_agent_config::GoldLoanConfig {
        self.config.read().clone()
    }

    /// Execute a tool
    pub async fn execute(&self, name: &str, arguments: Value) -> Result<ToolOutput, ToolError> {
        // Get the tool without holding the lock across await
        let tool = {
            let registry = self.inner.read();
            registry.get(name).cloned()
        };

        let tool = tool.ok_or_else(|| ToolError::not_found(format!("Tool not found: {}", name)))?;

        // Validate input
        tool.validate(&arguments)?;

        // Execute with timeout (from the Tool trait default)
        let timeout_secs = tool.timeout_secs();
        let timeout_duration = Duration::from_secs(timeout_secs);

        match tokio::time::timeout(timeout_duration, tool.execute(arguments)).await {
            Ok(result) => result,
            Err(_elapsed) => Err(ToolError::timeout(name, timeout_secs)),
        }
    }

    /// List available tools
    pub fn list_tools(&self) -> Vec<ToolSchema> {
        self.inner.read().list_tools()
    }

    /// Get tool schema
    pub fn get_tool(&self, name: &str) -> Option<ToolSchema> {
        self.inner.read().get_tool(name)
    }

    /// Check if tool exists
    pub fn has(&self, name: &str) -> bool {
        self.inner.read().has(name)
    }

    /// Get tool count
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }
}

#[async_trait]
impl ToolExecutor for ConfigurableToolRegistry {
    async fn execute(&self, name: &str, arguments: Value) -> Result<ToolOutput, ToolError> {
        self.execute(name, arguments).await
    }

    fn list_tools(&self) -> Vec<ToolSchema> {
        self.list_tools()
    }

    fn get_tool(&self, name: &str) -> Option<ToolSchema> {
        self.get_tool(name)
    }
}

/// P4 FIX: Integration configuration for tool registry
#[derive(Default)]
pub struct IntegrationConfig {
    /// CRM integration for lead management
    pub crm: Option<Arc<dyn crate::integrations::CrmIntegration>>,
    /// Calendar integration for appointment scheduling
    pub calendar: Option<Arc<dyn crate::integrations::CalendarIntegration>>,
}

impl IntegrationConfig {
    /// Create with stub integrations (for development/testing)
    pub fn with_stubs() -> Self {
        Self {
            crm: Some(Arc::new(crate::integrations::StubCrmIntegration::new())),
            calendar: Some(Arc::new(crate::integrations::StubCalendarIntegration::new())),
        }
    }

    /// Set CRM integration
    pub fn with_crm(mut self, crm: Arc<dyn crate::integrations::CrmIntegration>) -> Self {
        self.crm = Some(crm);
        self
    }

    /// Set calendar integration
    pub fn with_calendar(
        mut self,
        calendar: Arc<dyn crate::integrations::CalendarIntegration>,
    ) -> Self {
        self.calendar = Some(calendar);
        self
    }
}

/// P4 FIX: Create registry with integration support
///
/// Creates a tool registry with optional CRM and calendar integrations
/// injected into the appropriate tools (LeadCaptureTool, AppointmentSchedulerTool).
pub fn create_registry_with_integrations(config: IntegrationConfig) -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // Register gold loan tools
    registry.register(crate::gold_loan::EligibilityCheckTool::new());
    registry.register(crate::gold_loan::SavingsCalculatorTool::new());
    registry.register(crate::gold_loan::BranchLocatorTool::new());

    // P4 FIX: Register LeadCaptureTool with CRM integration if available
    if let Some(crm) = config.crm {
        registry.register(crate::gold_loan::LeadCaptureTool::with_crm(crm));
    } else {
        registry.register(crate::gold_loan::LeadCaptureTool::new());
    }

    // P4 FIX: Register AppointmentSchedulerTool with calendar integration if available
    if let Some(calendar) = config.calendar {
        registry.register(crate::gold_loan::AppointmentSchedulerTool::with_calendar(
            calendar,
        ));
    } else {
        registry.register(crate::gold_loan::AppointmentSchedulerTool::new());
    }

    // P0 FIX: Register missing tools that were implemented but not registered
    registry.register(crate::gold_loan::GetGoldPriceTool::new());
    registry.register(crate::gold_loan::EscalateToHumanTool::new());
    registry.register(crate::gold_loan::SendSmsTool::new());

    // Phase 6: Additional gold loan tools
    registry.register(crate::gold_loan::DocumentChecklistTool::new());
    registry.register(crate::gold_loan::CompetitorComparisonTool::new());

    registry
}

/// P2 FIX: Full configuration for tool registry with persistence
///
/// Includes both business integrations (CRM, Calendar) and persistence
/// services (SMS, Gold Price) for production deployment.
#[derive(Default)]
pub struct FullIntegrationConfig {
    /// CRM integration for lead management
    pub crm: Option<Arc<dyn crate::integrations::CrmIntegration>>,
    /// Calendar integration for appointment scheduling
    pub calendar: Option<Arc<dyn crate::integrations::CalendarIntegration>>,
    /// SMS service for sending messages (persisted to ScyllaDB)
    pub sms_service: Option<Arc<dyn voice_agent_persistence::SmsService>>,
    /// Gold price service (persisted to ScyllaDB)
    pub gold_price_service: Option<Arc<dyn voice_agent_persistence::GoldPriceService>>,
    /// P2-1 FIX: Domain configuration for business logic (rates, LTV, etc.)
    pub gold_loan_config: Option<voice_agent_config::GoldLoanConfig>,
}

impl FullIntegrationConfig {
    /// Create from persistence layer
    pub fn from_persistence(persistence: &voice_agent_persistence::PersistenceLayer) -> Self {
        Self {
            crm: Some(Arc::new(crate::integrations::StubCrmIntegration::new())),
            calendar: Some(Arc::new(crate::integrations::StubCalendarIntegration::new())),
            sms_service: Some(
                Arc::new(persistence.sms.clone()) as Arc<dyn voice_agent_persistence::SmsService>
            ),
            gold_price_service: Some(Arc::new(persistence.gold_price.clone())
                as Arc<dyn voice_agent_persistence::GoldPriceService>),
            gold_loan_config: None, // Set separately via with_gold_loan_config()
        }
    }

    /// P2-1 FIX: Create from persistence layer with domain config
    pub fn from_persistence_with_config(
        persistence: &voice_agent_persistence::PersistenceLayer,
        gold_loan_config: voice_agent_config::GoldLoanConfig,
    ) -> Self {
        Self::from_persistence(persistence).with_gold_loan_config(gold_loan_config)
    }

    /// Set CRM integration
    pub fn with_crm(mut self, crm: Arc<dyn crate::integrations::CrmIntegration>) -> Self {
        self.crm = Some(crm);
        self
    }

    /// Set calendar integration
    pub fn with_calendar(
        mut self,
        calendar: Arc<dyn crate::integrations::CalendarIntegration>,
    ) -> Self {
        self.calendar = Some(calendar);
        self
    }

    /// Set SMS service
    pub fn with_sms_service(mut self, sms: Arc<dyn voice_agent_persistence::SmsService>) -> Self {
        self.sms_service = Some(sms);
        self
    }

    /// Set gold price service
    pub fn with_gold_price_service(
        mut self,
        price: Arc<dyn voice_agent_persistence::GoldPriceService>,
    ) -> Self {
        self.gold_price_service = Some(price);
        self
    }

    /// P2-1 FIX: Set gold loan domain configuration
    ///
    /// This config controls business logic like:
    /// - Interest rates (base and tiered)
    /// - LTV percentages
    /// - Gold price per gram
    /// - Processing fees
    /// - Competitor rates for comparison
    pub fn with_gold_loan_config(mut self, config: voice_agent_config::GoldLoanConfig) -> Self {
        self.gold_loan_config = Some(config);
        self
    }
}

/// P2 FIX: Create registry with full persistence support
///
/// Creates a tool registry with:
/// - Business integrations (CRM, Calendar)
/// - Persistence services (SMS → ScyllaDB, Gold Price → ScyllaDB)
/// - Domain configuration for business logic (rates, LTV, etc.)
/// - All MCP tools properly wired
pub fn create_registry_with_persistence(config: FullIntegrationConfig) -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // P2-1 FIX: Use domain config for tools that need business logic
    // This ensures tools use configured rates/LTV instead of hardcoded defaults
    let gold_loan_config = config.gold_loan_config.unwrap_or_default();

    // Register gold loan tools WITH config (P2-1 FIX)
    registry.register(crate::gold_loan::EligibilityCheckTool::with_config(
        gold_loan_config.clone(),
    ));
    registry.register(crate::gold_loan::SavingsCalculatorTool::with_config(
        gold_loan_config.clone(),
    ));
    registry.register(crate::gold_loan::BranchLocatorTool::new());

    // LeadCaptureTool with CRM integration
    if let Some(crm) = config.crm {
        registry.register(crate::gold_loan::LeadCaptureTool::with_crm(crm));
    } else {
        registry.register(crate::gold_loan::LeadCaptureTool::new());
    }

    // AppointmentSchedulerTool with calendar integration
    if let Some(calendar) = config.calendar {
        registry.register(crate::gold_loan::AppointmentSchedulerTool::with_calendar(
            calendar,
        ));
    } else {
        registry.register(crate::gold_loan::AppointmentSchedulerTool::new());
    }

    // P2 FIX: GetGoldPriceTool with persistence service
    if let Some(price_service) = config.gold_price_service {
        registry.register(crate::gold_loan::GetGoldPriceTool::with_price_service(
            price_service,
        ));
    } else {
        registry.register(crate::gold_loan::GetGoldPriceTool::new());
    }

    // P2 FIX: EscalateToHumanTool (no persistence needed, logs via audit)
    registry.register(crate::gold_loan::EscalateToHumanTool::new());

    // P2 FIX: SendSmsTool with persistence service
    if let Some(sms_service) = config.sms_service {
        registry.register(crate::gold_loan::SendSmsTool::with_sms_service(sms_service));
    } else {
        registry.register(crate::gold_loan::SendSmsTool::new());
    }

    // Phase 6: Additional gold loan tools with config support
    registry.register(crate::gold_loan::DocumentChecklistTool::new());
    registry.register(crate::gold_loan::CompetitorComparisonTool::with_config(
        gold_loan_config,
    ));

    tracing::info!(
        tools = registry.len(),
        "Created tool registry with persistence support"
    );

    registry
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gold_loan::EligibilityCheckTool;

    #[test]
    fn test_registry_basic() {
        let mut registry = ToolRegistry::new();
        assert!(registry.is_empty());

        registry.register(EligibilityCheckTool::new());
        assert_eq!(registry.len(), 1);
        assert!(registry.has("check_eligibility"));
    }

    #[test]
    fn test_registry_list_tools() {
        let registry = create_default_registry();
        let tools = registry.list_tools();

        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.name == "check_eligibility"));
    }

    #[test]
    fn test_tool_call_tracker() {
        let mut tracker = ToolCallTracker::new(100);

        tracker.record(ToolCall {
            name: "test".to_string(),
            arguments: serde_json::json!({}),
            output: ToolOutput::text("result"),
            duration_ms: 10,
            timestamp: std::time::Instant::now(),
        });

        assert_eq!(tracker.all().len(), 1);
    }

    // P4 FIX: Tests for integration config

    #[test]
    fn test_integration_config_default() {
        let config = IntegrationConfig::default();
        assert!(config.crm.is_none());
        assert!(config.calendar.is_none());
    }

    #[test]
    fn test_integration_config_with_stubs() {
        let config = IntegrationConfig::with_stubs();
        assert!(config.crm.is_some());
        assert!(config.calendar.is_some());
    }

    #[test]
    fn test_registry_with_integrations() {
        let config = IntegrationConfig::with_stubs();
        let registry = create_registry_with_integrations(config);

        // Phase 6: Should have all 10 tools (8 original + 2 Phase 6 tools)
        assert_eq!(registry.len(), 10);
        assert!(registry.has("check_eligibility"));
        assert!(registry.has("calculate_savings"));
        assert!(registry.has("capture_lead"));
        assert!(registry.has("schedule_appointment"));
        assert!(registry.has("find_branches"));
        // P0 FIX: Verify new tools are registered
        assert!(registry.has("get_gold_price"));
        assert!(registry.has("escalate_to_human"));
        assert!(registry.has("send_sms"));
        // Phase 6: New tools
        assert!(registry.has("get_document_checklist"));
        assert!(registry.has("compare_lenders"));
    }

    #[test]
    fn test_registry_without_integrations() {
        let config = IntegrationConfig::default();
        let registry = create_registry_with_integrations(config);

        // Phase 6: Should still have all 10 tools (just without integrations)
        assert_eq!(registry.len(), 10);
        assert!(registry.has("capture_lead"));
        assert!(registry.has("schedule_appointment"));
        // P0 FIX: Verify new tools are registered
        assert!(registry.has("get_gold_price"));
        assert!(registry.has("escalate_to_human"));
        assert!(registry.has("send_sms"));
        // Phase 6: New tools
        assert!(registry.has("get_document_checklist"));
        assert!(registry.has("compare_lenders"));
    }

    #[test]
    fn test_default_registry_has_all_tools() {
        let registry = create_default_registry();

        // Phase 6: Registry should have all 10 tools (8 original + 2 new)
        assert_eq!(registry.len(), 10);
        assert!(registry.has("check_eligibility"));
        assert!(registry.has("calculate_savings"));
        assert!(registry.has("capture_lead"));
        assert!(registry.has("schedule_appointment"));
        assert!(registry.has("find_branches"));
        assert!(registry.has("get_gold_price"));
        assert!(registry.has("escalate_to_human"));
        assert!(registry.has("send_sms"));
        // Phase 6: New tools
        assert!(registry.has("get_document_checklist"));
        assert!(registry.has("compare_lenders"));
    }
}
