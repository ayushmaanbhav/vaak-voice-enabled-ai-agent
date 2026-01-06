# MCP-Compatible Tool Interface

> Industry-standard Model Context Protocol (MCP) implementation for voice agent tools
>
> **Compatibility:** Anthropic Claude, OpenAI, Google Gemini

---

## Table of Contents

1. [Overview](#overview)
2. [MCP Protocol Specification](#mcp-protocol-specification)
3. [Rust Implementation](#rust-implementation)
4. [Domain Tools](#domain-tools)
5. [Tool Registry](#tool-registry)
6. [Example Tools](#example-tools)

---

## Overview

### Why MCP?

The Model Context Protocol (MCP) is an emerging industry standard for tool calling:

| Benefit | Description |
|---------|-------------|
| **Interoperability** | Works with Claude, GPT-4, Gemini |
| **Standardization** | JSON Schema 2020-12 for parameters |
| **Type Safety** | Strongly typed tool definitions |
| **Extensibility** | Easy to add new tools |

### Protocol Version

We implement **MCP v2024.11** with:
- JSON-RPC 2.0 transport
- JSON Schema parameter validation
- Streaming result support
- Error code standardization

---

## MCP Protocol Specification

### Tool Definition Schema

```json
{
  "name": "gold_loan_calculator",
  "description": "Calculate gold loan eligibility, EMI, and savings vs competitors",
  "inputSchema": {
    "$schema": "https://json-schema.org/draft/2020-12/schema",
    "type": "object",
    "properties": {
      "gold_weight_grams": {
        "type": "number",
        "description": "Weight of gold in grams",
        "minimum": 1
      },
      "gold_purity": {
        "type": "string",
        "enum": ["24K", "22K", "18K"],
        "description": "Purity of gold",
        "default": "22K"
      },
      "loan_tenure_months": {
        "type": "integer",
        "description": "Loan tenure in months",
        "minimum": 1,
        "maximum": 36,
        "default": 12
      },
      "competitor": {
        "type": "string",
        "enum": ["muthoot", "manappuram", "iifl", "none"],
        "description": "Competitor to compare against",
        "default": "none"
      }
    },
    "required": ["gold_weight_grams"]
  }
}
```

### Tool Result Schema

```json
{
  "type": "object",
  "properties": {
    "success": { "type": "boolean" },
    "data": { "type": "object" },
    "error": {
      "type": "object",
      "properties": {
        "code": { "type": "string" },
        "message": { "type": "string" }
      }
    }
  }
}
```

---

## Rust Implementation

### Core Traits

```rust
// crates/tools/src/mcp/mod.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Unique tool name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON Schema for input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    /// Optional output schema
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<Value>,
    /// Tool category for organization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Whether tool can stream results
    #[serde(default)]
    pub supports_streaming: bool,
}

/// MCP Tool result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether execution succeeded
    pub success: bool,
    /// Result data (if success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// Error information (if failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolError>,
    /// Execution metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ToolMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidInput,
    ExecutionFailed,
    Timeout,
    RateLimited,
    Unauthorized,
    NotFound,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub execution_time_ms: u64,
    pub cache_hit: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Core MCP Tool trait
#[async_trait]
pub trait McpTool: Send + Sync {
    /// Get tool definition
    fn definition(&self) -> ToolDefinition;

    /// Execute tool with input
    async fn execute(&self, input: Value) -> Result<ToolResult, ToolError>;

    /// Validate input against schema
    fn validate_input(&self, input: &Value) -> Result<(), ToolError> {
        // Default implementation uses JSON Schema validation
        let schema = &self.definition().input_schema;
        validate_json_schema(input, schema)
    }

    /// Get tool name
    fn name(&self) -> &str {
        // Delegate to definition
        &self.definition().name
    }
}

/// Validate JSON against schema
fn validate_json_schema(value: &Value, schema: &Value) -> Result<(), ToolError> {
    // Use jsonschema crate for validation
    let compiled = jsonschema::JSONSchema::compile(schema)
        .map_err(|e| ToolError {
            code: ErrorCode::InternalError,
            message: format!("Invalid schema: {}", e),
            details: None,
        })?;

    if compiled.is_valid(value) {
        Ok(())
    } else {
        let errors: Vec<String> = compiled
            .validate(value)
            .err()
            .into_iter()
            .flat_map(|errs| errs.map(|e| e.to_string()))
            .collect();

        Err(ToolError {
            code: ErrorCode::InvalidInput,
            message: "Input validation failed".to_string(),
            details: Some(serde_json::json!({ "errors": errors })),
        })
    }
}

impl ToolResult {
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: None,
        }
    }

    pub fn failure(error: ToolError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: ToolMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}
```

### Tool Registry

```rust
// crates/tools/src/mcp/registry.rs

use std::sync::Arc;
use tokio::sync::RwLock;

/// Registry for MCP tools
pub struct ToolRegistry {
    tools: RwLock<HashMap<String, Arc<dyn McpTool>>>,
    categories: RwLock<HashMap<String, Vec<String>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: RwLock::new(HashMap::new()),
            categories: RwLock::new(HashMap::new()),
        }
    }

    /// Register a tool
    pub async fn register<T: McpTool + 'static>(&self, tool: T) {
        let definition = tool.definition();
        let name = definition.name.clone();
        let category = definition.category.clone();

        let mut tools = self.tools.write().await;
        tools.insert(name.clone(), Arc::new(tool));

        if let Some(cat) = category {
            let mut categories = self.categories.write().await;
            categories.entry(cat).or_default().push(name);
        }
    }

    /// Get tool by name
    pub async fn get(&self, name: &str) -> Option<Arc<dyn McpTool>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }

    /// List all tool definitions
    pub async fn list_definitions(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.values().map(|t| t.definition()).collect()
    }

    /// List tools by category
    pub async fn list_by_category(&self, category: &str) -> Vec<ToolDefinition> {
        let categories = self.categories.read().await;
        let tools = self.tools.read().await;

        categories
            .get(category)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| tools.get(name))
                    .map(|t| t.definition())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Execute tool by name
    pub async fn execute(&self, name: &str, input: Value) -> Result<ToolResult, ToolError> {
        let tool = self.get(name).await.ok_or(ToolError {
            code: ErrorCode::NotFound,
            message: format!("Tool '{}' not found", name),
            details: None,
        })?;

        // Validate input
        tool.validate_input(&input)?;

        // Execute
        let start = std::time::Instant::now();
        let result = tool.execute(input).await?;

        // Add metadata
        Ok(result.with_metadata(ToolMetadata {
            execution_time_ms: start.elapsed().as_millis() as u64,
            cache_hit: false,
            source: Some(name.to_string()),
        }))
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

### MCP Server

```rust
// crates/tools/src/mcp/server.rs

use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use std::sync::Arc;

/// MCP Server configuration
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
    /// Protocol version
    pub protocol_version: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            name: "voice-agent-tools".to_string(),
            version: "0.1.0".to_string(),
            protocol_version: "2024.11".to_string(),
        }
    }
}

/// MCP Server state
pub struct McpServer {
    registry: Arc<ToolRegistry>,
    config: McpServerConfig,
}

impl McpServer {
    pub fn new(registry: Arc<ToolRegistry>, config: McpServerConfig) -> Self {
        Self { registry, config }
    }

    /// Create Axum router for MCP endpoints
    pub fn router(self) -> Router {
        let state = Arc::new(self);

        Router::new()
            .route("/tools/list", post(list_tools))
            .route("/tools/call", post(call_tool))
            .route("/info", post(server_info))
            .with_state(state)
    }
}

/// JSON-RPC request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// List tools endpoint
async fn list_tools(
    State(server): State<Arc<McpServer>>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let tools = server.registry.list_definitions().await;

    Json(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: req.id,
        result: Some(serde_json::json!({ "tools": tools })),
        error: None,
    })
}

/// Call tool endpoint
async fn call_tool(
    State(server): State<Arc<McpServer>>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let params = req.params.unwrap_or(Value::Null);

    let name = params.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let arguments = params.get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));

    match server.registry.execute(name, arguments).await {
        Ok(result) => Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: Some(serde_json::to_value(result).unwrap()),
            error: None,
        }),
        Err(e) => Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: None,
            error: Some(JsonRpcError {
                code: match e.code {
                    ErrorCode::InvalidInput => -32602,
                    ErrorCode::NotFound => -32601,
                    _ => -32000,
                },
                message: e.message,
                data: e.details,
            }),
        }),
    }
}

