//! Savings Calculator Tool
//!
//! Calculate potential savings when switching from another lender to our loan.
//! P21 FIX: Made domain-agnostic (was gold loan specific).

use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use voice_agent_config::ToolsDomainView;

use crate::mcp::{InputSchema, PropertySchema, Tool, ToolError, ToolOutput, ToolSchema};
use super::super::utils::{calculate_emi, calculate_total_interest};

/// Savings calculator tool
///
/// P13 FIX: Uses ToolsDomainView instead of GoldLoanConfig
/// P15 FIX: ToolsDomainView is now REQUIRED - no more hardcoded fallbacks
pub struct SavingsCalculatorTool {
    view: Arc<ToolsDomainView>,
}

impl SavingsCalculatorTool {
    /// Create with required ToolsDomainView - domain config is mandatory
    pub fn new(view: Arc<ToolsDomainView>) -> Self {
        Self { view }
    }

    /// Alias for new() for backwards compatibility during migration
    pub fn with_view(view: Arc<ToolsDomainView>) -> Self {
        Self::new(view)
    }

    fn get_rate(&self, amount: f64) -> f64 {
        self.view.get_rate_for_amount(amount)
    }

    fn get_competitor_rate(&self, lender: &str) -> f64 {
        self.view.get_competitor_rate(lender)
    }

    fn get_rate_tier_name(&self, amount: f64) -> &str {
        self.view.get_rate_tier_name(amount)
    }

    fn company_name(&self) -> &str {
        self.view.company_name()
    }
}

#[async_trait]
impl Tool for SavingsCalculatorTool {
    fn name(&self) -> &str {
        "calculate_savings"
    }

    fn description(&self) -> &str {
        // P21 FIX: Domain-agnostic description
        "Calculate potential savings when switching from another lender to our loan"
    }

    fn schema(&self) -> ToolSchema {
        // P24 FIX: Get rate range from config
        let rate_min = self.view
            .tools_config()
            .get_tool_default("calculate_savings", "interest_rate_min")
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0);
        let rate_max = self.view
            .tools_config()
            .get_tool_default("calculate_savings", "interest_rate_max")
            .and_then(|v| v.as_f64())
            .unwrap_or(30.0);

        ToolSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: InputSchema::object()
                .property(
                    "current_loan_amount",
                    PropertySchema::number("Current loan amount"),
                    true,
                )
                .property(
                    "current_interest_rate",
                    PropertySchema::number("Current interest rate (%)").with_range(rate_min, rate_max),
                    true,
                )
                .property(
                    "remaining_tenure_months",
                    PropertySchema::integer("Remaining tenure in months"),
                    true,
                )
                .property(
                    "current_lender",
                    PropertySchema::enum_type(
                        "Current lender",
                        // P18 FIX: Load competitor names from config instead of hardcoding
                        {
                            let mut names = self.view.all_competitor_names();
                            if !names.iter().any(|n| n.to_lowercase().contains("other")) {
                                names.push("Other".into());
                            }
                            names
                        },
                    ),
                    false,
                ),
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput, ToolError> {
        let loan_amount: f64 = input
            .get("current_loan_amount")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ToolError::invalid_params("current_loan_amount is required"))?;

