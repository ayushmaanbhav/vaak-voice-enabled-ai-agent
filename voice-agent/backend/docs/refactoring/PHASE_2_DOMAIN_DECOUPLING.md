# Phase 2: Domain Decoupling

**Priority:** P1 - Core refactoring for domain-agnosticism
**Estimated Files:** 25 files
**Dependencies:** Phase 1 complete

---

## Overview

This phase removes all gold-loan-specific code from core modules and makes everything config-driven:
1. Make tool definitions config-driven
2. Make system prompts config-driven
3. Make slot extraction config-driven
4. Create generic DialogueState

---

## Task 2.1: Make Tool Definitions Config-Driven

### Problem
`llm/src/prompt.rs` has `gold_loan_tools()` function (lines 119-188) with hardcoded tool definitions.

### Files to Modify
- `crates/llm/src/prompt.rs` - Remove hardcoded tools
- `crates/config/src/domain/tools.rs` - Enhance tool config loading
- `config/domains/gold_loan/tools/schemas.yaml` - Ensure complete definitions

---

#### 2.1.1 Remove gold_loan_tools() Function

**File:** `crates/llm/src/prompt.rs`

**Delete lines 119-188** (entire `gold_loan_tools()` function)

**Replace with config-driven loading:**

```rust
use voice_agent_config::domain::ToolsConfig;

/// Load tool definitions from domain config
pub fn tools_from_config(tools_config: &ToolsConfig) -> Vec<ToolDefinition> {
    tools_config.tools.iter().map(|(name, schema)| {
        ToolDefinition {
            name: name.clone(),
            description: schema.description.clone(),
            parameters: schema.parameters.clone(),
        }
    }).collect()
}
```

#### 2.1.2 Update PromptBuilder to Accept Tool Config

**File:** `crates/llm/src/prompt.rs`

**Modify PromptBuilder struct:**

```rust
pub struct PromptBuilder {
    // ... existing fields ...
    tools_config: Option<ToolsConfig>,
}

impl PromptBuilder {
    pub fn with_tools_config(mut self, config: ToolsConfig) -> Self {
        self.tools_config = Some(config);
        self
    }

    pub fn build_tools(&self) -> Vec<ToolDefinition> {
        match &self.tools_config {
            Some(config) => tools_from_config(config),
            None => Vec::new(),
        }
    }
}
```

#### 2.1.3 Enhance tools/schemas.yaml

**File:** `config/domains/gold_loan/tools/schemas.yaml`

Ensure all 10 tools are fully defined:

```yaml
tools:
  check_eligibility:
    description: "Check if customer is eligible for a loan based on their collateral"
    category: "calculation"
    parameters:
      type: object
      required: ["collateral_weight", "collateral_quality"]
      properties:
        collateral_weight:
          type: number
          description: "Weight of collateral in grams"
          minimum: 1
          maximum: 10000
        collateral_quality:
          type: string
          description: "Quality/purity of collateral"
          enum: ["K24", "K22", "K18", "K14"]
        loan_amount:
          type: number
          description: "Desired loan amount"

  calculate_savings:
    description: "Calculate monthly savings when switching from competitor"
    category: "comparison"
    parameters:
      type: object
      required: ["current_lender", "loan_amount"]
      properties:
        current_lender:
          type: string
          description: "Name of current lender"
        loan_amount:
          type: number
          description: "Loan amount for comparison"
        current_rate:
          type: number
          description: "Current interest rate if known"

  find_branches:
    description: "Find nearby branches that offer this service"
    category: "location"
    parameters:
      type: object
      required: ["city"]
      properties:
        city:
          type: string
          description: "City to search in"
        pincode:
          type: string
          description: "Pincode for more precise results"
        service_type:
          type: string
          enum: ["doorstep", "branch_visit"]

  get_current_price:
    description: "Get current market price for collateral"
    category: "market_data"
    parameters:
      type: object
      properties:
        quality:
          type: string
          description: "Quality grade"

  compare_competitors:
    description: "Compare our offering with a specific competitor"
    category: "comparison"
    parameters:
      type: object
      required: ["competitor_name"]
      properties:
        competitor_name:
          type: string
          description: "Name of competitor to compare"
        loan_amount:
          type: number
          description: "Loan amount for comparison"

  capture_lead:
    description: "Capture customer information for follow-up"
    category: "crm"
    parameters:
      type: object
      required: ["customer_name", "phone_number"]
      properties:
        customer_name:
          type: string
        phone_number:
          type: string
          pattern: "^[0-9]{10}$"
        interest_level:
          type: string
          enum: ["high", "medium", "low"]

  schedule_appointment:
    description: "Schedule an appointment at a branch"
    category: "scheduling"
    parameters:
      type: object
      required: ["branch_id", "preferred_date", "preferred_time"]
      properties:
        branch_id:
          type: string
        preferred_date:
          type: string
          format: date
        preferred_time:
          type: string

  send_sms:
    description: "Send SMS to customer with information"
    category: "communication"
    parameters:
      type: object
      required: ["phone_number", "template_type"]
      properties:
        phone_number:
          type: string
        template_type:
          type: string
          enum: ["appointment_confirmation", "rate_info", "branch_details"]

  get_document_checklist:
    description: "Get list of required documents"
    category: "information"
    parameters:
      type: object
      properties:
        loan_type:
          type: string
        customer_type:
          type: string

  escalate_to_human:
    description: "Transfer call to human agent"
    category: "escalation"
    parameters:
      type: object
      required: ["reason"]
      properties:
        reason:
          type: string
        priority:
          type: string
          enum: ["high", "normal", "low"]
```

#### 2.1.4 Update Tool Loading in ToolsConfig

**File:** `crates/config/src/domain/tools.rs`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolSchema {
    pub description: String,
    pub category: String,
    pub parameters: serde_json::Value,

    #[serde(default)]
    pub requires_integration: Option<String>,
}

impl ToolsConfig {
    /// Get tool schemas suitable for LLM
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|(name, schema)| {
            ToolDefinition {
                name: name.clone(),
                description: schema.description.clone(),
                parameters: schema.parameters.clone(),
            }
        }).collect()
    }

    /// Get tools by category
    pub fn tools_by_category(&self, category: &str) -> Vec<&str> {
        self.tools.iter()
            .filter(|(_, s)| s.category == category)
            .map(|(n, _)| n.as_str())
            .collect()
    }
}
```

### Verification
```bash
cargo check -p voice-agent-llm
cargo test -p voice-agent-llm
```

---

## Task 2.2: Make System Prompt Config-Driven

### Problem
`llm/src/prompt.rs` lines 303-345 have hardcoded:
- "Gold Loan specialist"
- "Kotak Mahindra Bank"
- "Switch & Save program"
- Product-specific messaging

### Files to Modify
- `crates/llm/src/prompt.rs`
- `config/domains/gold_loan/prompts/system.yaml`

---

#### 2.2.1 Create Prompt Template Structure

**File:** `config/domains/gold_loan/prompts/system.yaml`

```yaml
# System prompt templates - fully parameterized
templates:
  base_persona: |
    You are {agent_name}, a friendly and knowledgeable {agent_role} at {bank_name}.

    Your communication style:
    - Warmth level: {warmth}% (scale: formal=0, warm=100)
    - Formality level: {formality}%
    - Empathy level: {empathy}%

    Language: Respond in {language_style}.

  key_facts: |
    Key Facts about our offering:
    - Interest rates: Starting from {min_rate}% p.a. (vs {competitor_rate_range} at competitors)
    - Maximum loan value: {ltv_percent}% of collateral value
    - Processing time: {processing_time}
    - Processing fee: {processing_fee}%

    {additional_facts}

  stage_guidance:
    greeting: |
      Warmly greet the customer. Introduce yourself and ask how you can help.
      Keep it brief and friendly.

    discovery: |
      Ask open questions to understand the customer's needs.
      Listen actively. Don't pitch yet.
      Key questions: Why are they interested? What's their timeline? Any concerns?

    qualification: |
      Gather specific details needed to provide a quote.
      Required information: {required_slots}

    presentation: |
      Present our offering tailored to their needs.
      Focus on: {key_benefits}
      Address their specific situation.

    objection_handling: |
      Listen to concerns empathetically.
      Address objections with facts, not pressure.
      Available responses: {objection_types}

    closing: |
      Summarize the conversation.
      Provide clear next steps.
      Thank them for their time.

  language_styles:
    en: "Clear, professional English"
    hi: "Hindi-English mix (Hinglish) that feels natural"
    ta: "Respectful Tamil with English technical terms"
    te: "Conversational Telugu"