/// Server info endpoint
async fn server_info(
    State(server): State<Arc<McpServer>>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    Json(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id: req.id,
        result: Some(serde_json::json!({
            "name": server.config.name,
            "version": server.config.version,
            "protocolVersion": server.config.protocol_version,
        })),
        error: None,
    })
}
```

---

## Domain Tools

### Gold Loan Calculator

```rust
// crates/tools/src/domain/gold_loan_calculator.rs

/// Gold loan calculator tool
pub struct GoldLoanCalculatorTool {
    rates: GoldLoanRates,
    competitor_rates: HashMap<String, CompetitorRates>,
}

#[derive(Debug, Clone)]
pub struct GoldLoanRates {
    /// Price per gram for different purities
    pub price_per_gram: HashMap<String, f64>,
    /// LTV (Loan-to-Value) ratios
    pub ltv_ratios: HashMap<String, f64>,
    /// Annual interest rate
    pub interest_rate: f64,
    /// Processing fee percentage
    pub processing_fee_pct: f64,
}

#[derive(Debug, Clone)]
pub struct CompetitorRates {
    pub interest_rate: f64,
    pub processing_fee_pct: f64,
    pub ltv_ratio: f64,
}

impl Default for GoldLoanCalculatorTool {
    fn default() -> Self {
        let mut price_per_gram = HashMap::new();
        price_per_gram.insert("24K".to_string(), 7500.0);
        price_per_gram.insert("22K".to_string(), 6875.0);
        price_per_gram.insert("18K".to_string(), 5625.0);

        let mut ltv_ratios = HashMap::new();
        ltv_ratios.insert("24K".to_string(), 0.75);
        ltv_ratios.insert("22K".to_string(), 0.75);
        ltv_ratios.insert("18K".to_string(), 0.65);

        let mut competitor_rates = HashMap::new();
        competitor_rates.insert("muthoot".to_string(), CompetitorRates {
            interest_rate: 0.21,
            processing_fee_pct: 0.01,
            ltv_ratio: 0.75,
        });
        competitor_rates.insert("manappuram".to_string(), CompetitorRates {
            interest_rate: 0.22,
            processing_fee_pct: 0.015,
            ltv_ratio: 0.70,
        });
        competitor_rates.insert("iifl".to_string(), CompetitorRates {
            interest_rate: 0.18,
            processing_fee_pct: 0.02,
            ltv_ratio: 0.65,
        });

        Self {
            rates: GoldLoanRates {
                price_per_gram,
                ltv_ratios,
                interest_rate: 0.105, // 10.5% Kotak rate
                processing_fee_pct: 0.005,
            },
            competitor_rates,
        }
    }
}

