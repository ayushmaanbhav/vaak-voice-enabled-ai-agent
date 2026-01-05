//! Gold Loan Dialogue State Slot Definitions
//!
//! Domain-specific slot schema based on LDST and ACL 2024 research.
//! Implements structured dialogue state for gold loan conversations.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Gold purity levels (in karats)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GoldPurity {
    /// 24 karat (99.9% pure)
    K24,
    /// 22 karat (91.6% pure)
    K22,
    /// 18 karat (75% pure)
    K18,
    /// 14 karat (58.3% pure)
    K14,
    /// Unknown purity
    Unknown,
}

impl GoldPurity {
    /// Get purity percentage
    pub fn percentage(&self) -> f32 {
        match self {
            GoldPurity::K24 => 99.9,
            GoldPurity::K22 => 91.6,
            GoldPurity::K18 => 75.0,
            GoldPurity::K14 => 58.3,
            GoldPurity::Unknown => 0.0,
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("24") {
            GoldPurity::K24
        } else if lower.contains("22") {
            GoldPurity::K22
        } else if lower.contains("18") {
            GoldPurity::K18
        } else if lower.contains("14") {
            GoldPurity::K14
        } else {
            GoldPurity::Unknown
        }
    }
}

impl std::fmt::Display for GoldPurity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GoldPurity::K24 => write!(f, "24 karat"),
            GoldPurity::K22 => write!(f, "22 karat"),
            GoldPurity::K18 => write!(f, "18 karat"),
            GoldPurity::K14 => write!(f, "14 karat"),
            GoldPurity::Unknown => write!(f, "unknown purity"),
        }
    }
}

/// Urgency level for loan requirement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrgencyLevel {
    /// Immediate need (today/tomorrow)
    Immediate,
    /// Soon (within a week)
    Soon,
    /// Planning ahead (no specific timeline)
    Planning,
    /// Just exploring options
    Exploring,
}

impl UrgencyLevel {
    /// Parse from utterance context
    pub fn from_utterance(text: &str) -> Option<Self> {
        let lower = text.to_lowercase();

        // Immediate indicators
        if lower.contains("urgent") || lower.contains("today") || lower.contains("now")
            || lower.contains("immediately") || lower.contains("abhi") || lower.contains("turant")
            || lower.contains("aaj") || lower.contains("emergency")
        {
            return Some(UrgencyLevel::Immediate);
        }

        // Soon indicators
        if lower.contains("this week") || lower.contains("few days") || lower.contains("jaldi")
            || lower.contains("soon") || lower.contains("is hafte")
        {
            return Some(UrgencyLevel::Soon);
        }

        // Planning indicators
        if lower.contains("next month") || lower.contains("planning") || lower.contains("thinking")
            || lower.contains("soch") || lower.contains("agle mahine")
        {
            return Some(UrgencyLevel::Planning);
        }

        // Exploring indicators
        if lower.contains("just checking") || lower.contains("exploring") || lower.contains("options")
            || lower.contains("jaankari") || lower.contains("information")
        {
            return Some(UrgencyLevel::Exploring);
        }

        None
    }
}

impl std::fmt::Display for UrgencyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UrgencyLevel::Immediate => write!(f, "immediate"),
            UrgencyLevel::Soon => write!(f, "soon"),
            UrgencyLevel::Planning => write!(f, "planning"),
            UrgencyLevel::Exploring => write!(f, "exploring"),
        }
    }
}

/// A slot value with confidence and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotValue {
    /// The value as a string
    pub value: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
    /// Turn index when this was set
    pub turn_set: usize,
    /// Whether user confirmed this value
    pub confirmed: bool,
}

impl SlotValue {
    /// Create a new slot value
    pub fn new(value: impl Into<String>, confidence: f32, turn: usize) -> Self {
        Self {
            value: value.into(),
            confidence,
            turn_set: turn,
            confirmed: false,
        }
    }

    /// Mark as confirmed
    pub fn confirm(&mut self) {
        self.confirmed = true;
        self.confidence = 1.0;
    }
}