# Response templates by stage
response_templates:
  greeting:
    en: "Hello! I'm {agent_name} from {bank_name}. How may I assist you today?"
    hi: "Namaste! Main {agent_name} hoon, {bank_name} se. Aapki kya madad kar sakti hoon?"

  closing:
    en: "Thank you for your time. Please feel free to call if you have any questions."
    hi: "Dhanyavaad aapka samay dene ke liye. Koi bhi sawal ho toh zaroor call karein."
```

#### 2.2.2 Refactor PromptBuilder to Use Templates

**File:** `crates/llm/src/prompt.rs`

**Remove hardcoded system_prompt() method. Replace with:**

```rust
impl PromptBuilder {
    /// Build system prompt from config templates
    pub fn system_prompt_from_config(
        &self,
        prompts_config: &PromptsConfig,
        brand: &BrandConfig,
        stage: &str,
        language: &str,
    ) -> String {
        let mut prompt = String::new();

        // Build persona section
        let persona = prompts_config.template("base_persona")
            .replace("{agent_name}", &brand.agent_name)
            .replace("{agent_role}", &brand.agent_role)
            .replace("{bank_name}", &brand.bank_name)
            .replace("{warmth}", &format!("{}", self.persona.warmth * 100.0))
            .replace("{formality}", &format!("{}", self.persona.formality * 100.0))
            .replace("{empathy}", &format!("{}", self.persona.empathy * 100.0))
            .replace("{language_style}", prompts_config.language_style(language));
        prompt.push_str(&persona);
        prompt.push_str("\n\n");

        // Build key facts section
        if let Some(facts) = &self.product_facts {
            let key_facts = prompts_config.template("key_facts")
                .replace("{min_rate}", &format!("{:.1}", facts.our_rate))
                .replace("{competitor_rate_range}", &format!("{:.0}-{:.0}%", facts.nbfc_rate_low, facts.nbfc_rate_high))
                .replace("{ltv_percent}", &format!("{:.0}", facts.ltv_percent))
                .replace("{processing_time}", "30 minutes")  // TODO: from config
                .replace("{processing_fee}", "1.0");  // TODO: from config
            prompt.push_str(&key_facts);
            prompt.push_str("\n\n");
        }

        // Build stage guidance
        if let Some(guidance) = prompts_config.stage_guidance(stage) {
            prompt.push_str("Current Stage Guidance:\n");
            prompt.push_str(guidance);
        }

        prompt
    }
}
```

#### 2.2.3 Update PromptsConfig

**File:** `crates/config/src/domain/prompts.rs`

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PromptsConfig {
    pub templates: HashMap<String, String>,
    pub stage_guidance: HashMap<String, String>,
    pub response_templates: HashMap<String, HashMap<String, String>>,
    pub language_styles: HashMap<String, String>,
}

impl PromptsConfig {
    pub fn template(&self, name: &str) -> String {
        self.templates.get(name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn stage_guidance(&self, stage: &str) -> Option<&str> {
        self.stage_guidance.get(stage).map(|s| s.as_str())
    }

    pub fn response_template(&self, stage: &str, language: &str) -> Option<&str> {
        self.response_templates
            .get(stage)
            .and_then(|langs| langs.get(language).or_else(|| langs.get("en")))
            .map(|s| s.as_str())
    }

    pub fn language_style(&self, language: &str) -> &str {
        self.language_styles
            .get(language)
            .map(|s| s.as_str())
            .unwrap_or("Professional English")
    }
}
```

