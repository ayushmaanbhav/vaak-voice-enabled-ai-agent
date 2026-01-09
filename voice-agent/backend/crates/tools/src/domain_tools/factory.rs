//! Domain Tool Factory
//!
//! Implements the ToolFactory trait for domain-agnostic tool creation.
//! All tool metadata and configuration comes from the ToolsDomainView.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_tools::domain_tools::DomainToolFactory;
//! use voice_agent_core::traits::ToolFactory;
//!
//! let view = Arc::new(ToolsDomainView::new(config));
//! let factory = DomainToolFactory::new(view);
//!
//! // Create all tools
//! let tools = factory.create_all_tools()?;
//!
//! // Or create specific tools
//! let eligibility = factory.create_tool("check_eligibility")?;
//! ```

use std::sync::Arc;

use voice_agent_config::ToolsDomainView;
use voice_agent_core::traits::{Tool, ToolFactory, ToolFactoryError, ToolMetadata};

use super::tools::{
    AppointmentSchedulerTool, BranchLocatorTool, CompetitorComparisonTool, DocumentChecklistTool,
    EligibilityCheckTool, EscalateToHumanTool, GetPriceTool, LeadCaptureTool,
    SavingsCalculatorTool, SendSmsTool,
};
use crate::integrations::{CalendarIntegration, CrmIntegration};

/// Domain Tool Factory
///
/// Creates all tools for a domain using configuration from `ToolsDomainView`.
/// The factory is domain-agnostic - all domain-specific information comes from config.
pub struct DomainToolFactory {
    /// Domain configuration view
    view: Arc<ToolsDomainView>,
    /// Optional CRM integration
    crm: Option<Arc<dyn CrmIntegration>>,
    /// Optional calendar integration
    calendar: Option<Arc<dyn CalendarIntegration>>,
}

impl DomainToolFactory {
    /// Create a new factory with domain view
    pub fn new(view: Arc<ToolsDomainView>) -> Self {
        Self {
            view,
            crm: None,
            calendar: None,
        }
    }

    /// Add CRM integration
    pub fn with_crm(mut self, crm: Arc<dyn CrmIntegration>) -> Self {
        self.crm = Some(crm);
        self
    }

    /// Add calendar integration
    pub fn with_calendar(mut self, calendar: Arc<dyn CalendarIntegration>) -> Self {
        self.calendar = Some(calendar);
        self
    }

    /// Get the domain view
    pub fn view(&self) -> &Arc<ToolsDomainView> {
        &self.view
    }
}

impl ToolFactory for DomainToolFactory {
    fn domain_name(&self) -> &str {
        // Get domain name from config - no hardcoding
        self.view.domain_id()
    }

