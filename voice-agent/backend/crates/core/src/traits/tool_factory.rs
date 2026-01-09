//! Tool Factory trait for domain-agnostic tool creation
//!
//! This module provides a factory pattern for creating domain-specific tools
//! from configuration, enabling true domain-agnosticism in the tool registry.
//!
//! # Example
//!
//! ```ignore
//! use voice_agent_core::traits::{ToolFactory, Tool};
//! use std::sync::Arc;
//!
//! // Create factory for a domain
//! let factory = create_tool_factory_for_domain("gold_loan", &config)?;
//!
//! // Create tools from factory
//! let tools = factory.create_all_tools();
//! ```

use std::sync::Arc;

use super::Tool;

/// Error type for tool factory operations
#[derive(Debug, Clone)]
pub struct ToolFactoryError {
    /// Error message
    pub message: String,
    /// Tool name that caused the error (if applicable)
    pub tool_name: Option<String>,
}

impl ToolFactoryError {
    /// Create a new error
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            tool_name: None,
        }
    }

    /// Create an error for a specific tool
    pub fn for_tool(tool_name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            tool_name: Some(tool_name.into()),
        }
    }
}

impl std::fmt::Display for ToolFactoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref tool) = self.tool_name {
            write!(f, "Tool '{}': {}", tool, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ToolFactoryError {}

/// Tool metadata for discovery
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    /// Tool name/ID
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Description
    pub description: String,
    /// Category (e.g., "eligibility", "comparison", "appointment")
    pub category: String,
    /// Whether this tool requires domain configuration
    pub requires_domain_config: bool,
    /// Whether this tool requires external integrations (CRM, calendar, etc.)
    pub requires_integrations: bool,
}

/// Tool Factory trait for domain-agnostic tool creation
///
/// This trait enables creating domain-specific tools from configuration
/// without hardcoding the tool implementations in the registry.
///
/// # Domain-Agnostic Design
///
/// Each domain (gold_loan, insurance, credit_card, etc.) implements
/// this trait to provide its own set of tools. The registry uses
/// the factory interface without knowing about specific tools.
///
/// # Example Implementation
///
/// ```ignore
/// struct GoldLoanToolFactory {
///     view: Arc<ToolsDomainView>,
/// }
///
/// impl ToolFactory for GoldLoanToolFactory {
///     fn domain_name(&self) -> &str { "gold_loan" }
///
///     fn available_tools(&self) -> Vec<ToolMetadata> {
///         vec![
///             ToolMetadata {
///                 name: "check_eligibility".to_string(),
///                 display_name: "Eligibility Check".to_string(),
///                 description: "Check loan eligibility".to_string(),
///                 category: "eligibility".to_string(),
///                 requires_domain_config: true,
///                 requires_integrations: false,
///             },
///             // ... more tools
///         ]
///     }
///
///     fn create_tool(&self, name: &str) -> Result<Arc<dyn Tool>, ToolFactoryError> {
///         match name {
///             "check_eligibility" => Ok(Arc::new(EligibilityCheckTool::new(self.view.clone()))),
///             _ => Err(ToolFactoryError::for_tool(name, "Unknown tool")),
///         }
///     }
/// }
/// ```
pub trait ToolFactory: Send + Sync {
    /// Get the domain name this factory serves
    fn domain_name(&self) -> &str;

    /// Get metadata for all available tools
    fn available_tools(&self) -> Vec<ToolMetadata>;

    /// Get metadata for a specific tool
    fn tool_metadata(&self, name: &str) -> Option<ToolMetadata> {
        self.available_tools()
            .into_iter()
            .find(|m| m.name == name)
    }

    /// Create a single tool by name
    fn create_tool(&self, name: &str) -> Result<Arc<dyn Tool>, ToolFactoryError>;

    /// Create all tools for this domain
    fn create_all_tools(&self) -> Result<Vec<Arc<dyn Tool>>, ToolFactoryError> {
        self.available_tools()
            .iter()
            .map(|m| self.create_tool(&m.name))
            .collect()
    }

    /// Create tools by category
    fn create_tools_by_category(&self, category: &str) -> Result<Vec<Arc<dyn Tool>>, ToolFactoryError> {
        self.available_tools()
            .iter()
            .filter(|m| m.category == category)
            .map(|m| self.create_tool(&m.name))
            .collect()
    }

    /// Check if a tool is available
    fn has_tool(&self, name: &str) -> bool {
        self.available_tools().iter().any(|m| m.name == name)
    }

    /// Get tool categories
    fn categories(&self) -> Vec<String> {
        let mut categories: Vec<_> = self
            .available_tools()
            .iter()
            .map(|m| m.category.clone())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

/// Registry for tool factories across domains
///
/// Allows multiple domains to register their factories, enabling
/// the system to create tools for any configured domain.
pub struct ToolFactoryRegistry {
    factories: std::collections::HashMap<String, Arc<dyn ToolFactory>>,
}

impl ToolFactoryRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            factories: std::collections::HashMap::new(),
        }
    }

    /// Register a factory for a domain
    pub fn register(&mut self, factory: Arc<dyn ToolFactory>) {
        let domain = factory.domain_name().to_string();
        self.factories.insert(domain, factory);
    }