#### 2.2.4 Remove Hardcoded Greetings

**File:** `crates/llm/src/prompt.rs`

**Delete lines 709-793** (hardcoded greeting/closing functions)

**Replace with:**
```rust
impl ResponseTemplates {
    pub fn greeting_from_config(
        prompts: &PromptsConfig,
        agent_name: &str,
        bank_name: &str,
        language: &str,
    ) -> String {
        prompts.response_template("greeting", language)
            .map(|t| t.replace("{agent_name}", agent_name)
                      .replace("{bank_name}", bank_name))
            .unwrap_or_else(|| format!("Hello! I'm {} from {}.", agent_name, bank_name))
    }

    pub fn closing_from_config(prompts: &PromptsConfig, language: &str) -> String {
        prompts.response_template("closing", language)
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Thank you for your time.".to_string())
    }
}
```

### Verification
```bash
cargo check -p voice-agent-llm
cargo test -p voice-agent-llm
```

---

## Task 2.3: Make Slot Extraction Config-Driven

### Problem
`text_processing/src/slot_extraction/mod.rs` has hardcoded:
- Gold-loan slot names (gold_weight, gold_purity)
- Lender patterns (Muthoot, Manappuram)
- Intent patterns (gold_price_inquiry)

### Files to Modify
- `crates/text_processing/src/slot_extraction/mod.rs`
- `config/domains/gold_loan/slots.yaml`
- `config/domains/gold_loan/goals.yaml`

---

#### 2.3.1 Create Pattern Config Structure

**File:** `config/domains/gold_loan/extraction_patterns.yaml` (NEW FILE)

```yaml
# Slot extraction patterns - domain-specific
amount_patterns:
  - pattern: '(\d+(?:\.\d+)?)\s*(?:crore|cr|करोड़)'
    multiplier: 10000000
    description: "Crore (1 crore = 10 million)"
  - pattern: '(\d+(?:\.\d+)?)\s*(?:lakh|lac|लाख)'
    multiplier: 100000
    description: "Lakh (1 lakh = 100,000)"
  - pattern: '(\d+(?:\.\d+)?)\s*(?:thousand|k|हज़ार)'
    multiplier: 1000
    description: "Thousand"
  - pattern: '(?:₹|rs\.?|rupees?)\s*(\d+(?:,\d+)*(?:\.\d+)?)'
    multiplier: 1
    description: "Direct rupee amount"

weight_patterns:
  - pattern: '(\d+(?:\.\d+)?)\s*(?:gram|gm|g|ग्राम)'
    unit: "grams"
  - pattern: '(\d+(?:\.\d+)?)\s*(?:tola|तोला)'
    multiplier: 11.66
    unit: "grams"

quality_patterns:
  - pattern: '(?:24\s*(?:k|karat|carat)|शुद्ध\s*सोना)'
    value: "K24"
  - pattern: '(?:22\s*(?:k|karat|carat)|बाईस\s*कैरेट)'
    value: "K22"
  - pattern: '(?:18\s*(?:k|karat|carat))'
    value: "K18"

lender_patterns:
  muthoot:
    - "muthoot"
    - "muthut"
    - "muthoot finance"
    - "मुथूट"
  manappuram:
    - "manappuram"
    - "manapuram"
    - "manappuram gold"
    - "मनप्पुरम"
  iifl:
    - "iifl"
    - "india infoline"
  kotak:
    - "kotak"
    - "kotak mahindra"
    - "कोटक"

intent_patterns:
  balance_transfer:
    patterns:
      - 'balance\s+transfer'
      - 'loan\s+transfer'
      - 'switch.*(?:from|to)'
      - 'बैलेंस\s*ट्रांसफर'
    confidence: 0.9

  price_inquiry:
    patterns:
      - '(?:gold\s+)?(?:price|rate)'
      - 'सोने\s+का\s+(?:rate|भाव)'
      - 'current\s+(?:price|rate)'
    confidence: 0.85

  rate_inquiry:
    patterns:
      - 'interest\s+rate'
      - 'ब्याज\s+दर'
      - 'what.*rate'
    confidence: 0.9

  eligibility_inquiry:
    patterns:
      - 'eligib(?:le|ility)'
      - 'qualify'
      - 'can\s+i\s+get'
      - 'कितना\s+मिलेगा'
    confidence: 0.85
```