    fn available_tools(&self) -> Vec<ToolMetadata> {
        // Tool metadata - descriptions are generic, domain context comes from config
        vec![
            ToolMetadata {
                name: "check_eligibility".to_string(),
                display_name: "Eligibility Check".to_string(),
                description: "Check service eligibility based on provided parameters".to_string(),
                category: "eligibility".to_string(),
                requires_domain_config: true,
                requires_integrations: false,
            },
            ToolMetadata {
                name: "calculate_savings".to_string(),
                display_name: "Savings Calculator".to_string(),
                description: "Calculate potential savings from switching providers".to_string(),
                category: "comparison".to_string(),
                requires_domain_config: true,
                requires_integrations: false,
            },
            ToolMetadata {
                name: "compare_providers".to_string(),
                display_name: "Provider Comparison".to_string(),
                description: "Compare rates and features with other providers".to_string(),
                category: "comparison".to_string(),
                requires_domain_config: true,
                requires_integrations: false,
            },
            ToolMetadata {
                name: "get_price".to_string(),
                display_name: "Price Information".to_string(),
                description: "Get current pricing information".to_string(),
                category: "information".to_string(),
                requires_domain_config: true,
                requires_integrations: false,
            },
            ToolMetadata {
                name: "find_locations".to_string(),
                display_name: "Location Finder".to_string(),
                description: "Find nearest service locations by city or area".to_string(),
                category: "appointment".to_string(),
                requires_domain_config: true,
                requires_integrations: false,
            },
            ToolMetadata {
                name: "capture_lead".to_string(),
                display_name: "Lead Capture".to_string(),
                description: "Capture customer contact details for follow-up".to_string(),
                category: "lead".to_string(),
                requires_domain_config: false,
                requires_integrations: true,
            },
            ToolMetadata {
                name: "schedule_appointment".to_string(),
                display_name: "Appointment Scheduler".to_string(),
                description: "Schedule location visit or callback".to_string(),
                category: "appointment".to_string(),
                requires_domain_config: false,
                requires_integrations: true,
            },
            ToolMetadata {
                name: "escalate_to_human".to_string(),
                display_name: "Escalate to Human".to_string(),
                description: "Transfer call to human agent".to_string(),
                category: "escalation".to_string(),
                requires_domain_config: false,
                requires_integrations: false,
            },
            ToolMetadata {
                name: "send_sms".to_string(),
                display_name: "Send SMS".to_string(),
                description: "Send SMS with details or confirmation".to_string(),
                category: "communication".to_string(),
                requires_domain_config: false,
                requires_integrations: false,
            },
            ToolMetadata {
                name: "get_document_checklist".to_string(),
                display_name: "Document Checklist".to_string(),
                description: "Get list of required documents".to_string(),
                category: "information".to_string(),
                requires_domain_config: false,
                requires_integrations: false,
            },
        ]
    }