#[async_trait]
impl McpTool for GoldLoanCalculatorTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "gold_loan_calculator".to_string(),
            description: "Calculate gold loan eligibility, EMI, and savings compared to competitors".to_string(),
            input_schema: serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {
                    "gold_weight_grams": {
                        "type": "number",
                        "description": "Weight of gold in grams",
                        "minimum": 1
                    },
                    "gold_purity": {
                        "type": "string",
                        "enum": ["24K", "22K", "18K"],
                        "description": "Purity of gold",
                        "default": "22K"
                    },
                    "loan_tenure_months": {
                        "type": "integer",
                        "description": "Loan tenure in months",
                        "minimum": 1,
                        "maximum": 36,
                        "default": 12
                    },
                    "competitor": {
                        "type": "string",
                        "enum": ["muthoot", "manappuram", "iifl", "none"],
                        "description": "Competitor to compare against",
                        "default": "none"
                    }
                },
                "required": ["gold_weight_grams"]
            }),
            output_schema: None,
            category: Some("financial".to_string()),
            supports_streaming: false,
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, ToolError> {
        let gold_weight: f64 = input.get("gold_weight_grams")
            .and_then(|v| v.as_f64())
            .ok_or(ToolError {
                code: ErrorCode::InvalidInput,
                message: "gold_weight_grams is required".to_string(),
                details: None,
            })?;

        let purity = input.get("gold_purity")
            .and_then(|v| v.as_str())
            .unwrap_or("22K");

        let tenure_months = input.get("loan_tenure_months")
            .and_then(|v| v.as_i64())
            .unwrap_or(12) as u32;

        let competitor = input.get("competitor")
            .and_then(|v| v.as_str())
            .unwrap_or("none");

        // Calculate gold value
        let price_per_gram = self.rates.price_per_gram.get(purity)
            .copied()
            .unwrap_or(6875.0);
        let gold_value = gold_weight * price_per_gram;

        // Calculate Kotak loan
        let ltv = self.rates.ltv_ratios.get(purity).copied().unwrap_or(0.75);
        let max_loan = gold_value * ltv;
        let monthly_rate = self.rates.interest_rate / 12.0;
        let emi = calculate_emi(max_loan, monthly_rate, tenure_months);
        let total_interest = (emi * tenure_months as f64) - max_loan;
        let processing_fee = max_loan * self.rates.processing_fee_pct;

        let mut result = serde_json::json!({
            "gold_value": gold_value,
            "kotak": {
                "max_loan_amount": max_loan,
                "interest_rate": self.rates.interest_rate,
                "monthly_emi": emi,
                "total_interest": total_interest,
                "processing_fee": processing_fee,
                "total_cost": total_interest + processing_fee
            }
        });

        // Compare with competitor
        if competitor != "none" {
            if let Some(comp_rates) = self.competitor_rates.get(competitor) {
                let comp_max_loan = gold_value * comp_rates.ltv_ratio;
                let comp_monthly_rate = comp_rates.interest_rate / 12.0;
                let comp_emi = calculate_emi(comp_max_loan, comp_monthly_rate, tenure_months);
                let comp_total_interest = (comp_emi * tenure_months as f64) - comp_max_loan;
                let comp_processing_fee = comp_max_loan * comp_rates.processing_fee_pct;
                let comp_total_cost = comp_total_interest + comp_processing_fee;

                let kotak_total_cost = total_interest + processing_fee;
                let savings = comp_total_cost - kotak_total_cost;

                result["competitor"] = serde_json::json!({
                    "name": competitor,
                    "max_loan_amount": comp_max_loan,
                    "interest_rate": comp_rates.interest_rate,
                    "monthly_emi": comp_emi,
                    "total_interest": comp_total_interest,
                    "processing_fee": comp_processing_fee,
                    "total_cost": comp_total_cost
                });

                result["savings"] = serde_json::json!({
                    "total_savings": savings,
                    "savings_percentage": (savings / comp_total_cost) * 100.0,
                    "monthly_savings": (comp_emi - emi)
                });
            }
        }

        Ok(ToolResult::success(result))
    }
}