/// Gold Loan Dialogue State
///
/// Tracks all slot values relevant to a gold loan conversation.
/// Implements domain-specific slot schema based on gold loan business logic.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GoldLoanDialogueState {
    // ====== Customer Information ======
    /// Customer name
    customer_name: Option<SlotValue>,
    /// Phone number
    phone_number: Option<SlotValue>,
    /// Location/city
    location: Option<SlotValue>,
    /// Pincode
    pincode: Option<SlotValue>,

    // ====== Gold Details ======
    /// Gold weight in grams
    gold_weight_grams: Option<SlotValue>,
    /// Gold purity (karat)
    gold_purity: Option<SlotValue>,
    /// Type of gold item (jewelry, coins, bars)
    gold_item_type: Option<SlotValue>,

    // ====== Loan Requirements ======
    /// Desired loan amount
    loan_amount: Option<SlotValue>,
    /// Loan purpose
    loan_purpose: Option<SlotValue>,
    /// Preferred tenure (months)
    loan_tenure: Option<SlotValue>,
    /// Urgency level
    urgency: Option<SlotValue>,

    // ====== Existing Loan (for balance transfer) ======
    /// Current lender
    current_lender: Option<SlotValue>,
    /// Current outstanding amount
    current_outstanding: Option<SlotValue>,
    /// Current interest rate
    current_interest_rate: Option<SlotValue>,

    // ====== Scheduling ======
    /// Preferred visit date
    preferred_date: Option<SlotValue>,
    /// Preferred time slot
    preferred_time: Option<SlotValue>,
    /// Branch preference
    preferred_branch: Option<SlotValue>,

    // ====== Intent Tracking ======
    /// Primary detected intent
    primary_intent: Option<String>,
    /// Intent confidence
    intent_confidence: f32,
    /// Secondary intents detected
    secondary_intents: Vec<String>,

    // ====== State Management ======
    /// Slots pending confirmation
    pending_slots: HashSet<String>,
    /// Confirmed slots
    confirmed_slots: HashSet<String>,
    /// Custom/dynamic slots
    custom_slots: HashMap<String, SlotValue>,
}