        // P24 FIX: Use config-driven default for current_lender
        let default_lender = self.view
            .tools_config()
            .get_tool_default("calculate_savings", "default_lender")
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "Other Lender".to_string());
        let current_lender = input
            .get("current_lender")
            .and_then(|v| v.as_str())
            .unwrap_or(&default_lender);

        // P13 FIX: Use ToolsDomainView for competitor rates
        let current_rate: f64 = input
            .get("current_interest_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| self.get_competitor_rate(current_lender));

        let tenure_months: i64 = input
            .get("remaining_tenure_months")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| ToolError::invalid_params("remaining_tenure_months is required"))?;

        // P15 FIX: Use config-driven rates and bank name
        let our_rate = self.get_rate(loan_amount);
        let rate_tier = self.get_rate_tier_name(loan_amount);
        let company_name = self.company_name();

        let current_emi = calculate_emi(loan_amount, current_rate, tenure_months);
        let our_emi = calculate_emi(loan_amount, our_rate, tenure_months);
        let emi_savings = current_emi - our_emi;

        let current_monthly_interest = loan_amount * (current_rate / 100.0 / 12.0);
        let our_monthly_interest = loan_amount * (our_rate / 100.0 / 12.0);
        let monthly_interest_savings = current_monthly_interest - our_monthly_interest;

        let total_emi_savings = emi_savings * tenure_months as f64;
        let total_interest_savings =
            calculate_total_interest(loan_amount, current_rate, tenure_months)
                - calculate_total_interest(loan_amount, our_rate, tenure_months);

        // P16 FIX: Use config-driven response templates
        // P23 FIX: Use config-driven currency symbol instead of hardcoded "₹"
        let currency = self.view.currency_symbol();
        let message = if self.view.has_response_templates("calculate_savings") {
            let mut vars = self.view.default_template_vars();
            vars.insert("company_name".to_string(), company_name.to_string());
            vars.insert("rate_tier".to_string(), rate_tier.to_string());
            vars.insert("our_rate".to_string(), format!("{:.1}", our_rate));
            vars.insert("emi_savings".to_string(), format!("{:.0}", emi_savings));
            vars.insert("interest_savings".to_string(), format!("{:.0}", monthly_interest_savings));
            vars.insert("total_savings".to_string(), format!("{:.0}", total_emi_savings));
            vars.insert("tenure_months".to_string(), tenure_months.to_string());
            vars.insert("current_lender".to_string(), current_lender.to_string());
            vars.insert("rate_reduction".to_string(), format!("{:.1}", current_rate - our_rate));
            vars.insert("currency".to_string(), currency.to_string());
            self.view.render_response("calculate_savings", "savings_available", "en", &vars)
                .unwrap_or_else(|| format!(
                    "By switching to {} at our {} rate of {}%, you can save {}{:.0} per month on EMI (or {}{:.0} on interest-only) and {}{:.0} total over the remaining {} months!",
                    company_name, rate_tier, our_rate, currency, emi_savings, currency, monthly_interest_savings, currency, total_emi_savings, tenure_months
                ))
        } else {
            format!(
                "By switching to {} at our {} rate of {}%, you can save {}{:.0} per month on EMI (or {}{:.0} on interest-only) and {}{:.0} total over the remaining {} months!",
                company_name, rate_tier, our_rate, currency, emi_savings, currency, monthly_interest_savings, currency, total_emi_savings, tenure_months
            )
        };

        // P2.6 FIX: Use config-driven currency field suffix instead of hardcoded "_inr"
        let suffix = self.view.currency_field_suffix();
        let result = json!({
            "current_lender": current_lender,
            "current_interest_rate_percent": current_rate,
            "our_interest_rate_percent": our_rate,
            "rate_reduction_percent": current_rate - our_rate,
            format!("current_emi_{}", suffix): current_emi.round(),
            format!("our_emi_{}", suffix): our_emi.round(),
            format!("monthly_emi_savings_{}", suffix): emi_savings.round(),
            format!("total_emi_savings_{}", suffix): total_emi_savings.round(),
            format!("current_monthly_interest_{}", suffix): current_monthly_interest.round(),
            format!("our_monthly_interest_{}", suffix): our_monthly_interest.round(),
            format!("monthly_interest_savings_{}", suffix): monthly_interest_savings.round(),
            format!("total_interest_savings_{}", suffix): total_interest_savings.round(),
            "tenure_months": tenure_months,
            "rate_tier": rate_tier,
            "company_name": company_name,
            "message": message
        });

        Ok(ToolOutput::json(result))
    }
}
