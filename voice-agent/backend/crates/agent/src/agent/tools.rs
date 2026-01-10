//! Tool Calling Methods for DomainAgent
//!
//! This module contains tool-related functionality including:
//! - Intent-based tool invocation
//! - DST-enriched tool calls
//! - Tool argument mapping and defaults
//!
//! # P20 FIX: Config-Driven Tool Resolution
//!
//! All intent-to-tool mappings and tool defaults now come from configuration:
//! - `intent_tool_mappings.yaml` - Maps intents to tools with required/optional slots
//! - `tools/schemas.yaml` - Tool definitions with default values
//!
//! Legacy hardcoded fallbacks have been removed. If config is missing,
//! tools will not be called (fail-fast approach).

use super::DomainAgent;
use crate::agent_config::AgentEvent;
use crate::dst::DialogueStateTrait;
use crate::AgentError;
use voice_agent_tools::ToolExecutor;

impl DomainAgent {
    /// Maybe call a tool based on intent
    ///
    /// P20 FIX: Fully config-driven - NO hardcoded fallback mappings.
    /// All intent-to-tool mappings come from intent_tool_mappings.yaml.
    pub(super) async fn maybe_call_tool(
        &self,
        intent: &crate::intent::DetectedIntent,
    ) -> Result<Option<String>, AgentError> {
        // Collect available slot names
        let available_slots: Vec<&str> = intent.slots.keys().map(|s| s.as_str()).collect();

        // P20 FIX: Config-driven intent-to-tool resolution ONLY
        // No hardcoded fallbacks - if config is missing, no tool is called
        let tool_name = self.domain_view
            .as_ref()
            .and_then(|view| {
                if view.has_intent_mappings() {
                    view.resolve_tool_for_intent(&intent.intent, &available_slots)
                        .map(|s| s.to_string())
                } else {
                    // Log warning when config is missing
                    tracing::warn!(
                        intent = %intent.intent,
                        "No intent-to-tool mappings configured. Check intent_tool_mappings.yaml"
                    );
                    None
                }
            })
            .or_else(|| {
                // P20 FIX: No fallback - log and return None
                if self.domain_view.is_none() {
                    tracing::debug!(
                        intent = %intent.intent,
                        "DomainView not configured - tool resolution skipped"
                    );
                }
                None
            });

        if let Some(name) = tool_name {
            let _ = self.event_tx.send(AgentEvent::ToolCall {
                name: name.to_string(),
            });

            // Build arguments from slots
            let mut args = serde_json::Map::new();
            for (key, slot) in &intent.slots {
                if let Some(ref value) = slot.value {
                    args.insert(key.clone(), serde_json::json!(value));
                }
            }

            // P20 FIX: Config-driven tool defaults ONLY
            // All defaults and argument mappings come from tools/schemas.yaml
            if let Some(view) = self.domain_view.as_ref() {
                // Apply argument name mappings from config
                if let Some(arg_mapping) = view.get_argument_mapping(&name) {
                    let keys: Vec<String> = args.keys().cloned().collect();
                    for slot_name in keys {
                        if let Some(arg_name) = arg_mapping.get(&slot_name) {
                            if !args.contains_key(arg_name) {
                                if let Some(value) = args.remove(&slot_name) {
                                    args.insert(arg_name.clone(), value);
                                }
                            }
                        }
                    }
                }

                // Apply defaults from config
                if let Some(tool_defaults) = view.get_tool_defaults(&name) {
                    for (arg_name, default_value) in tool_defaults {
                        if !args.contains_key(arg_name) {
                            args.insert(arg_name.clone(), default_value.clone());
                        }
                    }
                }
            } else {
                // P20 FIX: Log warning when domain view is not configured
                tracing::warn!(
                    tool = %name,
                    "DomainView not configured - tool defaults not available. Check domain config."
                );
            }

            // P20 FIX: Apply generic slot-to-argument mappings
            // These are common mappings that don't depend on domain
            self.apply_common_argument_mappings(&mut args);

            // P20 FIX: Interest level default based on intent confidence
            // This is a generic behavior, not domain-specific
            if !args.contains_key("interest_level") && name.contains("capture") {
                let level = if intent.confidence > 0.8 { "High" } else { "Medium" };
                args.insert("interest_level".to_string(), serde_json::json!(level));
            }

            let result = self
                .tools
                .execute(&name, serde_json::Value::Object(args))
                .await;

            let success = result.is_ok();
            let _ = self.event_tx.send(AgentEvent::ToolResult {
                name: name.to_string(),
                success,
            });

            match result {
                Ok(output) => {
                    // Extract text from output
                    let text = output
                        .content
                        .iter()
                        .filter_map(|c| match c {
                            voice_agent_tools::mcp::ContentBlock::Text { text } => {
                                Some(text.clone())
                            }
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    Ok(Some(text))
                }
                Err(e) => {
                    tracing::warn!("Tool error: {}", e);
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Call a tool by name using DST state for arguments (Phase 12 - proactive tool triggering)
    pub(super) async fn call_tool_by_name(
        &self,
        tool_name: &str,
        intent: &crate::intent::DetectedIntent,
    ) -> Result<Option<String>, AgentError> {
        let _ = self.event_tx.send(AgentEvent::ToolCall {
            name: tool_name.to_string(),
        });

        // Build arguments from DST state (more complete than just current intent slots)
        let mut args = serde_json::Map::new();

        // First, add slots from the current intent
        for (key, slot) in &intent.slots {
            if let Some(ref value) = slot.value {
                args.insert(key.clone(), serde_json::json!(value));
            }
        }

        // Then, enrich with DST state values that may have been collected over multiple turns
        {
            let dst = self.dialogue_state.read();
            let state = dst.state();

            // Map DST slot names to tool argument names
            // Uses generic get_slot_value() for domain-agnostic slot access
            if let Some(val) = state.customer_name() {
                args.entry("customer_name".to_string())
                    .or_insert(serde_json::json!(val));
            }
            if let Some(val) = state.phone_number() {
                args.entry("phone_number".to_string())
                    .or_insert(serde_json::json!(val));
            }
            if let Some(val) = state.location() {
                args.entry("city".to_string())
                    .or_insert(serde_json::json!(val));
            }
            // P19 FIX: Try generic slot names first, then legacy names
            // Generic names are defined in slots.yaml, legacy names for backwards compat
            if let Some(val) = state.get_slot_value("asset_quantity")
                .or_else(|| state.get_slot_value("collateral_weight")) {
                args.entry("collateral_weight".to_string())
                    .or_insert(serde_json::json!(val));
            }
            if let Some(val) = state.get_slot_value("asset_quality")
                .or_else(|| state.get_slot_value("collateral_variant")) {
                args.entry("collateral_variant".to_string())
                    .or_insert(serde_json::json!(val));
            }
            if let Some(val) = state.get_slot_value("loan_amount") {
                args.entry("loan_amount".to_string())
                    .or_insert(serde_json::json!(val));
                args.entry("current_loan_amount".to_string())
                    .or_insert(serde_json::json!(val));
            }
            if let Some(val) = state.get_slot_value("current_lender") {
                args.entry("current_lender".to_string())
                    .or_insert(serde_json::json!(val));
            }
            if let Some(val) = state.get_slot_value("current_interest_rate") {
                args.entry("current_interest_rate".to_string())
                    .or_insert(serde_json::json!(val));
            }
            if let Some(val) = state.get_slot_value("loan_tenure") {
                args.entry("remaining_tenure_months".to_string())
                    .or_insert(serde_json::json!(val));
            }
        }

        // P20 FIX: Config-driven defaults ONLY
        if let Some(view) = self.domain_view.as_ref() {
            // Apply argument name mappings from config
            if let Some(arg_mapping) = view.get_argument_mapping(tool_name) {
                let keys: Vec<String> = args.keys().cloned().collect();
                for slot_name in keys {
                    if let Some(arg_name) = arg_mapping.get(&slot_name) {
                        if !args.contains_key(arg_name) {
                            if let Some(value) = args.remove(&slot_name) {
                                args.insert(arg_name.clone(), value);
                            }
                        }
                    }
                }
            }

            // Apply defaults from config
            if let Some(tool_defaults) = view.get_tool_defaults(tool_name) {
                for (arg_name, default_value) in tool_defaults {
                    if !args.contains_key(arg_name) {
                        args.insert(arg_name.clone(), default_value.clone());
                    }
                }
            }
        } else {
            // P20 FIX: Log warning when domain view is not configured
            tracing::warn!(
                tool = %tool_name,
                "DomainView not configured - tool defaults not available. Check domain config."
            );
        }

        // P20 FIX: Apply generic slot-to-argument mappings
        self.apply_common_argument_mappings(&mut args);

        // P20 FIX: Interest level default (generic behavior)
        if tool_name.contains("capture") && !args.contains_key("interest_level") {
            // Default interest level to High for proactive capture
            args.insert("interest_level".to_string(), serde_json::json!("High"));
        }

        tracing::debug!(
            tool = tool_name,
            args = ?args,
            "Calling tool proactively with DST state"
        );

        let result = self
            .tools
            .execute(tool_name, serde_json::Value::Object(args))
            .await;

        let success = result.is_ok();
        let _ = self.event_tx.send(AgentEvent::ToolResult {
            name: tool_name.to_string(),
            success,
        });

        match result {
            Ok(output) => {
                let text = output
                    .content
                    .iter()
                    .filter_map(|c| match c {
                        voice_agent_tools::mcp::ContentBlock::Text { text } => Some(text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(Some(text))
            }
            Err(e) => {
                tracing::warn!("Proactive tool error: {}", e);
                Ok(None)
            }
        }
    }

    /// Apply common slot-to-argument mappings
    ///
    /// P20 FIX: Uses config-driven common mappings when available.
    /// Falls back to hardcoded mappings only when domain_view is not configured.
    fn apply_common_argument_mappings(&self, args: &mut serde_json::Map<String, serde_json::Value>) {
        // P20 FIX: Try config-driven common mappings first
        if let Some(ref view) = self.domain_view {
            let common_mappings = view.get_common_argument_mappings();
            if !common_mappings.is_empty() {
                let keys: Vec<String> = args.keys().cloned().collect();
                for short_name in keys {
                    if let Some(standard_name) = common_mappings.get(&short_name) {
                        if !args.contains_key(standard_name) {
                            if let Some(v) = args.remove(&short_name) {
                                args.insert(standard_name.clone(), v);
                            }
                        }
                    }
                }
                return;
            }
        }

        // Fallback: hardcoded common mappings (deprecated - prefer config)
        let common_mappings = [
            ("name", "customer_name"),
            ("phone", "phone_number"),
            ("date", "preferred_date"),
            ("time", "preferred_time"),
            ("branch", "branch_id"),
            ("location", "city"),
        ];

        for (short_name, standard_name) in common_mappings {
            if args.contains_key(short_name) && !args.contains_key(standard_name) {
                if let Some(v) = args.remove(short_name) {
                    args.insert(standard_name.to_string(), v);
                }
            }
        }
    }
}