#### 2.3.2 Refactor SlotExtractor to Use Config

**File:** `crates/text_processing/src/slot_extraction/mod.rs`

**Remove hardcoded static patterns (lines 13-176)**

**Replace with config-driven approach:**

```rust
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ExtractionConfig {
    pub amount_patterns: Vec<AmountPattern>,
    pub weight_patterns: Vec<WeightPattern>,
    pub quality_patterns: Vec<QualityPattern>,
    pub lender_patterns: HashMap<String, Vec<String>>,
    pub intent_patterns: HashMap<String, IntentPattern>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AmountPattern {
    pub pattern: String,
    pub multiplier: f64,
    #[serde(skip)]
    compiled: OnceCell<Regex>,
}

impl AmountPattern {
    pub fn regex(&self) -> &Regex {
        self.compiled.get_or_init(|| {
            Regex::new(&self.pattern).expect("Invalid amount pattern")
        })
    }
}

pub struct ConfigurableSlotExtractor {
    config: ExtractionConfig,
    compiled_lender_patterns: HashMap<String, Vec<Regex>>,
    compiled_intent_patterns: Vec<(Regex, String, f32)>,
}

impl ConfigurableSlotExtractor {
    pub fn from_config(config: ExtractionConfig) -> Self {
        // Pre-compile lender patterns
        let compiled_lender_patterns = config.lender_patterns.iter()
            .map(|(lender, patterns)| {
                let compiled = patterns.iter()
                    .filter_map(|p| Regex::new(&format!("(?i){}", p)).ok())
                    .collect();
                (lender.clone(), compiled)
            })
            .collect();

        // Pre-compile intent patterns
        let compiled_intent_patterns = config.intent_patterns.iter()
            .flat_map(|(intent, pattern)| {
                pattern.patterns.iter()
                    .filter_map(|p| Regex::new(&format!("(?i){}", p)).ok())
                    .map(|r| (r, intent.clone(), pattern.confidence))
                    .collect::<Vec<_>>()
            })
            .collect();

        Self {
            config,
            compiled_lender_patterns,
            compiled_intent_patterns,
        }
    }

    pub fn extract(&self, utterance: &str, slots_config: &SlotsConfig) -> ExtractedSlots {
        let mut slots = HashMap::new();

        // Extract amounts using config patterns
        if let Some((amount, confidence)) = self.extract_amount(utterance) {
            let slot_name = slots_config.amount_slot_name();
            slots.insert(slot_name, SlotValue::Number { value: amount, confidence });
        }

        // Extract weight using config patterns
        if let Some((weight, confidence)) = self.extract_weight(utterance) {
            let slot_name = slots_config.weight_slot_name();
            slots.insert(slot_name, SlotValue::Number { value: weight, confidence });
        }

        // Extract lender using config patterns
        if let Some((lender, confidence)) = self.extract_lender(utterance) {
            let slot_name = slots_config.lender_slot_name();
            slots.insert(slot_name, SlotValue::String { value: lender, confidence });
        }

        ExtractedSlots { slots, intents: self.extract_intents(utterance) }
    }

    fn extract_amount(&self, text: &str) -> Option<(f64, f32)> {
        for pattern in &self.config.amount_patterns {
            if let Some(caps) = pattern.regex().captures(text) {
                if let Some(num_str) = caps.get(1) {
                    let clean = num_str.as_str().replace(",", "");
                    if let Ok(num) = clean.parse::<f64>() {
                        return Some((num * pattern.multiplier, 0.9));
                    }
                }
            }
        }
        None
    }

    fn extract_lender(&self, text: &str) -> Option<(String, f32)> {
        let lower = text.to_lowercase();
        for (lender, patterns) in &self.compiled_lender_patterns {
            for pattern in patterns {
                if pattern.is_match(&lower) {
                    return Some((lender.clone(), 0.95));
                }
            }
        }
        None
    }

    fn extract_intents(&self, text: &str) -> Vec<(String, f32)> {
        let lower = text.to_lowercase();
        self.compiled_intent_patterns.iter()
            .filter(|(regex, _, _)| regex.is_match(&lower))
            .map(|(_, intent, conf)| (intent.clone(), *conf))
            .collect()
    }
}
```