fn calculate_emi(principal: f64, monthly_rate: f64, months: u32) -> f64 {
    if monthly_rate == 0.0 {
        return principal / months as f64;
    }
    let r = monthly_rate;
    let n = months as f64;
    principal * r * (1.0 + r).powf(n) / ((1.0 + r).powf(n) - 1.0)
}
```

### Branch Locator

```rust
// crates/tools/src/domain/branch_locator.rs

/// Branch locator tool
pub struct BranchLocatorTool {
    branches: Vec<Branch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: String,
    pub name: String,
    pub address: String,
    pub city: String,
    pub pincode: String,
    pub latitude: f64,
    pub longitude: f64,
    pub phone: String,
    pub timings: String,
    pub services: Vec<String>,
}

impl BranchLocatorTool {
    pub fn new(branches: Vec<Branch>) -> Self {
        Self { branches }
    }
}

#[async_trait]
impl McpTool for BranchLocatorTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "branch_locator".to_string(),
            description: "Find nearest Kotak bank branches that offer gold loan services".to_string(),
            input_schema: serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {
                    "city": {
                        "type": "string",
                        "description": "City name"
                    },
                    "pincode": {
                        "type": "string",
                        "description": "Pincode/ZIP code",
                        "pattern": "^[0-9]{6}$"
                    },
                    "latitude": {
                        "type": "number",
                        "description": "Latitude for location-based search"
                    },
                    "longitude": {
                        "type": "number",
                        "description": "Longitude for location-based search"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of branches to return",
                        "default": 3,
                        "minimum": 1,
                        "maximum": 10
                    }
                },
                "anyOf": [
                    { "required": ["city"] },
                    { "required": ["pincode"] },
                    { "required": ["latitude", "longitude"] }
                ]
            }),
            output_schema: None,
            category: Some("location".to_string()),
            supports_streaming: false,
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, ToolError> {
        let limit = input.get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(3) as usize;

        let mut matching: Vec<(Branch, f64)> = Vec::new();

        // Search by city
        if let Some(city) = input.get("city").and_then(|v| v.as_str()) {
            let city_lower = city.to_lowercase();
            for branch in &self.branches {
                if branch.city.to_lowercase().contains(&city_lower) {
                    matching.push((branch.clone(), 0.0));
                }
            }
        }

        // Search by pincode
        if let Some(pincode) = input.get("pincode").and_then(|v| v.as_str()) {
            for branch in &self.branches {
                if branch.pincode == pincode {
                    matching.push((branch.clone(), 0.0));
                }
            }
        }

        // Search by location
        if let (Some(lat), Some(lon)) = (
            input.get("latitude").and_then(|v| v.as_f64()),
            input.get("longitude").and_then(|v| v.as_f64()),
        ) {
            for branch in &self.branches {
                let distance = haversine_distance(lat, lon, branch.latitude, branch.longitude);
                matching.push((branch.clone(), distance));
            }
            // Sort by distance
            matching.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        // Deduplicate and limit
        let mut seen = std::collections::HashSet::new();
        let results: Vec<Value> = matching
            .into_iter()
            .filter(|(b, _)| seen.insert(b.id.clone()))
            .take(limit)
            .map(|(branch, distance)| {
                let mut obj = serde_json::to_value(&branch).unwrap();
                if distance > 0.0 {
                    obj["distance_km"] = serde_json::json!(distance);
                }
                obj
            })
            .collect();

        Ok(ToolResult::success(serde_json::json!({
            "branches": results,
            "count": results.len()
        })))
    }
}

