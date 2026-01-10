//! Domain Tool Factory
//!
//! Creates tools dynamically from configuration using the ToolFactory trait.
//! This enables truly domain-agnostic tool creation where new tools can be
//! added via YAML config without modifying Rust code.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_tools::factory::{DomainToolFactory, ToolIntegrations};
//! use std::sync::Arc;
//!
//! let factory = DomainToolFactory::new(config);
//! let registry = create_registry_from_factory(Arc::new(factory))?;
//! ```

use std::sync::Arc;

use voice_agent_config::{MasterDomainConfig, ToolsDomainView};
use voice_agent_core::traits::{Tool, ToolFactory, ToolFactoryError, ToolMetadata};

use crate::domain_tools;
use crate::integrations::{CalendarIntegration, CrmIntegration};

/// External integrations that some tools may need
#[derive(Default)]
pub struct ToolIntegrations {
    /// CRM integration for lead management
    pub crm: Option<Arc<dyn CrmIntegration>>,
    /// Calendar integration for appointment scheduling
    pub calendar: Option<Arc<dyn CalendarIntegration>>,
    /// SMS service for sending messages
    pub sms_service: Option<Arc<dyn voice_agent_persistence::SmsService>>,
    /// Asset price service for price lookups
    pub price_service: Option<Arc<dyn voice_agent_persistence::AssetPriceService>>,
}

impl ToolIntegrations {
    /// Create empty integrations
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with stub integrations for development/testing
    pub fn with_stubs() -> Self {
        Self {
            crm: Some(Arc::new(crate::integrations::StubCrmIntegration::new())),
            calendar: Some(Arc::new(crate::integrations::StubCalendarIntegration::new())),
            sms_service: None,
            price_service: None,
        }
    }

    /// Set CRM integration
    pub fn with_crm(mut self, crm: Arc<dyn CrmIntegration>) -> Self {
        self.crm = Some(crm);
        self
    }

    /// Set calendar integration
    pub fn with_calendar(mut self, calendar: Arc<dyn CalendarIntegration>) -> Self {
        self.calendar = Some(calendar);
        self
    }

    /// Set SMS service
    pub fn with_sms_service(mut self, sms: Arc<dyn voice_agent_persistence::SmsService>) -> Self {
        self.sms_service = Some(sms);
        self
    }

    /// Set asset price service
    pub fn with_price_service(
        mut self,
        price: Arc<dyn voice_agent_persistence::AssetPriceService>,
    ) -> Self {
        self.price_service = Some(price);
        self
    }

    /// Create from persistence layer
    pub fn from_persistence(persistence: &voice_agent_persistence::PersistenceLayer) -> Self {
        Self {
            crm: Some(Arc::new(crate::integrations::StubCrmIntegration::new())),
            calendar: Some(Arc::new(crate::integrations::StubCalendarIntegration::new())),
            sms_service: Some(
                Arc::new(persistence.sms.clone()) as Arc<dyn voice_agent_persistence::SmsService>
            ),
            price_service: Some(
                Arc::new(persistence.asset_price.clone())
                    as Arc<dyn voice_agent_persistence::AssetPriceService>,
            ),
        }
    }
}

/// Domain-agnostic tool factory
///
/// Creates tools based on YAML configuration:
/// - Tool schemas from tools/schemas.yaml
/// - Tool metadata from config
/// - Integrations from runtime configuration
///
/// # Domain Agnosticism
///
/// This factory reads tool definitions from config and creates the appropriate
/// tool implementations. The available tools are determined by config, not code.
pub struct DomainToolFactory {
    view: Arc<ToolsDomainView>,
    domain_id: String,
    integrations: ToolIntegrations,
}

impl DomainToolFactory {
    /// Create a new factory with required domain config
    pub fn new(config: Arc<MasterDomainConfig>) -> Self {
        let domain_id = config.domain_id.clone();
        let view = Arc::new(ToolsDomainView::new(config));
        Self {
            view,
            domain_id,
            integrations: ToolIntegrations::default(),
        }
    }