#### 2.3.3 Update SlotsConfig with Slot Name Mappings

**File:** `crates/config/src/domain/slots.rs`

```rust
impl SlotsConfig {
    /// Get the slot name for amount values (domain-specific)
    pub fn amount_slot_name(&self) -> String {
        self.slot_mappings
            .get("amount")
            .cloned()
            .unwrap_or_else(|| "loan_amount".to_string())
    }

    /// Get the slot name for weight/quantity values
    pub fn weight_slot_name(&self) -> String {
        self.slot_mappings
            .get("weight")
            .cloned()
            .unwrap_or_else(|| "collateral_weight".to_string())
    }

    /// Get the slot name for lender values
    pub fn lender_slot_name(&self) -> String {
        self.slot_mappings
            .get("lender")
            .cloned()
            .unwrap_or_else(|| "current_lender".to_string())
    }
}
```

#### 2.3.4 Add Slot Mappings to slots.yaml

**File:** `config/domains/gold_loan/slots.yaml`

```yaml
# Add at top of file
slot_mappings:
  amount: "loan_amount"
  weight: "gold_weight_grams"
  quality: "gold_purity"
  lender: "current_lender"
  rate: "current_interest_rate"

# ... rest of existing slots config
```

### Verification
```bash
cargo check -p voice-agent-text-processing
cargo test -p voice-agent-text-processing
```

---

## Task 2.4: Create Generic DialogueState

### Problem
`agent/src/dst/slots.rs` has `GoldLoanDialogueState` struct (lines 187-320) with 15 gold-loan-specific fields.

### Files to Modify
- `crates/agent/src/dst/slots.rs`
- `crates/agent/src/dst/mod.rs`

---

#### 2.4.1 Create Generic DynamicDialogueState

**File:** `crates/agent/src/dst/dynamic_state.rs` (NEW FILE)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dynamic slot value that can hold different types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum SlotValue {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    StringList(Vec<String>),
    Null,
}

impl SlotValue {
    pub fn as_string(&self) -> Option<&str> {
        match self {
            SlotValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            SlotValue::Number(n) => Some(*n),
            SlotValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SlotValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, SlotValue::Null)
    }
}

/// Generic dialogue state that works for any domain
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DynamicDialogueState {
    /// Current conversation stage
    pub stage: String,

    /// Detected customer intent
    pub intent: Option<String>,

    /// Dynamic slots storage
    slots: HashMap<String, SlotValue>,

    /// Slot confidence scores
    confidences: HashMap<String, f32>,

    /// Conversation metadata
    pub turn_count: usize,
    pub last_updated_ms: u64,
}

impl DynamicDialogueState {
    pub fn new() -> Self {
        Self {
            stage: "greeting".to_string(),
            ..Default::default()
        }
    }

    /// Set a slot value
    pub fn set_slot(&mut self, name: &str, value: SlotValue, confidence: f32) {
        self.slots.insert(name.to_string(), value);
        self.confidences.insert(name.to_string(), confidence);
        self.last_updated_ms = current_time_ms();
    }

    /// Get a slot value
    pub fn get_slot(&self, name: &str) -> Option<&SlotValue> {
        self.slots.get(name)
    }

    /// Get slot as string
    pub fn get_string(&self, name: &str) -> Option<&str> {
        self.slots.get(name).and_then(|v| v.as_string())
    }

    /// Get slot as number
    pub fn get_number(&self, name: &str) -> Option<f64> {
        self.slots.get(name).and_then(|v| v.as_number())
    }