/// Haversine distance in kilometers
fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0; // Earth radius in km
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();
    let lat1 = lat1.to_radians();
    let lat2 = lat2.to_radians();

    let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();

    r * c
}
```

### Appointment Scheduler

```rust
// crates/tools/src/domain/appointment_scheduler.rs

/// Appointment scheduler tool
pub struct AppointmentSchedulerTool {
    available_slots: Arc<RwLock<HashMap<String, Vec<TimeSlot>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSlot {
    pub date: String,
    pub time: String,
    pub available: bool,
    pub branch_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appointment {
    pub id: String,
    pub customer_id: String,
    pub branch_id: String,
    pub date: String,
    pub time: String,
    pub service: String,
    pub status: AppointmentStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppointmentStatus {
    Confirmed,
    Pending,
    Cancelled,
    Completed,
}

#[async_trait]
impl McpTool for AppointmentSchedulerTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "appointment_scheduler".to_string(),
            description: "Schedule gold loan consultation appointments at Kotak branches".to_string(),
            input_schema: serde_json::json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list_slots", "book", "cancel"],
                        "description": "Action to perform"
                    },
                    "branch_id": {
                        "type": "string",
                        "description": "Branch ID for the appointment"
                    },
                    "date": {
                        "type": "string",
                        "description": "Date in YYYY-MM-DD format",
                        "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}$"
                    },
                    "time": {
                        "type": "string",
                        "description": "Time in HH:MM format",
                        "pattern": "^[0-9]{2}:[0-9]{2}$"
                    },
                    "customer_name": {
                        "type": "string",
                        "description": "Customer name for booking"
                    },
                    "customer_phone": {
                        "type": "string",
                        "description": "Customer phone for booking"
                    },
                    "appointment_id": {
                        "type": "string",
                        "description": "Appointment ID for cancellation"
                    }
                },
                "required": ["action"]
            }),
            output_schema: None,
            category: Some("scheduling".to_string()),
            supports_streaming: false,
        }
    }

    async fn execute(&self, input: Value) -> Result<ToolResult, ToolError> {
        let action = input.get("action")
            .and_then(|v| v.as_str())
            .ok_or(ToolError {
                code: ErrorCode::InvalidInput,
                message: "action is required".to_string(),
                details: None,
            })?;

        match action {
            "list_slots" => {
                let branch_id = input.get("branch_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");
                let date = input.get("date").and_then(|v| v.as_str());

                let slots = self.available_slots.read().await;
                let branch_slots = slots.get(branch_id).cloned().unwrap_or_default();

                let filtered: Vec<_> = branch_slots
                    .into_iter()
                    .filter(|s| s.available && date.map_or(true, |d| s.date == d))
                    .collect();

                Ok(ToolResult::success(serde_json::json!({
                    "slots": filtered
                })))
            }
            "book" => {
                let branch_id = input.get("branch_id")
                    .and_then(|v| v.as_str())
                    .ok_or(ToolError {
                        code: ErrorCode::InvalidInput,
                        message: "branch_id is required for booking".to_string(),
                        details: None,
                    })?;

                let date = input.get("date")
                    .and_then(|v| v.as_str())
                    .ok_or(ToolError {
                        code: ErrorCode::InvalidInput,
                        message: "date is required for booking".to_string(),
                        details: None,
                    })?;

                let time = input.get("time")
                    .and_then(|v| v.as_str())
                    .ok_or(ToolError {
                        code: ErrorCode::InvalidInput,
                        message: "time is required for booking".to_string(),
                        details: None,
                    })?;

                // In production, this would create a real appointment
                let appointment = Appointment {
                    id: uuid::Uuid::new_v4().to_string(),
                    customer_id: input.get("customer_phone")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    branch_id: branch_id.to_string(),
                    date: date.to_string(),
                    time: time.to_string(),
                    service: "gold_loan_consultation".to_string(),
                    status: AppointmentStatus::Confirmed,
                };

                Ok(ToolResult::success(serde_json::json!({
                    "appointment": appointment,
                    "message": "Appointment booked successfully"
                })))
            }
            "cancel" => {
                let appointment_id = input.get("appointment_id")
                    .and_then(|v| v.as_str())
                    .ok_or(ToolError {
                        code: ErrorCode::InvalidInput,
                        message: "appointment_id is required for cancellation".to_string(),
                        details: None,
                    })?;

                // In production, this would cancel the actual appointment
                Ok(ToolResult::success(serde_json::json!({
                    "appointment_id": appointment_id,
                    "status": "cancelled",
                    "message": "Appointment cancelled successfully"
                })))
            }
            _ => Err(ToolError {
                code: ErrorCode::InvalidInput,
                message: format!("Unknown action: {}", action),
                details: None,
            }),
        }
    }
}
```

---

## Tool Registry Setup

```rust
// crates/tools/src/lib.rs