    /// Get factory for a domain
    pub fn get(&self, domain: &str) -> Option<&Arc<dyn ToolFactory>> {
        self.factories.get(domain)
    }

    /// Check if a domain is registered
    pub fn has_domain(&self, domain: &str) -> bool {
        self.factories.contains_key(domain)
    }

    /// Get all registered domain names
    pub fn domains(&self) -> Vec<&str> {
        self.factories.keys().map(|s| s.as_str()).collect()
    }

    /// Create all tools for a domain
    pub fn create_tools_for_domain(&self, domain: &str) -> Result<Vec<Arc<dyn Tool>>, ToolFactoryError> {
        self.factories
            .get(domain)
            .ok_or_else(|| ToolFactoryError::new(format!("Domain not registered: {}", domain)))?
            .create_all_tools()
    }

    /// Create a specific tool from a domain
    pub fn create_tool(&self, domain: &str, name: &str) -> Result<Arc<dyn Tool>, ToolFactoryError> {
        self.factories
            .get(domain)
            .ok_or_else(|| ToolFactoryError::new(format!("Domain not registered: {}", domain)))?
            .create_tool(name)
    }
}

impl Default for ToolFactoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::Value;

    /// Mock tool for testing
    struct MockTool {
        name: String,
    }

    #[async_trait]
    impl Tool for MockTool {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            "Mock tool for testing"
        }

        fn schema(&self) -> super::super::ToolSchema {
            super::super::ToolSchema {
                name: self.name.clone(),
                description: self.description().to_string(),
                input_schema: super::super::InputSchema::object(),
            }
        }

        async fn execute(&self, _input: Value) -> Result<super::super::ToolOutput, super::super::ToolError> {
            Ok(super::super::ToolOutput::text("mock result"))
        }
    }

    /// Mock factory for testing
    struct MockToolFactory {
        domain: String,
    }

    impl ToolFactory for MockToolFactory {
        fn domain_name(&self) -> &str {
            &self.domain
        }

        fn available_tools(&self) -> Vec<ToolMetadata> {
            vec![
                ToolMetadata {
                    name: "mock_tool_1".to_string(),
                    display_name: "Mock Tool 1".to_string(),
                    description: "First mock tool".to_string(),
                    category: "testing".to_string(),
                    requires_domain_config: false,
                    requires_integrations: false,
                },
                ToolMetadata {
                    name: "mock_tool_2".to_string(),
                    display_name: "Mock Tool 2".to_string(),
                    description: "Second mock tool".to_string(),
                    category: "testing".to_string(),
                    requires_domain_config: true,
                    requires_integrations: false,
                },
            ]
        }

        fn create_tool(&self, name: &str) -> Result<Arc<dyn Tool>, ToolFactoryError> {
            match name {
                "mock_tool_1" | "mock_tool_2" => Ok(Arc::new(MockTool {
                    name: name.to_string(),
                })),
                _ => Err(ToolFactoryError::for_tool(name, "Unknown tool")),
            }
        }
    }

    #[test]
    fn test_factory_metadata() {
        let factory = MockToolFactory {
            domain: "test_domain".to_string(),
        };

        assert_eq!(factory.domain_name(), "test_domain");
        assert_eq!(factory.available_tools().len(), 2);
        assert!(factory.has_tool("mock_tool_1"));
        assert!(!factory.has_tool("nonexistent"));
    }

    #[test]
    fn test_factory_create_tool() {
        let factory = MockToolFactory {
            domain: "test_domain".to_string(),
        };

        let tool = factory.create_tool("mock_tool_1").unwrap();
        assert_eq!(tool.name(), "mock_tool_1");

        match factory.create_tool("nonexistent") {
            Err(err) => assert!(err.message.contains("Unknown")),
            Ok(_) => panic!("Expected error for nonexistent tool"),
        }
    }

    #[test]
    fn test_factory_create_all() {
        let factory = MockToolFactory {
            domain: "test_domain".to_string(),
        };

        let tools = factory.create_all_tools().unwrap();
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn test_factory_categories() {
        let factory = MockToolFactory {
            domain: "test_domain".to_string(),
        };

        let categories = factory.categories();
        assert_eq!(categories, vec!["testing"]);
    }

    #[test]
    fn test_registry() {
        let mut registry = ToolFactoryRegistry::new();

        let factory = Arc::new(MockToolFactory {
            domain: "test_domain".to_string(),
        });

        registry.register(factory);

        assert!(registry.has_domain("test_domain"));
        assert!(!registry.has_domain("other_domain"));
        assert_eq!(registry.domains(), vec!["test_domain"]);

        let tools = registry.create_tools_for_domain("test_domain").unwrap();
        assert_eq!(tools.len(), 2);

        let tool = registry.create_tool("test_domain", "mock_tool_1").unwrap();
        assert_eq!(tool.name(), "mock_tool_1");
    }

    #[test]
    fn test_registry_unknown_domain() {
        let registry = ToolFactoryRegistry::new();

        match registry.create_tools_for_domain("unknown") {
            Err(err) => assert!(err.message.contains("not registered")),
            Ok(_) => panic!("Expected error for unknown domain"),
        }
    }
}