    /// Get slot confidence
    pub fn get_confidence(&self, name: &str) -> f32 {
        self.confidences.get(name).copied().unwrap_or(0.0)
    }

    /// Check if slot is filled
    pub fn has_slot(&self, name: &str) -> bool {
        self.slots.get(name).map(|v| !v.is_null()).unwrap_or(false)
    }

    /// Get all filled slot names
    pub fn filled_slots(&self) -> Vec<&str> {
        self.slots.iter()
            .filter(|(_, v)| !v.is_null())
            .map(|(k, _)| k.as_str())
            .collect()
    }

    /// Get missing slots for a goal (from config)
    pub fn missing_slots_for_goal(&self, goal: &GoalDefinition) -> Vec<&str> {
        goal.required_slots()
            .iter()
            .filter(|s| !self.has_slot(s))
            .copied()
            .collect()
    }

    /// Clear all slots
    pub fn clear(&mut self) {
        self.slots.clear();
        self.confidences.clear();
        self.intent = None;
    }

    /// Export to JSON for logging/persistence
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "stage": self.stage,
            "intent": self.intent,
            "slots": self.slots,
            "turn_count": self.turn_count,
        })
    }
}

fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
```

#### 2.4.2 Create Compatibility Layer for GoldLoanDialogueState

**File:** `crates/agent/src/dst/slots.rs`

Add a compatibility wrapper:

```rust
/// Compatibility wrapper that exposes GoldLoanDialogueState API
/// but uses DynamicDialogueState internally
#[deprecated(since = "0.2.0", note = "Use DynamicDialogueState directly")]
pub struct GoldLoanDialogueStateCompat {
    inner: DynamicDialogueState,
}

impl GoldLoanDialogueStateCompat {
    pub fn new() -> Self {
        Self { inner: DynamicDialogueState::new() }
    }

    // Compatibility getters
    pub fn gold_weight_grams(&self) -> Option<f64> {
        self.inner.get_number("gold_weight_grams")
    }

    pub fn gold_purity(&self) -> Option<&str> {
        self.inner.get_string("gold_purity")
    }

    pub fn loan_amount(&self) -> Option<f64> {
        self.inner.get_number("loan_amount")
    }

    pub fn current_lender(&self) -> Option<&str> {
        self.inner.get_string("current_lender")
    }

    // Compatibility setters
    pub fn set_gold_weight(&mut self, weight: f64, confidence: f32) {
        self.inner.set_slot("gold_weight_grams", SlotValue::Number(weight), confidence);
    }

    pub fn set_gold_purity(&mut self, purity: &str, confidence: f32) {
        self.inner.set_slot("gold_purity", SlotValue::String(purity.to_string()), confidence);
    }

    // ... other compatibility methods

    /// Get underlying dynamic state
    pub fn into_dynamic(self) -> DynamicDialogueState {
        self.inner
    }
}
```

#### 2.4.3 Update DialogueStateTracker to Use DynamicDialogueState

**File:** `crates/agent/src/dst/mod.rs`

```rust
use crate::dst::dynamic_state::DynamicDialogueState;

pub struct DialogueStateTracker {
    state: RwLock<DynamicDialogueState>,
    slots_config: Arc<SlotsConfig>,
    goals_config: Arc<GoalsConfig>,
    extractor: Arc<ConfigurableSlotExtractor>,
}

impl DialogueStateTracker {
    pub fn new(
        slots_config: Arc<SlotsConfig>,
        goals_config: Arc<GoalsConfig>,
        extraction_config: ExtractionConfig,
    ) -> Self {
        Self {
            state: RwLock::new(DynamicDialogueState::new()),
            slots_config,
            goals_config,
            extractor: Arc::new(ConfigurableSlotExtractor::from_config(extraction_config)),
        }
    }

    pub fn update(&self, utterance: &str) {
        let extracted = self.extractor.extract(utterance, &self.slots_config);
        let mut state = self.state.write();

        for (slot_name, slot_value) in extracted.slots {
            state.set_slot(&slot_name, slot_value.value, slot_value.confidence);
        }

        if let Some((intent, _)) = extracted.intents.first() {
            state.intent = Some(intent.clone());
        }

        state.turn_count += 1;
    }