/// Initialize tool registry with all domain tools
pub async fn create_tool_registry() -> Arc<ToolRegistry> {
    let registry = Arc::new(ToolRegistry::new());

    // Register financial tools
    registry.register(GoldLoanCalculatorTool::default()).await;

    // Register location tools
    let branches = load_branches().await.unwrap_or_default();
    registry.register(BranchLocatorTool::new(branches)).await;

    // Register scheduling tools
    registry.register(AppointmentSchedulerTool::default()).await;

    // Register eligibility checker
    registry.register(EligibilityCheckerTool::default()).await;

    registry
}

/// Format tools for LLM prompt
pub fn format_tools_for_prompt(tools: &[ToolDefinition]) -> String {
    let mut output = String::new();
    output.push_str("Available tools:\n\n");

    for tool in tools {
        output.push_str(&format!("## {}\n", tool.name));
        output.push_str(&format!("{}\n\n", tool.description));
        output.push_str("Parameters:\n");
        output.push_str(&format!("```json\n{}\n```\n\n",
            serde_json::to_string_pretty(&tool.input_schema).unwrap_or_default()
        ));
    }

    output
}
```

---

## Usage in Agent

```rust
// Example: Using tools in conversation agent

impl GoldLoanVoiceAgent {
    async fn handle_tool_call(
        &self,
        tool_name: &str,
        arguments: Value,
        state: &mut AgentState,
    ) -> Result<AgentResponse, Error> {
        let result = self.tools.execute(tool_name, arguments.clone()).await?;

        // Log tool usage
        tracing::info!(
            tool = tool_name,
            success = result.success,
            execution_ms = result.metadata.as_ref().map(|m| m.execution_time_ms),
            "Tool executed"
        );

        if result.success {
            // Format result for conversation
            let formatted = self.format_tool_result(tool_name, &result.data)?;

            // Update state with tool result
            state.entities.insert(
                format!("last_tool_{}", tool_name),
                result.data.clone().unwrap_or_default(),
            );

            Ok(AgentResponse {
                text: formatted,
                tool_results: vec![result],
                should_end_turn: false,
            })
        } else {
            // Handle tool error gracefully
            let error_message = result.error
                .as_ref()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Tool execution failed".to_string());

            Ok(AgentResponse {
                text: format!("I apologize, but I couldn't complete that action: {}", error_message),
                tool_results: vec![result],
                should_end_turn: false,
            })
        }
    }