impl GoldLoanDialogueState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self::default()
    }

    // ====== Customer Information Accessors ======

    /// Get customer name
    pub fn customer_name(&self) -> Option<&str> {
        self.customer_name.as_ref().map(|v| v.value.as_str())
    }

    /// Get phone number
    pub fn phone_number(&self) -> Option<&str> {
        self.phone_number.as_ref().map(|v| v.value.as_str())
    }

    /// Get location
    pub fn location(&self) -> Option<&str> {
        self.location.as_ref().map(|v| v.value.as_str())
    }

    /// Get pincode
    pub fn pincode(&self) -> Option<&str> {
        self.pincode.as_ref().map(|v| v.value.as_str())
    }

    // ====== Gold Details Accessors ======

    /// Get gold weight in grams
    pub fn gold_weight_grams(&self) -> Option<f64> {
        self.gold_weight_grams
            .as_ref()
            .and_then(|v| v.value.parse().ok())
    }

    /// Get gold purity
    pub fn gold_purity(&self) -> Option<GoldPurity> {
        self.gold_purity
            .as_ref()
            .map(|v| GoldPurity::from_str(&v.value))
    }

    /// Get gold item type
    pub fn gold_item_type(&self) -> Option<&str> {
        self.gold_item_type.as_ref().map(|v| v.value.as_str())
    }

    // ====== Loan Requirements Accessors ======

    /// Get loan amount
    pub fn loan_amount(&self) -> Option<f64> {
        self.loan_amount
            .as_ref()
            .and_then(|v| v.value.parse().ok())
    }

    /// Get loan purpose
    pub fn loan_purpose(&self) -> Option<&str> {
        self.loan_purpose.as_ref().map(|v| v.value.as_str())
    }

    /// Get loan tenure in months
    pub fn loan_tenure(&self) -> Option<u32> {
        self.loan_tenure
            .as_ref()
            .and_then(|v| v.value.parse().ok())
    }

    /// Get urgency level
    pub fn urgency(&self) -> Option<UrgencyLevel> {
        self.urgency.as_ref().and_then(|v| {
            match v.value.to_lowercase().as_str() {
                "immediate" => Some(UrgencyLevel::Immediate),
                "soon" => Some(UrgencyLevel::Soon),
                "planning" => Some(UrgencyLevel::Planning),
                "exploring" => Some(UrgencyLevel::Exploring),
                _ => None,
            }
        })
    }

    // ====== Existing Loan Accessors ======

    /// Get current lender
    pub fn current_lender(&self) -> Option<&str> {
        self.current_lender.as_ref().map(|v| v.value.as_str())
    }

    /// Get current outstanding amount
    pub fn current_outstanding(&self) -> Option<f64> {
        self.current_outstanding
            .as_ref()
            .and_then(|v| v.value.parse().ok())
    }

    /// Get current interest rate
    pub fn current_interest_rate(&self) -> Option<f32> {
        self.current_interest_rate
            .as_ref()
            .and_then(|v| v.value.parse().ok())
    }

    // ====== Scheduling Accessors ======

    /// Get preferred date
    pub fn preferred_date(&self) -> Option<&str> {
        self.preferred_date.as_ref().map(|v| v.value.as_str())
    }

    /// Get preferred time
    pub fn preferred_time(&self) -> Option<&str> {
        self.preferred_time.as_ref().map(|v| v.value.as_str())
    }

    /// Get preferred branch
    pub fn preferred_branch(&self) -> Option<&str> {
        self.preferred_branch.as_ref().map(|v| v.value.as_str())
    }

    // ====== Intent Accessors ======

    /// Get primary intent
    pub fn primary_intent(&self) -> Option<&str> {
        self.primary_intent.as_deref()
    }

    /// Get intent confidence
    pub fn intent_confidence(&self) -> f32 {
        self.intent_confidence
    }

    /// Get secondary intents
    pub fn secondary_intents(&self) -> &[String] {
        &self.secondary_intents
    }

    /// Update primary intent
    pub fn update_intent(&mut self, intent: &str, confidence: f32) {
        // If we already have this intent, just update confidence
        if self.primary_intent.as_deref() == Some(intent) {
            self.intent_confidence = confidence;
            return;
        }

        // Move current primary to secondary if exists
        if let Some(ref prev) = self.primary_intent {
            if !self.secondary_intents.contains(prev) {
                self.secondary_intents.push(prev.clone());
            }
        }

        self.primary_intent = Some(intent.to_string());
        self.intent_confidence = confidence;
    }

    // ====== State Management ======

    /// Get slots pending confirmation
    pub fn pending_slots(&self) -> &HashSet<String> {
        &self.pending_slots
    }

    /// Get confirmed slots
    pub fn confirmed_slots(&self) -> &HashSet<String> {
        &self.confirmed_slots
    }

    /// Mark a slot as pending confirmation
    pub fn mark_pending(&mut self, slot_name: &str) {
        self.confirmed_slots.remove(slot_name);
        self.pending_slots.insert(slot_name.to_string());
    }

    /// Mark a slot as confirmed
    pub fn mark_confirmed(&mut self, slot_name: &str) {
        self.pending_slots.remove(slot_name);
        self.confirmed_slots.insert(slot_name.to_string());

        // Also update the slot value's confirmed flag
        match slot_name {
            "customer_name" => { if let Some(ref mut v) = self.customer_name { v.confirm(); } }
            "phone_number" => { if let Some(ref mut v) = self.phone_number { v.confirm(); } }
            "location" => { if let Some(ref mut v) = self.location { v.confirm(); } }
            "pincode" => { if let Some(ref mut v) = self.pincode { v.confirm(); } }
            "gold_weight" => { if let Some(ref mut v) = self.gold_weight_grams { v.confirm(); } }
            "gold_purity" => { if let Some(ref mut v) = self.gold_purity { v.confirm(); } }
            "gold_item_type" => { if let Some(ref mut v) = self.gold_item_type { v.confirm(); } }
            "loan_amount" => { if let Some(ref mut v) = self.loan_amount { v.confirm(); } }
            "loan_purpose" => { if let Some(ref mut v) = self.loan_purpose { v.confirm(); } }
            "loan_tenure" => { if let Some(ref mut v) = self.loan_tenure { v.confirm(); } }
            "urgency" => { if let Some(ref mut v) = self.urgency { v.confirm(); } }
            "current_lender" => { if let Some(ref mut v) = self.current_lender { v.confirm(); } }
            "current_outstanding" => { if let Some(ref mut v) = self.current_outstanding { v.confirm(); } }
            "current_interest_rate" => { if let Some(ref mut v) = self.current_interest_rate { v.confirm(); } }
            "preferred_date" => { if let Some(ref mut v) = self.preferred_date { v.confirm(); } }
            "preferred_time" => { if let Some(ref mut v) = self.preferred_time { v.confirm(); } }
            "preferred_branch" => { if let Some(ref mut v) = self.preferred_branch { v.confirm(); } }
            _ => {
                if let Some(v) = self.custom_slots.get_mut(slot_name) {
                    v.confirm();
                }
            }
        }
    }

    // ====== Generic Slot Access ======

    /// Get slot value by name
    pub fn get_slot_value(&self, slot_name: &str) -> Option<String> {
        match slot_name {
            "customer_name" => self.customer_name.as_ref().map(|v| v.value.clone()),
            "phone_number" => self.phone_number.as_ref().map(|v| v.value.clone()),
            "location" => self.location.as_ref().map(|v| v.value.clone()),
            "pincode" => self.pincode.as_ref().map(|v| v.value.clone()),
            "gold_weight" => self.gold_weight_grams.as_ref().map(|v| v.value.clone()),
            "gold_purity" => self.gold_purity.as_ref().map(|v| v.value.clone()),
            "gold_item_type" => self.gold_item_type.as_ref().map(|v| v.value.clone()),
            "loan_amount" => self.loan_amount.as_ref().map(|v| v.value.clone()),
            "loan_purpose" => self.loan_purpose.as_ref().map(|v| v.value.clone()),
            "loan_tenure" => self.loan_tenure.as_ref().map(|v| v.value.clone()),
            "urgency" => self.urgency.as_ref().map(|v| v.value.clone()),
            "current_lender" => self.current_lender.as_ref().map(|v| v.value.clone()),
            "current_outstanding" => self.current_outstanding.as_ref().map(|v| v.value.clone()),
            "current_interest_rate" => self.current_interest_rate.as_ref().map(|v| v.value.clone()),
            "preferred_date" => self.preferred_date.as_ref().map(|v| v.value.clone()),
            "preferred_time" => self.preferred_time.as_ref().map(|v| v.value.clone()),
            "preferred_branch" => self.preferred_branch.as_ref().map(|v| v.value.clone()),
            _ => self.custom_slots.get(slot_name).map(|v| v.value.clone()),
        }
    }

    /// Get slot with confidence
    pub fn get_slot_with_confidence(&self, slot_name: &str) -> Option<&SlotValue> {
        match slot_name {
            "customer_name" => self.customer_name.as_ref(),
            "phone_number" => self.phone_number.as_ref(),
            "location" => self.location.as_ref(),
            "pincode" => self.pincode.as_ref(),
            "gold_weight" => self.gold_weight_grams.as_ref(),
            "gold_purity" => self.gold_purity.as_ref(),
            "gold_item_type" => self.gold_item_type.as_ref(),
            "loan_amount" => self.loan_amount.as_ref(),
            "loan_purpose" => self.loan_purpose.as_ref(),
            "loan_tenure" => self.loan_tenure.as_ref(),
            "urgency" => self.urgency.as_ref(),
            "current_lender" => self.current_lender.as_ref(),
            "current_outstanding" => self.current_outstanding.as_ref(),
            "current_interest_rate" => self.current_interest_rate.as_ref(),
            "preferred_date" => self.preferred_date.as_ref(),
            "preferred_time" => self.preferred_time.as_ref(),
            "preferred_branch" => self.preferred_branch.as_ref(),
            _ => self.custom_slots.get(slot_name),
        }
    }

    /// Set slot value by name
    pub fn set_slot_value(&mut self, slot_name: &str, value: &str, confidence: f32) {
        let slot_value = SlotValue::new(value, confidence, 0);

        match slot_name {
            "customer_name" => self.customer_name = Some(slot_value),
            "phone_number" => self.phone_number = Some(slot_value),
            "location" => self.location = Some(slot_value),
            "pincode" => self.pincode = Some(slot_value),
            "gold_weight" => self.gold_weight_grams = Some(slot_value),
            "gold_purity" => self.gold_purity = Some(slot_value),
            "gold_item_type" => self.gold_item_type = Some(slot_value),
            "loan_amount" => self.loan_amount = Some(slot_value),
            "loan_purpose" => self.loan_purpose = Some(slot_value),
            "loan_tenure" => self.loan_tenure = Some(slot_value),
            "urgency" => self.urgency = Some(slot_value),
            "current_lender" => self.current_lender = Some(slot_value),
            "current_outstanding" => self.current_outstanding = Some(slot_value),
            "current_interest_rate" => self.current_interest_rate = Some(slot_value),
            "preferred_date" => self.preferred_date = Some(slot_value),
            "preferred_time" => self.preferred_time = Some(slot_value),
            "preferred_branch" => self.preferred_branch = Some(slot_value),
            _ => {
                self.custom_slots.insert(slot_name.to_string(), slot_value);
            }
        }
    }

    /// Clear a slot
    pub fn clear_slot(&mut self, slot_name: &str) {
        self.pending_slots.remove(slot_name);
        self.confirmed_slots.remove(slot_name);

        match slot_name {
            "customer_name" => self.customer_name = None,
            "phone_number" => self.phone_number = None,
            "location" => self.location = None,
            "pincode" => self.pincode = None,
            "gold_weight" => self.gold_weight_grams = None,
            "gold_purity" => self.gold_purity = None,
            "gold_item_type" => self.gold_item_type = None,
            "loan_amount" => self.loan_amount = None,
            "loan_purpose" => self.loan_purpose = None,
            "loan_tenure" => self.loan_tenure = None,
            "urgency" => self.urgency = None,
            "current_lender" => self.current_lender = None,
            "current_outstanding" => self.current_outstanding = None,
            "current_interest_rate" => self.current_interest_rate = None,
            "preferred_date" => self.preferred_date = None,
            "preferred_time" => self.preferred_time = None,
            "preferred_branch" => self.preferred_branch = None,
            _ => {
                self.custom_slots.remove(slot_name);
            }
        }
    }

    /// Convert state to context string for LLM prompts
    pub fn to_context_string(&self) -> String {
        let mut parts = Vec::new();

        // Customer info
        if let Some(name) = self.customer_name() {
            parts.push(format!("Customer: {}", name));
        }
        if let Some(phone) = self.phone_number() {
            parts.push(format!("Phone: {}", phone));
        }
        if let Some(loc) = self.location() {
            parts.push(format!("Location: {}", loc));
        }

        // Gold details
        if let Some(weight) = self.gold_weight_grams() {
            parts.push(format!("Gold weight: {}g", weight));
        }
        if let Some(purity) = self.gold_purity() {
            parts.push(format!("Purity: {}", purity));
        }

        // Loan requirements
        if let Some(amount) = self.loan_amount() {
            let formatted = if amount >= 100_000.0 {
                format!("₹{:.1} lakh", amount / 100_000.0)
            } else {
                format!("₹{:.0}", amount)
            };
            parts.push(format!("Loan amount: {}", formatted));
        }
        if let Some(purpose) = self.loan_purpose() {
            parts.push(format!("Purpose: {}", purpose));
        }

        // Existing loan
        if let Some(lender) = self.current_lender() {
            parts.push(format!("Current lender: {}", lender));
        }
        if let Some(outstanding) = self.current_outstanding() {
            parts.push(format!("Outstanding: ₹{:.0}", outstanding));
        }

        // Intent
        if let Some(intent) = self.primary_intent() {
            parts.push(format!("Intent: {}", intent));
        }

        if parts.is_empty() {
            "No information collected yet.".to_string()
        } else {
            parts.join("\n")
        }
    }

    /// Get all filled slot names
    pub fn filled_slots(&self) -> Vec<&str> {
        let mut slots = Vec::new();

        if self.customer_name.is_some() { slots.push("customer_name"); }
        if self.phone_number.is_some() { slots.push("phone_number"); }
        if self.location.is_some() { slots.push("location"); }
        if self.pincode.is_some() { slots.push("pincode"); }
        if self.gold_weight_grams.is_some() { slots.push("gold_weight"); }
        if self.gold_purity.is_some() { slots.push("gold_purity"); }
        if self.gold_item_type.is_some() { slots.push("gold_item_type"); }
        if self.loan_amount.is_some() { slots.push("loan_amount"); }
        if self.loan_purpose.is_some() { slots.push("loan_purpose"); }
        if self.loan_tenure.is_some() { slots.push("loan_tenure"); }
        if self.urgency.is_some() { slots.push("urgency"); }
        if self.current_lender.is_some() { slots.push("current_lender"); }
        if self.current_outstanding.is_some() { slots.push("current_outstanding"); }
        if self.current_interest_rate.is_some() { slots.push("current_interest_rate"); }
        if self.preferred_date.is_some() { slots.push("preferred_date"); }
        if self.preferred_time.is_some() { slots.push("preferred_time"); }
        if self.preferred_branch.is_some() { slots.push("preferred_branch"); }

        for key in self.custom_slots.keys() {
            slots.push(key.as_str());
        }

        slots
    }

    /// Calculate completion percentage for a given intent
    pub fn completion_for_intent(&self, intent: &str) -> f32 {
        let (filled, required) = match intent {
            "eligibility_check" => {
                let required = ["gold_weight"];
                let filled = required.iter().filter(|s| self.get_slot_value(s).is_some()).count();
                (filled, required.len())
            }
            "loan_inquiry" => {
                let required = ["loan_amount"];
                let optional = ["gold_weight", "gold_purity", "loan_tenure"];
                let filled_req = required.iter().filter(|s| self.get_slot_value(s).is_some()).count();
                let filled_opt = optional.iter().filter(|s| self.get_slot_value(s).is_some()).count();
                // Weight: required slots 70%, optional 30%
                let score = (filled_req as f32 / required.len() as f32) * 0.7
                    + (filled_opt as f32 / optional.len() as f32) * 0.3;
                return score;
            }
            "switch_lender" | "balance_transfer" => {
                let required = ["current_lender"];
                let filled = required.iter().filter(|s| self.get_slot_value(s).is_some()).count();
                (filled, required.len())
            }
            "schedule_visit" => {
                let required = ["location"];
                let optional = ["preferred_date", "preferred_time"];
                let filled_req = required.iter().filter(|s| self.get_slot_value(s).is_some()).count();
                let filled_opt = optional.iter().filter(|s| self.get_slot_value(s).is_some()).count();
                let score = (filled_req as f32 / required.len() as f32) * 0.6
                    + (filled_opt as f32 / optional.len() as f32) * 0.4;
                return score;
            }
            "send_sms" | "contact_callback" => {
                let required = ["phone_number"];
                let filled = required.iter().filter(|s| self.get_slot_value(s).is_some()).count();
                (filled, required.len())
            }
            _ => return 1.0, // Unknown intents are always "complete"
        };

        if required == 0 {
            1.0
        } else {
            filled as f32 / required as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gold_purity_parsing() {
        assert_eq!(GoldPurity::from_str("24k gold"), GoldPurity::K24);
        assert_eq!(GoldPurity::from_str("22 karat"), GoldPurity::K22);
        assert_eq!(GoldPurity::from_str("18kt"), GoldPurity::K18);
        assert_eq!(GoldPurity::from_str("pure gold"), GoldPurity::Unknown);
    }

    #[test]
    fn test_gold_purity_percentage() {
        assert!((GoldPurity::K24.percentage() - 99.9).abs() < 0.1);
        assert!((GoldPurity::K22.percentage() - 91.6).abs() < 0.1);
    }

    #[test]
    fn test_urgency_detection() {
        assert_eq!(UrgencyLevel::from_utterance("I need it today"), Some(UrgencyLevel::Immediate));
        assert_eq!(UrgencyLevel::from_utterance("mujhe abhi chahiye"), Some(UrgencyLevel::Immediate));
        assert_eq!(UrgencyLevel::from_utterance("this week sometime"), Some(UrgencyLevel::Soon));
        assert_eq!(UrgencyLevel::from_utterance("just exploring options"), Some(UrgencyLevel::Exploring));
    }

    #[test]
    fn test_state_creation() {
        let state = GoldLoanDialogueState::new();
        assert!(state.customer_name().is_none());
        assert!(state.loan_amount().is_none());
        assert!(state.filled_slots().is_empty());
    }

    #[test]
    fn test_slot_set_and_get() {
        let mut state = GoldLoanDialogueState::new();

        state.set_slot_value("customer_name", "Rahul", 0.9);
        state.set_slot_value("loan_amount", "500000", 0.85);

        assert_eq!(state.customer_name(), Some("Rahul"));
        assert_eq!(state.loan_amount(), Some(500000.0));
    }

    #[test]
    fn test_slot_confirmation() {
        let mut state = GoldLoanDialogueState::new();

        state.set_slot_value("gold_weight", "50", 0.8);
        state.mark_pending("gold_weight");

        assert!(state.pending_slots().contains("gold_weight"));
        assert!(!state.confirmed_slots().contains("gold_weight"));

        state.mark_confirmed("gold_weight");

        assert!(!state.pending_slots().contains("gold_weight"));
        assert!(state.confirmed_slots().contains("gold_weight"));
    }

    #[test]
    fn test_custom_slots() {
        let mut state = GoldLoanDialogueState::new();

        state.set_slot_value("custom_field", "custom_value", 0.9);

        assert_eq!(state.get_slot_value("custom_field"), Some("custom_value".to_string()));
    }

    #[test]
    fn test_context_string() {
        let mut state = GoldLoanDialogueState::new();

        state.set_slot_value("customer_name", "Rahul", 0.9);
        state.set_slot_value("loan_amount", "500000", 0.9);
        state.set_slot_value("gold_weight", "50", 0.9);

        let context = state.to_context_string();
        assert!(context.contains("Rahul"));
        assert!(context.contains("5.0 lakh"));
        assert!(context.contains("50g"));
    }

    #[test]
    fn test_intent_completion() {
        let mut state = GoldLoanDialogueState::new();

        // Eligibility check requires gold_weight
        assert_eq!(state.completion_for_intent("eligibility_check"), 0.0);

        state.set_slot_value("gold_weight", "50", 0.9);
        assert_eq!(state.completion_for_intent("eligibility_check"), 1.0);
    }

    #[test]
    fn test_clear_slot() {
        let mut state = GoldLoanDialogueState::new();

        state.set_slot_value("customer_name", "Rahul", 0.9);
        state.mark_confirmed("customer_name");

        assert!(state.customer_name().is_some());
        assert!(state.confirmed_slots().contains("customer_name"));

        state.clear_slot("customer_name");

        assert!(state.customer_name().is_none());
        assert!(!state.confirmed_slots().contains("customer_name"));
    }

    #[test]
    fn test_intent_tracking() {
        let mut state = GoldLoanDialogueState::new();

        state.update_intent("loan_inquiry", 0.9);
        assert_eq!(state.primary_intent(), Some("loan_inquiry"));

        state.update_intent("eligibility_check", 0.85);
        assert_eq!(state.primary_intent(), Some("eligibility_check"));
        assert!(state.secondary_intents().contains(&"loan_inquiry".to_string()));
    }
}