    pub fn current_state(&self) -> DynamicDialogueState {
        self.state.read().clone()
    }

    pub fn missing_slots_for_current_goal(&self) -> Vec<String> {
        let state = self.state.read();
        if let Some(intent) = &state.intent {
            if let Some(goal) = self.goals_config.goal_for_intent(intent) {
                return goal.required_slots()
                    .iter()
                    .filter(|s| !state.has_slot(s))
                    .map(|s| s.to_string())
                    .collect();
            }
        }
        Vec::new()
    }
}
```

### Verification
```bash
cargo check -p voice-agent-agent
cargo test -p voice-agent-agent dst
```

---

## Task 2.5: Make Speculative Execution Domain-Agnostic

### Problem
`llm/src/speculative.rs` lines 813-853 hardcode gold-loan relevance scoring.

### Files to Modify
- `crates/llm/src/speculative.rs`
- Add relevance config to domain.yaml

---

#### 2.5.1 Add Relevance Terms to Domain Config

**File:** `config/domains/gold_loan/domain.yaml`

```yaml
# Add new section
relevance_scoring:
  domain_terms:
    - "gold"
    - "loan"
    - "interest"
    - "rate"
    - "emi"
    - "tenure"
    - "collateral"
    - "purity"
    - "valuation"
    - "disbursement"
    - "सोना"
    - "ऋण"
    - "ब्याज"

  brand_terms:
    - "kotak"
    - "kotak mahindra"

  competitor_terms:
    - "muthoot"
    - "manappuram"
    - "iifl"

  term_weight: 0.05  # Weight per matching term
  max_score: 1.0
```

#### 2.5.2 Refactor Domain Relevance Scoring

**File:** `crates/llm/src/speculative.rs`

**Replace hardcoded estimate_domain_relevance() (lines 813-853):**

```rust
pub struct RelevanceConfig {
    pub domain_terms: Vec<String>,
    pub brand_terms: Vec<String>,
    pub competitor_terms: Vec<String>,
    pub term_weight: f32,
    pub max_score: f32,
}

impl SpeculativeExecutor {
    /// Estimate domain relevance using config-driven terms
    fn estimate_domain_relevance_from_config(
        &self,
        response: &str,
        config: &RelevanceConfig,
    ) -> f32 {
        let lower = response.to_lowercase();
        let mut score = 0.0;

        // Check domain terms
        for term in &config.domain_terms {
            if lower.contains(&term.to_lowercase()) {
                score += config.term_weight;
            }
        }

        // Check brand terms (higher weight)
        for term in &config.brand_terms {
            if lower.contains(&term.to_lowercase()) {
                score += config.term_weight * 1.5;
            }
        }

        // Check competitor terms
        for term in &config.competitor_terms {
            if lower.contains(&term.to_lowercase()) {
                score += config.term_weight;
            }
        }

        score.min(config.max_score)
    }
}
```

### Verification
```bash
cargo check -p voice-agent-llm
cargo test -p voice-agent-llm speculative
```

---

## Phase 2 Completion Checklist

- [ ] 2.1 Tool definitions loaded from YAML, gold_loan_tools() removed
- [ ] 2.2 System prompts fully templated from config
- [ ] 2.3 Slot extraction patterns loaded from config
- [ ] 2.4 DynamicDialogueState replaces GoldLoanDialogueState
- [ ] 2.5 Speculative execution uses config-driven relevance scoring

### Verification Commands
```bash
# Check for remaining hardcoded tool definitions
grep -rn "gold_loan_tools" crates/

# Check for hardcoded prompts
grep -rn "Gold Loan specialist" crates/

# Check for hardcoded slot names
grep -rn '"gold_weight"' crates/
grep -rn '"gold_purity"' crates/

# Full test suite
cargo test --workspace
```

---

## Dependencies for Phase 3

Phase 3 (Code Organization) can proceed independently once:
- Task 2.3 complete (slot extraction refactored)
- Task 2.4 complete (DialogueState generic)