    fn create_tool(&self, name: &str) -> Result<Arc<dyn Tool>, ToolFactoryError> {
        match name {
            // Tools that require domain config
            "check_eligibility" => Ok(Arc::new(EligibilityCheckTool::new(self.view.clone()))),
            "calculate_savings" => Ok(Arc::new(SavingsCalculatorTool::new(self.view.clone()))),
            "compare_providers" | "compare_lenders" => {
                Ok(Arc::new(CompetitorComparisonTool::new(self.view.clone())))
            }
            "get_price" | "get_gold_price" => {
                Ok(Arc::new(GetPriceTool::new(self.view.clone())))
            }

            // Tools that require config for location data
            "find_locations" | "find_branches" => {
                Ok(Arc::new(BranchLocatorTool::new(self.view.clone())))
            }

            // Tools that don't need domain config but may use integrations
            "capture_lead" => {
                if let Some(ref crm) = self.crm {
                    Ok(Arc::new(LeadCaptureTool::with_crm(crm.clone())))
                } else {
                    Ok(Arc::new(LeadCaptureTool::new()))
                }
            }
            // P16 FIX: Appointment tool uses view for config-driven purposes/times
            "schedule_appointment" => {
                if let Some(ref calendar) = self.calendar {
                    Ok(Arc::new(AppointmentSchedulerTool::with_calendar_and_view(
                        calendar.clone(),
                        self.view.clone(),
                    )))
                } else {
                    Ok(Arc::new(AppointmentSchedulerTool::with_view(self.view.clone())))
                }
            }
            "escalate_to_human" => Ok(Arc::new(EscalateToHumanTool::new())),
            // P16 FIX: SMS and Document tools now use view for config-driven content
            "send_sms" => Ok(Arc::new(SendSmsTool::with_view(self.view.clone()))),
            "get_document_checklist" => Ok(Arc::new(DocumentChecklistTool::with_view(self.view.clone()))),

            // Unknown tool
            _ => Err(ToolFactoryError::for_tool(
                name,
                format!("Unknown tool for domain '{}'", self.domain_name()),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use voice_agent_config::MasterDomainConfig;

    fn test_view() -> Arc<ToolsDomainView> {
        let config = Arc::new(MasterDomainConfig::default());
        Arc::new(ToolsDomainView::new(config))
    }

    #[test]
    fn test_factory_domain_name() {
        let factory = DomainToolFactory::new(test_view());
        // Domain name comes from config, not hardcoded
        assert!(!factory.domain_name().is_empty());
    }

    #[test]
    fn test_factory_available_tools() {
        let factory = DomainToolFactory::new(test_view());
        let tools = factory.available_tools();

        assert_eq!(tools.len(), 10);

        // Check all expected tools are present
        let names: Vec<_> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"check_eligibility"));
        assert!(names.contains(&"calculate_savings"));
        assert!(names.contains(&"compare_providers"));
        assert!(names.contains(&"get_price"));
        assert!(names.contains(&"find_locations"));
        assert!(names.contains(&"capture_lead"));
        assert!(names.contains(&"schedule_appointment"));
        assert!(names.contains(&"escalate_to_human"));
        assert!(names.contains(&"send_sms"));
        assert!(names.contains(&"get_document_checklist"));
    }

    #[test]
    fn test_factory_create_tool() {
        let factory = DomainToolFactory::new(test_view());

        // Create a tool that requires domain config
        let tool = factory.create_tool("check_eligibility").unwrap();
        assert_eq!(tool.name(), "check_eligibility");

        // Create a tool that doesn't require domain config
        let tool = factory.create_tool("escalate_to_human").unwrap();
        assert_eq!(tool.name(), "escalate_to_human");

        // Unknown tool should error
        let result = factory.create_tool("unknown_tool");
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.message.contains("Unknown"));
    }

    #[test]
    fn test_factory_create_all_tools() {
        let factory = DomainToolFactory::new(test_view());
        let tools = factory.create_all_tools().unwrap();

        assert_eq!(tools.len(), 10);
    }

    #[test]
    fn test_factory_categories() {
        let factory = DomainToolFactory::new(test_view());
        let categories = factory.categories();

        assert!(categories.contains(&"eligibility".to_string()));
        assert!(categories.contains(&"comparison".to_string()));
        assert!(categories.contains(&"appointment".to_string()));
        assert!(categories.contains(&"information".to_string()));
    }

    #[test]
    fn test_factory_create_by_category() {
        let factory = DomainToolFactory::new(test_view());

        let comparison_tools = factory.create_tools_by_category("comparison").unwrap();
        assert_eq!(comparison_tools.len(), 2); // calculate_savings, compare_providers

        let info_tools = factory.create_tools_by_category("information").unwrap();
        assert_eq!(info_tools.len(), 2); // get_price, get_document_checklist
    }

    #[test]
    fn test_factory_has_tool() {
        let factory = DomainToolFactory::new(test_view());

        assert!(factory.has_tool("check_eligibility"));
        assert!(factory.has_tool("send_sms"));
        assert!(!factory.has_tool("nonexistent"));
    }

    #[test]
    fn test_factory_tool_metadata() {
        let factory = DomainToolFactory::new(test_view());

        let meta = factory.tool_metadata("check_eligibility").unwrap();
        assert_eq!(meta.name, "check_eligibility");
        assert_eq!(meta.category, "eligibility");
        assert!(meta.requires_domain_config);
        assert!(!meta.requires_integrations);

        let meta = factory.tool_metadata("capture_lead").unwrap();
        assert!(!meta.requires_domain_config);
        assert!(meta.requires_integrations);

        assert!(factory.tool_metadata("nonexistent").is_none());
    }

    #[test]
    fn test_legacy_tool_names() {
        // Test that legacy tool names still work for backwards compatibility
        let factory = DomainToolFactory::new(test_view());

        let tool = factory.create_tool("compare_lenders").unwrap();
        assert_eq!(tool.name(), "compare_providers");

        let tool = factory.create_tool("get_gold_price").unwrap();
        assert_eq!(tool.name(), "get_price");

        let tool = factory.create_tool("find_branches").unwrap();
        assert_eq!(tool.name(), "find_locations");
    }
}
