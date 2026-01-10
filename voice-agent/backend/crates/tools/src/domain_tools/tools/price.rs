//! Price Tool
//!
//! Get current asset prices per gram for different purities/variants.
//! All schema content (names, descriptions, parameters) comes from YAML config.

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{json, Value};
use std::sync::Arc;
use voice_agent_config::ToolsDomainView;

use crate::mcp::{InputSchema, PropertySchema, Tool, ToolError, ToolOutput, ToolSchema};

/// Tool name as defined in config - used to look up schema
/// Note: Must match the key in schemas.yaml for this domain
const TOOL_NAME: &str = "get_price";

/// Get asset price tool (generic name for domain-agnostic code)
///
/// P15 FIX: ToolsDomainView is now REQUIRED - no more hardcoded fallbacks
pub struct GetPriceTool {
    price_service: Option<Arc<dyn voice_agent_persistence::AssetPriceService>>,
    view: Arc<ToolsDomainView>,
}

impl GetPriceTool {
    /// Create with required ToolsDomainView - domain config is mandatory
    pub fn new(view: Arc<ToolsDomainView>) -> Self {
        Self {
            price_service: None,
            view,
        }
    }

    /// Alias for new() for backwards compatibility during migration
    pub fn with_view(view: Arc<ToolsDomainView>) -> Self {
        Self::new(view)
    }

    /// Create with price service and required view
    pub fn with_price_service(
        service: Arc<dyn voice_agent_persistence::AssetPriceService>,
        view: Arc<ToolsDomainView>,
    ) -> Self {
        Self {
            price_service: Some(service),
            view,
        }
    }

    /// Alias for with_price_service - clearer naming
    pub fn with_service_and_view(
        service: Arc<dyn voice_agent_persistence::AssetPriceService>,
        view: Arc<ToolsDomainView>,
    ) -> Self {
        Self::with_price_service(service, view)
    }

    /// Get fallback base price from config
    fn fallback_base_price(&self) -> f64 {
        self.view.asset_price_per_unit()
    }

}

#[async_trait]
impl Tool for GetPriceTool {
    fn name(&self) -> &str {
        // Return tool name from config, fallback to constant
        self.view
            .tools_config()
            .get_tool(TOOL_NAME)
            .map(|t| t.name.as_str())
            .unwrap_or(TOOL_NAME)
    }

    fn description(&self) -> &str {
        // Return description from config if available
        // Note: We can't return &str from owned String, so use static fallback
        // The actual description is included in schema()
        "Get current prices per gram for different purities"
    }