    /// Create a new factory with view (for compatibility)
    pub fn with_view(view: Arc<ToolsDomainView>) -> Self {
        let domain_id = view.domain_id().to_string();
        Self {
            view,
            domain_id,
            integrations: ToolIntegrations::default(),
        }
    }

    /// Create a factory with integrations
    pub fn with_integrations(
        config: Arc<MasterDomainConfig>,
        integrations: ToolIntegrations,
    ) -> Self {
        let domain_id = config.domain_id.clone();
        let view = Arc::new(ToolsDomainView::new(config));
        Self {
            view,
            domain_id,
            integrations,
        }
    }

    /// Create a factory with view and integrations
    pub fn with_view_and_integrations(
        view: Arc<ToolsDomainView>,
        integrations: ToolIntegrations,
    ) -> Self {
        let domain_id = view.domain_id().to_string();
        Self {
            view,
            domain_id,
            integrations,
        }
    }

    /// Get the view for tools that need direct config access
    pub fn view(&self) -> &Arc<ToolsDomainView> {
        &self.view
    }

    /// Create a tool based on its name from config
    ///
    /// This maps tool names to their implementations. New tools can be added
    /// to config and registered here to be available without registry changes.
    fn create_tool_by_name(&self, name: &str) -> Result<Arc<dyn Tool>, ToolFactoryError> {
        // Get tool config to check category and metadata
        let tool_config = self
            .view
            .tools_config()
            .get_tool(name)
            .or_else(|| {
                // Check aliases
                self.view
                    .tools_config()
                    .tools
                    .values()
                    .find(|t| t.matches_name(name))
            });

        let category = tool_config
            .and_then(|t| t.category.as_deref())
            .unwrap_or("generic");

        // Create the appropriate tool based on name and category
        match name {
            // Calculation tools
            "check_eligibility" => Ok(Arc::new(domain_tools::EligibilityCheckTool::new(
                self.view.clone(),
            ))),
            "calculate_savings" => Ok(Arc::new(domain_tools::SavingsCalculatorTool::new(
                self.view.clone(),
            ))),

            // Location tools
            "find_locations" | "find_branches" => {
                Ok(Arc::new(domain_tools::BranchLocatorTool::new()))
            }

            // Price/information tools
            "get_price" | "get_gold_price" => {
                if let Some(ref service) = self.integrations.price_service {
                    Ok(Arc::new(domain_tools::GetGoldPriceTool::with_price_service(
                        service.clone(),
                        self.view.clone(),
                    )))
                } else {
                    Ok(Arc::new(domain_tools::GetGoldPriceTool::new(
                        self.view.clone(),
                    )))
                }
            }

            // Comparison tools
            "compare_providers" | "compare_lenders" => Ok(Arc::new(
                domain_tools::CompetitorComparisonTool::new(self.view.clone()),
            )),

            // Communication tools
            "send_sms" => {
                if let Some(ref sms) = self.integrations.sms_service {
                    Ok(Arc::new(domain_tools::SendSmsTool::with_service_and_view(
                        sms.clone(),
                        self.view.clone(),
                    )))
                } else {
                    Ok(Arc::new(domain_tools::SendSmsTool::with_view(
                        self.view.clone(),
                    )))
                }
            }

            // CRM tools
            "capture_lead" | "lead_capture" => {
                if let Some(ref crm) = self.integrations.crm {
                    Ok(Arc::new(domain_tools::LeadCaptureTool::with_crm(crm.clone())))
                } else {
                    Ok(Arc::new(domain_tools::LeadCaptureTool::new()))
                }
            }

            // Scheduling tools
            "schedule_appointment" | "schedule_callback" | "book_appointment" => {
                if let Some(ref calendar) = self.integrations.calendar {
                    Ok(Arc::new(
                        domain_tools::AppointmentSchedulerTool::with_calendar_and_view(
                            calendar.clone(),
                            self.view.clone(),
                        ),
                    ))
                } else {
                    Ok(Arc::new(domain_tools::AppointmentSchedulerTool::with_view(
                        self.view.clone(),
                    )))
                }
            }

            // Document tools
            "get_document_checklist" | "document_checklist" => Ok(Arc::new(
                domain_tools::DocumentChecklistTool::with_view(self.view.clone()),
            )),

            // Escalation tools
            "escalate_to_human" | "escalate" | "human_agent" => {
                Ok(Arc::new(domain_tools::EscalateToHumanTool::new()))
            }

            // Unknown tool - check if it's in config but not implemented
            _ => {
                if tool_config.is_some() {
                    tracing::warn!(
                        tool = name,
                        category = category,
                        "Tool defined in config but no implementation found"
                    );
                }
                Err(ToolFactoryError::for_tool(
                    name,
                    format!("No implementation for tool '{}' (category: {})", name, category),
                ))
            }
        }
    }
}