    fn format_tool_result(&self, tool_name: &str, data: &Option<Value>) -> Result<String, Error> {
        let data = data.as_ref().ok_or(Error::NoToolData)?;

        match tool_name {
            "gold_loan_calculator" => {
                let kotak = &data["kotak"];
                let mut response = format!(
                    "Based on your gold, you can get a loan of up to Rs. {:.0}. ",
                    kotak["max_loan_amount"].as_f64().unwrap_or(0.0)
                );
                response.push_str(&format!(
                    "Your monthly EMI would be Rs. {:.0} at our rate of {:.1}%. ",
                    kotak["monthly_emi"].as_f64().unwrap_or(0.0),
                    kotak["interest_rate"].as_f64().unwrap_or(0.0) * 100.0
                ));

                if let Some(savings) = data.get("savings") {
                    response.push_str(&format!(
                        "You could save Rs. {:.0} compared to your current lender!",
                        savings["total_savings"].as_f64().unwrap_or(0.0)
                    ));
                }

                Ok(response)
            }
            "branch_locator" => {
                let branches = data["branches"].as_array().unwrap_or(&vec![]);
                if branches.is_empty() {
                    Ok("I couldn't find any branches in that area. Would you like to search in a different location?".to_string())
                } else {
                    let branch = &branches[0];
                    Ok(format!(
                        "The nearest branch is {} at {}. It's open {}. Shall I book an appointment for you?",
                        branch["name"].as_str().unwrap_or(""),
                        branch["address"].as_str().unwrap_or(""),
                        branch["timings"].as_str().unwrap_or("")
                    ))
                }
            }
            "appointment_scheduler" => {
                if let Some(appointment) = data.get("appointment") {
                    Ok(format!(
                        "Your appointment is confirmed for {} at {}. You'll receive an SMS confirmation shortly.",
                        appointment["date"].as_str().unwrap_or(""),
                        appointment["time"].as_str().unwrap_or("")
                    ))
                } else {
                    Ok(data["message"].as_str().unwrap_or("Done").to_string())
                }
            }
            _ => Ok(serde_json::to_string_pretty(data).unwrap_or_default()),
        }
    }
}
```