    fn schema(&self) -> ToolSchema {
        // P16 FIX: Read schema from config - all content comes from YAML
        if let Some(core_schema) = self.view.tools_config().get_core_schema(TOOL_NAME) {
            core_schema
        } else {
            // P20 FIX: Use config-driven tier codes even in fallback schema
            let tier_codes = self.view.quality_tier_short_codes();
            tracing::warn!("Tool schema not found in config for {}, using fallback with config tiers", TOOL_NAME);
            ToolSchema {
                name: TOOL_NAME.to_string(),
                description: "Get current prices".to_string(),
                input_schema: InputSchema::object()
                    .property(
                        "purity",
                        PropertySchema::enum_type(
                            "Quality tier to get price for",
                            tier_codes,
                        ),
                        false,
                    )
                    .property(
                        "weight_grams",
                        PropertySchema::number("Weight to calculate total value"),
                        false,
                    ),
            }
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolOutput, ToolError> {
        // P24 FIX: Use config-driven parameter aliases
        // "purity" is already the generic name, but can also accept "collateral_variant", "quality"
        let purity = self.view
            .tools_config()
            .get_string_param_with_aliases(&input, "collateral_variant")
            .or_else(|| input.get("purity").and_then(|v| v.as_str()).map(|s| s.to_string()));
        let purity = purity.as_deref();

        // "weight_grams" can also accept "collateral_weight", "weight"
        let weight = self.view
            .tools_config()
            .get_numeric_param_with_aliases(&input, "collateral_weight")
            .or_else(|| input.get("weight_grams").and_then(|v| v.as_f64()));

        // P20 FIX: Get quality tiers from config dynamically
        let tiers = self.view.quality_tiers_full();
        let base_price = self.fallback_base_price();

        // Calculate prices for each tier from config
        // Price service returns specific prices, otherwise calculate from base * factor
        let mut tier_prices: std::collections::HashMap<String, (f64, String)> =
            std::collections::HashMap::new();

        let source = if let Some(ref service) = self.price_service {
            match service.get_current_price().await {
                Ok(price) => {
                    // Use dynamic tier prices from service - supports any domain's tier structure
                    for (code, _factor, desc) in &tiers {
                        let tier_price = price.price_for_tier(code);
                        tier_prices.insert(code.clone(), (tier_price, desc.clone()));
                    }
                    // Also include any tiers from service not in config
                    for tier_code in price.tier_codes() {
                        if !tier_prices.contains_key(tier_code) {
                            let tier_price = price.price_for_tier(tier_code);
                            tier_prices.insert(tier_code.to_string(), (tier_price, tier_code.to_string()));
                        }
                    }
                    price.source.clone()
                }
                Err(e) => {
                    tracing::warn!("Failed to get price from service: {}", e);
                    // Calculate all from config
                    for (code, factor, desc) in &tiers {
                        tier_prices.insert(code.clone(), (base_price * factor, desc.clone()));
                    }
                    "fallback".to_string()
                }
            }
        } else {
            // No service - calculate all from config
            for (code, factor, desc) in &tiers {
                tier_prices.insert(code.clone(), (base_price * factor, desc.clone()));
            }
            "fallback".to_string()
        };

        // P2.6 FIX: Use config-driven currency field suffix
        let suffix = self.view.currency_field_suffix();
        let price_field = format!("price_per_gram_{}", suffix);

        // Build prices object dynamically from config tiers
        let mut prices_obj = serde_json::Map::new();
        for (code, factor, desc) in &tiers {
            let price = tier_prices
                .get(code)
                .map(|(p, _)| *p)
                .unwrap_or(base_price * factor);
            let tier_desc = tier_prices
                .get(code)
                .map(|(_, d)| d.clone())
                .unwrap_or_else(|| desc.clone());

            prices_obj.insert(
                code.clone(),
                json!({
                    price_field.clone(): price.round(),
                    "description": tier_desc
                }),
            );
        }

        let mut result = json!({
            "prices": prices_obj,
            "source": source,
            "updated_at": Utc::now().to_rfc3339(),
            "disclaimer": "Prices are indicative. Final value determined at branch during valuation."
        });

        // Add estimated values if weight provided
        if let Some(w) = weight {
            let mut values_obj = serde_json::Map::new();
            for (code, factor, _) in &tiers {
                let price = tier_prices
                    .get(code)
                    .map(|(p, _)| *p)
                    .unwrap_or(base_price * factor);
                values_obj.insert(code.clone(), json!((w * price).round()));
            }
            // P2.6 FIX: Use config-driven currency field suffix
            result[format!("estimated_values_{}", suffix)] = Value::Object(values_obj);
            result["weight_grams"] = json!(w);
        }

        // P20 FIX: Build message dynamically from config tiers
        let default_tier = self.view.default_quality_tier_display();
        if let Some(p) = purity {
            let price = tier_prices
                .get(p)
                .map(|(pr, _)| *pr)
                .unwrap_or_else(|| {
                    // Fallback to default tier price
                    tier_prices.get(&default_tier).map(|(pr, _)| *pr).unwrap_or(base_price)
                });

            result["requested_purity"] = json!(p);
            // P20 FIX: Use config template keys (single_variant) with correct variable names
            // P3.2 FIX: Use config-driven currency symbol
            let currency = self.view.currency_symbol();
            let message = if self.view.has_response_templates("get_price") {
                let mut vars = self.view.default_template_vars();
                vars.insert("variant_name".to_string(), p.to_string());
                vars.insert("price".to_string(), format!("{:.0}", price));
                vars.insert("unit".to_string(), self.view.asset_unit().to_string());
                vars.insert("currency".to_string(), currency.to_string());
                self.view
                    .render_response("get_price", "single_variant", "en", &vars)
                    .unwrap_or_else(|| {
                        format!(
                            "Current {} {} price is {}{:.0} per {}.",
                            p,
                            self.view.product_name(),
                            currency,
                            price,
                            self.view.asset_unit()
                        )
                    })
            } else {
                format!(
                    "Current {} {} price is {}{:.0} per {}.",
                    p,
                    self.view.product_name(),
                    currency,
                    price,
                    self.view.asset_unit()
                )
            };
            result["message"] = json!(message);
        } else {
            // P20 FIX: Build all prices message dynamically from config tiers
            // Use config template (all_variants) with positional tier variables
            let message = if self.view.has_response_templates("get_price") {
                let mut vars = self.view.default_template_vars();
                vars.insert("unit".to_string(), self.view.asset_unit().to_string());
                // Add positional tier vars (tier_1_name, tier_1_price, tier_2_name, tier_2_price, etc.)
                for (idx, (code, factor, _)) in tiers.iter().enumerate() {
                    let price = tier_prices
                        .get(code)
                        .map(|(p, _)| *p)
                        .unwrap_or(base_price * factor);
                    let tier_num = idx + 1;
                    vars.insert(format!("tier_{}_name", tier_num), code.clone());
                    vars.insert(format!("tier_{}_price", tier_num), format!("{:.0}", price));
                }
                self.view
                    .render_response("get_price", "all_variants", "en", &vars)
                    .unwrap_or_else(|| self.build_all_prices_message(&tiers, &tier_prices, base_price))
            } else {
                self.build_all_prices_message(&tiers, &tier_prices, base_price)
            };
            result["message"] = json!(message);
        }

        Ok(ToolOutput::json(result))
    }
}

impl GetPriceTool {
    /// P20 FIX: Build "all prices" message dynamically from config tiers
    fn build_all_prices_message(
        &self,
        tiers: &[(String, f64, String)],
        tier_prices: &std::collections::HashMap<String, (f64, String)>,
        base_price: f64,
    ) -> String {
        let product = self.view.product_name();
        // P3.2 FIX: Use config-driven currency symbol
        let currency = self.view.currency_symbol();
        let unit = self.view.asset_unit();
        let price_parts: Vec<String> = tiers
            .iter()
            .map(|(code, factor, _)| {
                let price = tier_prices
                    .get(code)
                    .map(|(p, _)| *p)
                    .unwrap_or(base_price * factor);
                format!("{}: {}{:.0}/{}", code, currency, price, unit)
            })
            .collect();

        format!("Current {} prices - {}", product, price_parts.join(", "))
    }
}