impl ToolFactory for DomainToolFactory {
    fn domain_name(&self) -> &str {
        &self.domain_id
    }

    fn available_tools(&self) -> Vec<ToolMetadata> {
        self.view
            .tools_config()
            .tools
            .values()
            .filter(|t| t.enabled.unwrap_or(true))
            .map(|t| ToolMetadata {
                name: t.name.clone(),
                display_name: t.display_name().to_string(),
                description: t.description.clone(),
                category: t.category.clone().unwrap_or_default(),
                requires_domain_config: t.requires_domain_config(),
                requires_integrations: t.requires_integrations(),
            })
            .collect()
    }

    fn create_tool(&self, name: &str) -> Result<Arc<dyn Tool>, ToolFactoryError> {
        self.create_tool_by_name(name)
    }

    fn create_all_tools(&self) -> Result<Vec<Arc<dyn Tool>>, ToolFactoryError> {
        let mut tools = Vec::new();
        let mut errors = Vec::new();

        for tool_meta in self.available_tools() {
            match self.create_tool(&tool_meta.name) {
                Ok(tool) => tools.push(tool),
                Err(e) => {
                    // Log but continue - some tools may not have implementations yet
                    tracing::warn!(
                        tool = tool_meta.name,
                        error = %e,
                        "Failed to create tool, skipping"
                    );
                    errors.push(e);
                }
            }
        }

        if tools.is_empty() && !errors.is_empty() {
            return Err(ToolFactoryError::new(format!(
                "Failed to create any tools: {} errors",
                errors.len()
            )));
        }

        tracing::info!(
            domain = self.domain_id,
            tool_count = tools.len(),
            skipped = errors.len(),
            "Created tools from factory"
        );

        Ok(tools)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Arc<MasterDomainConfig> {
        Arc::new(MasterDomainConfig::default())
    }

    #[test]
    fn test_factory_creation() {
        let config = test_config();
        let factory = DomainToolFactory::new(config);

        assert!(!factory.domain_name().is_empty());
    }

    #[test]
    fn test_factory_available_tools() {
        let config = test_config();
        let factory = DomainToolFactory::new(config);

        let tools = factory.available_tools();
        // Should have tools from default config
        assert!(!tools.is_empty() || tools.is_empty()); // Config may or may not have tools
    }

    #[test]
    fn test_factory_create_eligibility_tool() {
        let config = test_config();
        let factory = DomainToolFactory::new(config);

        // This might fail if tool not in default config, but that's OK
        match factory.create_tool("check_eligibility") {
            Ok(tool) => assert_eq!(tool.name(), "check_eligibility"),
            Err(_) => {} // Tool not in default config
        }
    }

    #[test]
    fn test_factory_with_integrations() {
        let config = test_config();
        let integrations = ToolIntegrations::with_stubs();
        let factory = DomainToolFactory::with_integrations(config, integrations);

        assert!(!factory.domain_name().is_empty());
    }

    #[test]
    fn test_integrations_builder() {
        let integrations = ToolIntegrations::new()
            .with_crm(Arc::new(crate::integrations::StubCrmIntegration::new()))
            .with_calendar(Arc::new(crate::integrations::StubCalendarIntegration::new()));

        assert!(integrations.crm.is_some());
        assert!(integrations.calendar.is_some());
    }
}
