//! Slot Value Extraction for Conversations
//!
//! Moved from agent/dst/extractor.rs as part of Phase 3.3 crate boundary fix.
//!
//! Implements rule-based and pattern-based slot extraction from user utterances.
//! Supports Hindi, Hinglish, and English utterances.
//!
//! ## Config-Driven Slot Extraction (P16 FIX)
//!
//! Slot extraction patterns can be loaded from domain config (slots.yaml).
//! Use `SlotExtractor::from_config()` for domain-agnostic operation.
//!
//! ## Optimization: Static Regex Patterns
//!
//! Static patterns are compiled once at program start using `once_cell::sync::Lazy`.
//! These serve as fallbacks when config-driven patterns are not available.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

use crate::intent::{Slot, SlotType};

/// P16 FIX: Slot extraction configuration from domain config
/// This mirrors the structure in slots.yaml
#[derive(Debug, Clone, Default)]
pub struct SlotExtractionConfig {
    /// Custom extraction patterns by slot name -> language -> patterns
    pub custom_patterns: HashMap<String, HashMap<String, Vec<String>>>,
    /// Lender patterns for competitor detection
    pub lenders: HashMap<String, Vec<String>>,
    /// Intent patterns for intent detection
    pub intent_patterns: Vec<(String, String)>, // (pattern, intent_name)
}

// =============================================================================
// STATIC REGEX PATTERNS - Compiled once at program start
// =============================================================================

/// Amount multiplier for parsing
#[derive(Debug, Clone, Copy)]
enum AmountMultiplier {
    Unit,       // 1
    Thousand,   // 1,000
    Lakh,       // 100,000
    Crore,      // 10,000,000
}

impl AmountMultiplier {
    fn value(&self) -> f64 {
        match self {
            AmountMultiplier::Unit => 1.0,
            AmountMultiplier::Thousand => 1_000.0,
            AmountMultiplier::Lakh => 100_000.0,
            AmountMultiplier::Crore => 10_000_000.0,
        }
    }
}

// Amount patterns (Crore, Lakh, Thousand, Rupee, Plain numbers)
static AMOUNT_PATTERNS: Lazy<Vec<(Regex, AmountMultiplier)>> = Lazy::new(|| vec![
    (Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:crore|cr|करोड़)").unwrap(), AmountMultiplier::Crore),
    (Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:lakh|lac|लाख)").unwrap(), AmountMultiplier::Lakh),
    (Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:thousand|k|हज़ार|hazar)").unwrap(), AmountMultiplier::Thousand),
    (Regex::new(r"(?:₹|rs\.?|rupees?)\s*(\d+(?:,\d+)*)").unwrap(), AmountMultiplier::Unit),
    (Regex::new(r"\b(\d{5,8})\b").unwrap(), AmountMultiplier::Unit),
]);

// Weight patterns (grams, tola, contextual)
static WEIGHT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:grams?|gm|g|ग्राम)").unwrap(),
    Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:tola|तोला)").unwrap(),
    Regex::new(r"(?i)(?:have|hai|है)\s*(\d+(?:\.\d+)?)\s*(?:grams?|g)?\s*(?:gold|sona|सोना)").unwrap(),
]);

// Phone patterns (Indian mobile numbers)
static PHONE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"\b([6-9]\d{9})\b").unwrap(),
    Regex::new(r"(?:\+91|91)?[-\s]?([6-9]\d{9})\b").unwrap(),
    Regex::new(r"\b([6-9]\d{2})[-\s]?(\d{3})[-\s]?(\d{4})\b").unwrap(),
]);

// Pincode patterns (Indian 6-digit pincodes)
static PINCODE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"\b([1-9]\d{5})\b").unwrap(),
    Regex::new(r"(?i)(?:pincode|pin|पिनकोड)\s*(?:is|hai|है)?\s*(\d{6})").unwrap(),
]);

// Time patterns
static TIME_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)(\d{1,2})(?::(\d{2}))?\s*(am|pm|बजे)").unwrap(),
    Regex::new(r"(?i)(morning|afternoon|evening|subah|dopahar|shaam)").unwrap(),
]);

// Name patterns (English and Hindi)
static NAME_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)(?:my\s+name\s+is|i\s+am|i'm|this\s+is|call\s+me)\s+([A-Z][a-zA-Z]+(?:\s+[A-Z][a-zA-Z]+)*)").unwrap(),
    Regex::new(r"(?i)(?:mera\s+)?(?:naam|name)\s+([A-Z][a-zA-Z]+(?:\s+[A-Z][a-zA-Z]*)?)\s+(?:hai|h)\b").unwrap(),
    Regex::new(r"(?i)(?:mera\s+)?(?:naam|name)\s+([A-Z][a-zA-Z]+)(?:\s+[A-Z][a-zA-Z]+)?(?:\s|$|[.,])").unwrap(),
    Regex::new(r"(?i)(?:myself|name[:\s]+)\s*([A-Z][a-zA-Z]+(?:\s+[A-Z][a-zA-Z]+)*)").unwrap(),
]);

// PAN patterns
static PAN_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)(?:pan|pan\s+(?:card|number|no\.?)|my\s+pan)\s*(?:is|:)?\s*([A-Z]{5}[0-9]{4}[A-Z])").unwrap(),
    Regex::new(r"\b([A-Z]{5}[0-9]{4}[A-Z])\b").unwrap(),
    Regex::new(r"(?i)pan\s+(?:is|:)?\s*(\d{8,10})").unwrap(),
]);

// DOB patterns
static DOB_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)(?:date\s+of\s+birth|dob|born\s+on|birthday)\s*(?:is|:)?\s*(\d{1,2}[/\-\.]\d{1,2}[/\-\.]\d{2,4})").unwrap(),
    Regex::new(r"(?i)(?:date\s+of\s+birth|dob|born\s+on|birthday)\s*(?:is|:)?\s*(\d{1,2}(?:st|nd|rd|th)?\s+(?:jan(?:uary)?|feb(?:ruary)?|mar(?:ch)?|apr(?:il)?|may|jun(?:e)?|jul(?:y)?|aug(?:ust)?|sep(?:tember)?|oct(?:ober)?|nov(?:ember)?|dec(?:ember)?)\s+\d{2,4})").unwrap(),
    Regex::new(r"(?i)(?:janam\s+din|janam\s+tithi)\s*(?:hai|:)?\s*(\d{1,2}\s+\w+\s+\d{2,4})").unwrap(),
]);

// Loan purpose patterns
static PURPOSE_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| vec![
    (Regex::new(r"(?i)(?:business|dhandha|vyapaar|karobar|shop|dukaan)").unwrap(), "business"),
    (Regex::new(r"(?i)(?:working\s+capital|stock|inventory|माल)").unwrap(), "business_working_capital"),
    (Regex::new(r"(?i)(?:medical|hospital|doctor|treatment|ilaj|ilaaj|dawai|medicine|surgery|operation)").unwrap(), "medical"),
    (Regex::new(r"(?i)(?:education|school|college|fees|padhai|study|exam|admission)").unwrap(), "education"),
    (Regex::new(r"(?i)(?:wedding|marriage|shaadi|shadi|vivah|byah)").unwrap(), "wedding"),
    (Regex::new(r"(?i)(?:renovation|repair|construction|ghar|home\s+improvement|makaan)").unwrap(), "home_renovation"),
    (Regex::new(r"(?i)(?:farming|agriculture|khet|kheti|crop|fasal|tractor|seeds|beej)").unwrap(), "agriculture"),
    (Regex::new(r"(?i)(?:debt|loan\s+repay|karza|karz|EMI\s+pay)").unwrap(), "debt_consolidation"),
    (Regex::new(r"(?i)(?:emergency|urgent|zaruri|jaldi|turant|immediately)").unwrap(), "emergency"),
]);

// Repayment type patterns
static REPAYMENT_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| vec![
    (Regex::new(r"(?i)(?:EMI|monthly\s+(?:payment|installment)|mahina|kishte)").unwrap(), "emi"),
    (Regex::new(r"(?i)(?:bullet|lump\s*sum|one\s+time|ek\s+baar|ekmusht)").unwrap(), "bullet"),
    (Regex::new(r"(?i)(?:overdraft|OD|credit\s+line|flexible)").unwrap(), "overdraft"),
    (Regex::new(r"(?i)(?:interest\s+only|sirf\s+byaaj|only\s+interest)").unwrap(), "interest_only"),
]);

// City patterns (major Indian cities)
static CITY_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)(?:from|in|at|near|city|sheher)\s+([A-Z][a-zA-Z]+(?:\s+[A-Z][a-zA-Z]+)?)").unwrap(),
    Regex::new(r"(?i)\b(Mumbai|Delhi|Bangalore|Bengaluru|Chennai|Hyderabad|Kolkata|Pune|Ahmedabad|Jaipur|Lucknow|Kanpur|Nagpur|Indore|Thane|Bhopal|Visakhapatnam|Patna|Vadodara|Ghaziabad|Ludhiana|Agra|Nashik|Faridabad|Meerut|Rajkot|Kalyan|Vasai|Varanasi|Srinagar|Aurangabad|Dhanbad|Amritsar|Navi Mumbai|Allahabad|Ranchi|Howrah|Coimbatore|Jabalpur|Gwalior|Vijayawada|Jodhpur|Madurai|Raipur|Kota|Guwahati|Chandigarh|Solapur|Hubli|Mysore|Tiruchirappalli|Bareilly|Aligarh|Tiruppur|Gurgaon|Noida|NCR)\b").unwrap(),
    Regex::new(r"(?i)\b(Dilli|Mumbay|Calcutta|Madras|Bangaluru)\b").unwrap(),
]);

// Intent detection patterns (order matters - more specific first)
static INTENT_PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| vec![
    (Regex::new(r"(?i)(?:balance\s+transfer|loan\s+transfer|transfer\s+(?:my\s+)?loan|move\s+(?:my\s+)?loan|transfer\s+kar|BT\s+kar|switch\s+(?:to|from)\s+\w+)").unwrap(), "balance_transfer"),
    (Regex::new(r"(?i)(?:gold\s+(?:price|rate)|sone\s+ka\s+(?:rate|bhav|price)|aaj\s+ka\s+(?:gold\s+)?rate|today.+gold|current\s+gold)").unwrap(), "gold_price_inquiry"),
    (Regex::new(r"(?i)(?:interest\s+rate|byaaj\s+dar|rate\s+kya|kitna\s+percent|what.+(?:interest|byaaj)\s+rate)").unwrap(), "rate_inquiry"),
    (Regex::new(r"(?i)(?:kitna\s+bachega|how\s+much\s+(?:can\s+i\s+)?sav|bachat|savings|save\s+money|calculate\s+saving)").unwrap(), "savings_inquiry"),
    (Regex::new(r"(?i)(?:am\s+i\s+eligible|eligibility|loan\s+milega|kitna\s+loan|qualify|kya\s+mil\s+sakta|eligible\s+for)").unwrap(), "eligibility_inquiry"),
    (Regex::new(r"(?i)(?:documents?\s+(?:required|needed|chahiye|list)|kya\s+laana|what\s+(?:documents?|to\s+bring)|kaunsa\s+document|laana\s+(?:hoga|padega))").unwrap(), "document_inquiry"),
    (Regex::new(r"(?i)(?:book\s+(?:an?\s+)?appointment|schedule\s+(?:a\s+)?(?:visit|appointment)|fix\s+(?:a\s+)?time|milna\s+(?:hai|chahta)|time\s+slot|slot\s+book)").unwrap(), "appointment_request"),
    (Regex::new(r"(?i)(?:(?:nearest|nearby)\s+branch|branch\s+(?:location|kahan|where)|where\s+is\s+(?:the\s+)?(?:branch|office)|office\s+address|location\s+of)").unwrap(), "branch_inquiry"),
    (Regex::new(r"(?i)(?:(?:is\s+)?(?:my\s+)?gold\s+safe|security|suraksha|chori|theft|insurance|vault|locker)").unwrap(), "safety_inquiry"),
    (Regex::new(r"(?i)(?:repay|payment\s+(?:option|method)|EMI\s+(?:kaise|how)|bhugtan|kaise\s+dena|how\s+to\s+pay|repayment)").unwrap(), "repayment_inquiry"),
    (Regex::new(r"(?i)(?:close\s+(?:my\s+)?loan|loan\s+close|release\s+(?:my\s+)?gold|gold\s+back|sona\s+wapas|get\s+(?:my\s+)?gold\s+back)").unwrap(), "closure_inquiry"),
    (Regex::new(r"(?i)(?:talk\s+to\s+(?:a\s+)?human|(?:real\s+)?agent|real\s+person|customer\s+care|complaint|shikayat|(?:speak\s+(?:to|with)\s+)?manager)").unwrap(), "human_escalation"),
    (Regex::new(r"(?i)(?:call\s+(?:me\s+)?back|callback|phone\s+kar|give\s+(?:me\s+)?(?:a\s+)?call|ring\s+me)").unwrap(), "callback_request"),
    (Regex::new(r"(?i)(?:send\s+(?:me\s+)?(?:sms|message|details|info)|SMS\s+kar|whatsapp\s+(?:me|kar))").unwrap(), "sms_request"),
    (Regex::new(r"(?i)(?:compare\s+(?:with|to)|comparison|vs\s+\w+|versus|better\s+than\s+(?:muthoot|manappuram|iifl))").unwrap(), "comparison_inquiry"),
]);

// Additional inline patterns (purity, tenure, rate, location)
static PURITY_24K: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)24\s*(?:k|karat|carat|kt)").unwrap());
static PURITY_22K: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)22\s*(?:k|karat|carat|kt)").unwrap());
static PURITY_18K: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)18\s*(?:k|karat|carat|kt)").unwrap());
static PURITY_14K: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)14\s*(?:k|karat|carat|kt)").unwrap());
static PURITY_PURE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)pure\s*gold").unwrap());
static PURITY_HALLMARK: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)hallmark(?:ed)?").unwrap());

static TENURE_MONTHS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)\s*(?:months?|mahine|महीने)").unwrap());
static TENURE_YEARS: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+)\s*(?:years?|saal|साल)").unwrap());

static RATE_CONTEXT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(?:interest\s+)?rate\s+(?:is|:)?\s*(\d+(?:\.\d+)?)\s*(?:%|percent|प्रतिशत)?").unwrap());
static RATE_PERCENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+(?:\.\d+)?)\s*(?:%|percent|प्रतिशत)").unwrap());

static LOCATION_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)(?:from|in|at|near|mein|में)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?)").unwrap());

// Lender patterns (built as HashMap for exact matching)
// NOTE: This is intentionally empty - lender patterns should be loaded from config
// (config/domains/{domain}/slots.yaml) for domain-agnostic operation.
// Use SlotExtractor::with_lenders() or SlotExtractor::from_config() to provide
// domain-specific lender patterns.
static LENDER_PATTERNS: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    // Empty map - all lender patterns come from domain config
    HashMap::new()
});

// =============================================================================
// SLOT EXTRACTOR
// =============================================================================

/// Slot extractor for conversations
///
/// Uses static regex patterns compiled at program start for efficiency.
/// All patterns are stored as module-level statics using `once_cell::sync::Lazy`.
///
/// P16 FIX: Can be configured with domain-specific patterns via `from_config()`.
#[derive(Debug, Clone)]
pub struct SlotExtractor {
    /// Config-driven extraction patterns (optional)
    config: Option<SlotExtractionConfig>,
    /// Compiled lender patterns from config
    config_lenders: HashMap<String, Vec<String>>,
}

impl SlotExtractor {
    /// Create a new slot extractor using only static fallback patterns
    ///
    /// All patterns are static and compiled once at program start,
    /// so this is a very cheap operation.
    pub fn new() -> Self {
        Self {
            config: None,
            config_lenders: HashMap::new(),
        }
    }

    /// P16 FIX: Create a slot extractor with domain-specific configuration
    ///
    /// This allows loading extraction patterns from slots.yaml config file
    /// for domain-agnostic operation.
    pub fn from_config(config: SlotExtractionConfig) -> Self {
        let config_lenders = config.lenders.clone();
        Self {
            config: Some(config),
            config_lenders,
        }
    }

    /// P16 FIX: Create with lender patterns from domain config
    ///
    /// Example usage with voice_agent_config:
    /// ```ignore
    /// let mut lenders = HashMap::new();
    /// for slot in &slots_config.slots {
    ///     if slot.name == "current_lender" {
    ///         if let Some(values) = &slot.values {
    ///             for value in values {
    ///                 if let Some(patterns) = &value.patterns {
    ///                     lenders.insert(value.id.clone(), patterns.clone());
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// let extractor = SlotExtractor::with_lenders(lenders);
    /// ```
    pub fn with_lenders(lenders: HashMap<String, Vec<String>>) -> Self {
        Self::from_config(SlotExtractionConfig {
            custom_patterns: HashMap::new(),
            lenders,
            intent_patterns: Vec::new(),
        })
    }

    /// Extract all slots from an utterance
    pub fn extract(&self, utterance: &str) -> HashMap<String, Slot> {
        let mut slots = HashMap::new();

        // Extract amount
        if let Some((amount, confidence)) = self.extract_amount(utterance) {
            slots.insert("loan_amount".to_string(), Slot {
                name: "loan_amount".to_string(),
                value: Some(amount.to_string()),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract weight
        if let Some((weight, confidence)) = self.extract_weight(utterance) {
            slots.insert("gold_weight".to_string(), Slot {
                name: "gold_weight".to_string(),
                value: Some(weight.to_string()),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract phone
        if let Some((phone, confidence)) = self.extract_phone(utterance) {
            slots.insert("phone_number".to_string(), Slot {
                name: "phone_number".to_string(),
                value: Some(phone),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract pincode
        if let Some((pincode, confidence)) = self.extract_pincode(utterance) {
            slots.insert("pincode".to_string(), Slot {
                name: "pincode".to_string(),
                value: Some(pincode),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract lender
        if let Some((lender, confidence)) = self.extract_lender(utterance) {
            slots.insert("current_lender".to_string(), Slot {
                name: "current_lender".to_string(),
                value: Some(lender),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract purity
        if let Some((purity, confidence)) = self.extract_purity(utterance) {
            slots.insert("gold_purity".to_string(), Slot {
                name: "gold_purity".to_string(),
                value: Some(purity),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract purpose
        if let Some((purpose, confidence)) = self.extract_purpose(utterance) {
            slots.insert("loan_purpose".to_string(), Slot {
                name: "loan_purpose".to_string(),
                value: Some(purpose),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract location
        if let Some((location, confidence)) = self.extract_location(utterance) {
            slots.insert("location".to_string(), Slot {
                name: "location".to_string(),
                value: Some(location),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract customer name
        if let Some((name, confidence)) = self.extract_name(utterance) {
            slots.insert("customer_name".to_string(), Slot {
                name: "customer_name".to_string(),
                value: Some(name),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract PAN number
        if let Some((pan, confidence)) = self.extract_pan(utterance) {
            slots.insert("pan_number".to_string(), Slot {
                name: "pan_number".to_string(),
                value: Some(pan),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract date of birth
        if let Some((dob, confidence)) = self.extract_dob(utterance) {
            slots.insert("date_of_birth".to_string(), Slot {
                name: "date_of_birth".to_string(),
                value: Some(dob),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract interest rate
        if let Some((rate, confidence)) = self.extract_interest_rate(utterance) {
            slots.insert("current_interest_rate".to_string(), Slot {
                name: "current_interest_rate".to_string(),
                value: Some(rate.to_string()),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract tenure
        if let Some((tenure, confidence)) = self.extract_tenure(utterance) {
            slots.insert("tenure_months".to_string(), Slot {
                name: "tenure_months".to_string(),
                value: Some(tenure.to_string()),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract repayment type preference
        if let Some((repayment_type, confidence)) = self.extract_repayment_type(utterance) {
            slots.insert("repayment_type".to_string(), Slot {
                name: "repayment_type".to_string(),
                value: Some(repayment_type),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract city
        if let Some((city, confidence)) = self.extract_city(utterance) {
            slots.insert("city".to_string(), Slot {
                name: "city".to_string(),
                value: Some(city),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        // Extract detected intent (helps LLM understand what user wants)
        if let Some((intent, confidence)) = self.extract_intent(utterance) {
            slots.insert("detected_intent".to_string(), Slot {
                name: "detected_intent".to_string(),
                value: Some(intent),
                confidence,
                slot_type: SlotType::Text,
            });
        }

        slots
    }

    /// Extract amount from utterance
    pub fn extract_amount(&self, utterance: &str) -> Option<(f64, f32)> {
        let lower = utterance.to_lowercase();

        for (pattern, multiplier) in AMOUNT_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(&lower) {
                if let Some(num_match) = caps.get(1) {
                    let num_str = num_match.as_str().replace(',', "");
                    if let Ok(num) = num_str.parse::<f64>() {
                        let amount = num * multiplier.value();

                        // Skip if looks like a phone number (10-digit starting with 6-9)
                        let clean_str = num_str.replace(',', "");
                        if clean_str.len() == 10 {
                            if let Some(first) = clean_str.chars().next() {
                                if first >= '6' && first <= '9' {
                                    tracing::debug!(
                                        value = %clean_str,
                                        "Skipping amount extraction - looks like phone number"
                                    );
                                    continue;
                                }
                            }
                        }

                        // Skip if unreasonably large (> 100 crore)
                        if amount > 1_000_000_000.0 {
                            tracing::debug!(
                                amount = amount,
                                "Skipping amount extraction - unreasonably large"
                            );
                            continue;
                        }

                        // Confidence based on context
                        let confidence = if lower.contains("loan") || lower.contains("lakh")
                            || lower.contains("amount") || lower.contains("chahiye")
                        {
                            0.9
                        } else {
                            0.7
                        };

                        return Some((amount, confidence));
                    }
                }
            }
        }

        None
    }

    /// Extract weight from utterance
    pub fn extract_weight(&self, utterance: &str) -> Option<(f64, f32)> {
        let lower = utterance.to_lowercase();

        for pattern in WEIGHT_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(&lower) {
                if let Some(num_match) = caps.get(1) {
                    if let Ok(num) = num_match.as_str().parse::<f64>() {
                        // Check if it's tola (convert to grams)
                        let weight = if lower.contains("tola") || lower.contains("तोला") {
                            num * 11.66 // 1 tola ≈ 11.66 grams
                        } else {
                            num
                        };

                        // Confidence based on context
                        let confidence = if lower.contains("gold") || lower.contains("sona")
                            || lower.contains("gram") || lower.contains("tola")
                        {
                            0.9
                        } else {
                            0.7
                        };

                        return Some((weight, confidence));
                    }
                }
            }
        }

        None
    }

    /// Extract phone number from utterance
    pub fn extract_phone(&self, utterance: &str) -> Option<(String, f32)> {
        for pattern in PHONE_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(utterance) {
                // Handle formatted numbers
                if caps.len() > 2 {
                    // Formatted pattern with groups
                    let parts: Vec<&str> = caps.iter()
                        .skip(1)
                        .filter_map(|m| m.map(|m| m.as_str()))
                        .collect();
                    let phone = parts.join("");
                    if phone.len() == 10 {
                        return Some((phone, 0.95));
                    }
                } else if let Some(m) = caps.get(1) {
                    let phone = m.as_str().to_string();
                    if phone.len() == 10 {
                        return Some((phone, 0.95));
                    }
                }
            }
        }

        None
    }

    /// Extract pincode from utterance
    pub fn extract_pincode(&self, utterance: &str) -> Option<(String, f32)> {
        for pattern in PINCODE_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(utterance) {
                if let Some(m) = caps.get(1) {
                    let pincode = m.as_str().to_string();
                    // Basic validation - Indian pincodes
                    if pincode.len() == 6 && pincode.chars().next().unwrap() != '0' {
                        let confidence = if utterance.to_lowercase().contains("pincode")
                            || utterance.to_lowercase().contains("pin")
                        {
                            0.95
                        } else {
                            0.7
                        };
                        return Some((pincode, confidence));
                    }
                }
            }
        }

        None
    }

    /// Extract lender name from utterance
    ///
    /// P16 FIX: Uses config-driven lender patterns when available,
    /// falls back to static LENDER_PATTERNS otherwise.
    pub fn extract_lender(&self, utterance: &str) -> Option<(String, f32)> {
        let lower = utterance.to_lowercase();

        // P16 FIX: Try config-driven lenders first
        if !self.config_lenders.is_empty() {
            for (canonical, variants) in &self.config_lenders {
                for variant in variants {
                    if lower.contains(&variant.to_lowercase()) {
                        let confidence = if lower.contains("from") || lower.contains("with")
                            || lower.contains("se") || lower.contains("current")
                        {
                            0.9
                        } else {
                            0.7
                        };
                        return Some((canonical.clone(), confidence));
                    }
                }
            }
        }

        // Fallback to static patterns
        for (canonical, variants) in LENDER_PATTERNS.iter() {
            for variant in variants.iter() {
                if lower.contains(variant) {
                    let confidence = if lower.contains("from") || lower.contains("with")
                        || lower.contains("se") || lower.contains("current")
                    {
                        0.9
                    } else {
                        0.7
                    };
                    return Some(((*canonical).to_string(), confidence));
                }
            }
        }

        None
    }

    /// Extract gold purity from utterance
    pub fn extract_purity(&self, utterance: &str) -> Option<(String, f32)> {
        let lower = utterance.to_lowercase();

        // Use static purity patterns
        if PURITY_24K.is_match(&lower) {
            return Some(("24".to_string(), 0.85));
        }
        if PURITY_22K.is_match(&lower) {
            return Some(("22".to_string(), 0.85));
        }
        if PURITY_18K.is_match(&lower) {
            return Some(("18".to_string(), 0.85));
        }
        if PURITY_14K.is_match(&lower) {
            return Some(("14".to_string(), 0.85));
        }
        if PURITY_PURE.is_match(&lower) {
            return Some(("24".to_string(), 0.85));
        }
        if PURITY_HALLMARK.is_match(&lower) {
            return Some(("22".to_string(), 0.85)); // Hallmarked is typically 22k in India
        }

        None
    }

    /// Extract loan purpose from utterance
    pub fn extract_purpose(&self, utterance: &str) -> Option<(String, f32)> {
        let lower = utterance.to_lowercase();

        let purposes = [
            // Medical
            (vec!["medical", "hospital", "treatment", "surgery", "ilaj", "dawai", "doctor"],
             "medical"),
            // Education
            (vec!["education", "school", "college", "fees", "padhai", "admission"],
             "education"),
            // Business
            (vec!["business", "shop", "dukan", "karobar", "vyapaar", "investment"],
             "business"),
            // Wedding
            (vec!["wedding", "marriage", "shaadi", "vivah", "function"],
             "wedding"),
            // Emergency
            (vec!["emergency", "urgent", "zaruri", "turant"],
             "emergency"),
            // Home
            (vec!["home", "house", "ghar", "renovation", "repair", "construction"],
             "home"),
            // Personal
            (vec!["personal", "family", "apna kaam"],
             "personal"),
        ];

        for (keywords, purpose) in &purposes {
            for keyword in keywords {
                if lower.contains(keyword) {
                    return Some((purpose.to_string(), 0.8));
                }
            }
        }

        None
    }

    /// Extract location from utterance
    pub fn extract_location(&self, utterance: &str) -> Option<(String, f32)> {
        let lower = utterance.to_lowercase();

        // Major Indian cities
        let cities = [
            "mumbai", "delhi", "bangalore", "bengaluru", "chennai", "hyderabad",
            "kolkata", "pune", "ahmedabad", "jaipur", "surat", "lucknow",
            "kanpur", "nagpur", "indore", "thane", "bhopal", "visakhapatnam",
            "patna", "vadodara", "ghaziabad", "ludhiana", "agra", "nashik",
            "faridabad", "meerut", "rajkot", "kalyan", "vasai", "varanasi",
            "aurangabad", "dhanbad", "amritsar", "allahabad", "ranchi", "gwalior",
            "jodhpur", "coimbatore", "vijayawada", "madurai", "raipur", "kota",
        ];

        for city in &cities {
            if lower.contains(city) {
                let confidence = if lower.contains("in ") || lower.contains("at ")
                    || lower.contains("from ") || lower.contains("near ")
                    || lower.contains("mein") || lower.contains("में")
                {
                    0.9
                } else {
                    0.7
                };

                // Capitalize city name
                let capitalized = city.chars().next().unwrap().to_uppercase().to_string()
                    + &city[1..];
                return Some((capitalized, confidence));
            }
        }

        // Try to extract location after keywords using static pattern
        if let Some(caps) = LOCATION_PATTERN.captures(utterance) {
            if let Some(m) = caps.get(1) {
                let location = m.as_str().to_string();
                if location.len() >= 3 && location.len() <= 30 {
                    return Some((location, 0.6));
                }
            }
        }

        None
    }

    /// Extract tenure from utterance
    pub fn extract_tenure(&self, utterance: &str) -> Option<(u32, f32)> {
        let lower = utterance.to_lowercase();

        // Month patterns using static pattern
        if let Some(caps) = TENURE_MONTHS.captures(&lower) {
            if let Some(m) = caps.get(1) {
                if let Ok(months) = m.as_str().parse::<u32>() {
                    if months >= 1 && months <= 60 {
                        return Some((months, 0.85));
                    }
                }
            }
        }

        // Year patterns using static pattern
        if let Some(caps) = TENURE_YEARS.captures(&lower) {
            if let Some(m) = caps.get(1) {
                if let Ok(years) = m.as_str().parse::<u32>() {
                    if years >= 1 && years <= 5 {
                        return Some((years * 12, 0.85));
                    }
                }
            }
        }

        None
    }

    /// Extract interest rate from utterance
    pub fn extract_interest_rate(&self, utterance: &str) -> Option<(f32, f32)> {
        let lower = utterance.to_lowercase();

        // Pattern with explicit rate context using static pattern
        if let Some(caps) = RATE_CONTEXT.captures(&lower) {
            if let Some(m) = caps.get(1) {
                if let Ok(rate) = m.as_str().parse::<f32>() {
                    if rate >= 5.0 && rate <= 30.0 {
                        return Some((rate, 0.9));
                    }
                }
            }
        }

        // Pattern with percent symbol using static pattern
        if let Some(caps) = RATE_PERCENT.captures(&lower) {
            if let Some(m) = caps.get(1) {
                if let Ok(rate) = m.as_str().parse::<f32>() {
                    // Gold loan rates are typically 7-24%
                    if rate >= 5.0 && rate <= 30.0 {
                        return Some((rate, 0.85));
                    }
                }
            }
        }

        None
    }

    /// Extract customer name from utterance
    pub fn extract_name(&self, utterance: &str) -> Option<(String, f32)> {
        for pattern in NAME_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(utterance) {
                if let Some(m) = caps.get(1) {
                    let name = m.as_str().trim().to_string();
                    // Basic validation: name should be 2-50 chars and not be common words
                    if name.len() >= 2 && name.len() <= 50 {
                        let lower = name.to_lowercase();
                        // Filter out common false positives
                        let exclude_words = [
                            "loan", "gold", "bank", "kotak", "muthoot", "amount",
                            "rate", "interest", "help", "need", "want", "please",
                        ];
                        if !exclude_words.iter().any(|w| lower == *w) {
                            return Some((name, 0.85));
                        }
                    }
                }
            }
        }

        None
    }

    /// Extract PAN number from utterance
    pub fn extract_pan(&self, utterance: &str) -> Option<(String, f32)> {
        let upper = utterance.to_uppercase();

        for pattern in PAN_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(&upper) {
                if let Some(m) = caps.get(1) {
                    let pan = m.as_str().to_string();
                    // Validate PAN format: 5 letters + 4 digits + 1 letter
                    if pan.len() == 10 {
                        let chars: Vec<char> = pan.chars().collect();
                        let valid_format = chars[0..5].iter().all(|c| c.is_ascii_alphabetic())
                            && chars[5..9].iter().all(|c| c.is_ascii_digit())
                            && chars[9].is_ascii_alphabetic();

                        if valid_format {
                            return Some((pan, 0.95));
                        }
                    }
                    // Numeric PAN (incomplete/incorrect format)
                    if pan.chars().all(|c| c.is_ascii_digit()) && pan.len() >= 8 {
                        return Some((pan, 0.5)); // Low confidence for numeric-only
                    }
                }
            }
        }

        None
    }

    /// Extract date of birth from utterance
    pub fn extract_dob(&self, utterance: &str) -> Option<(String, f32)> {
        for pattern in DOB_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(utterance) {
                if let Some(m) = caps.get(1) {
                    let dob = m.as_str().trim().to_string();
                    // Basic validation: should look like a date
                    if dob.len() >= 6 && dob.len() <= 30 {
                        return Some((dob, 0.85));
                    }
                }
            }
        }

        None
    }

    /// Extract repayment type preference from utterance
    pub fn extract_repayment_type(&self, utterance: &str) -> Option<(String, f32)> {
        let lower = utterance.to_lowercase();

        for (pattern, repayment_type) in REPAYMENT_PATTERNS.iter() {
            if pattern.is_match(&lower) {
                return Some((repayment_type.to_string(), 0.8));
            }
        }

        None
    }

    /// Extract city from utterance
    pub fn extract_city(&self, utterance: &str) -> Option<(String, f32)> {
        // First try direct city patterns
        for pattern in CITY_PATTERNS.iter() {
            if let Some(caps) = pattern.captures(utterance) {
                if let Some(m) = caps.get(1) {
                    let city = m.as_str().trim().to_string();
                    // Basic validation
                    if city.len() >= 2 && city.len() <= 30 {
                        // Capitalize first letter
                        let capitalized = city.chars().next().unwrap().to_uppercase().to_string()
                            + &city[1..].to_lowercase();
                        return Some((capitalized, 0.85));
                    }
                }
            }
        }

        None
    }

    /// Extract detected intent from utterance (helps small models understand what user wants)
    pub fn extract_intent(&self, utterance: &str) -> Option<(String, f32)> {
        let lower = utterance.to_lowercase();

        // Check all intent patterns and return the first (most specific) match
        for (pattern, intent) in INTENT_PATTERNS.iter() {
            if pattern.is_match(&lower) {
                return Some((intent.to_string(), 0.8));
            }
        }

        None
    }

    /// Extract loan purpose from utterance
    pub fn extract_loan_purpose(&self, utterance: &str) -> Option<(String, f32)> {
        let lower = utterance.to_lowercase();

        for (pattern, purpose) in PURPOSE_PATTERNS.iter() {
            if pattern.is_match(&lower) {
                return Some((purpose.to_string(), 0.8));
            }
        }

        None
    }
}

impl Default for SlotExtractor {
    /// Creates a slot extractor with static fallback patterns only.
    /// For domain-agnostic operation, use `from_slots_config()` instead.
    fn default() -> Self {
        Self::new()
    }
}

/// P16 FIX: Export SlotExtractionConfig for external use
pub use SlotExtractionConfig as ExtractionConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_extraction() {
        let extractor = SlotExtractor::new();

        // Lakh amounts
        let (amount, _) = extractor.extract_amount("I need a loan of 5 lakh").unwrap();
        assert!((amount - 500_000.0).abs() < 1.0);

        let (amount, _) = extractor.extract_amount("mujhe 3.5 lakh chahiye").unwrap();
        assert!((amount - 350_000.0).abs() < 1.0);

        // Crore amounts
        let (amount, _) = extractor.extract_amount("loan of 1 crore").unwrap();
        assert!((amount - 10_000_000.0).abs() < 1.0);

        // Thousand amounts
        let (amount, _) = extractor.extract_amount("50 thousand rupees").unwrap();
        assert!((amount - 50_000.0).abs() < 1.0);
    }

    #[test]
    fn test_weight_extraction() {
        let extractor = SlotExtractor::new();

        // Gram weights
        let (weight, _) = extractor.extract_weight("I have 50 grams of gold").unwrap();
        assert!((weight - 50.0).abs() < 0.1);

        let (weight, _) = extractor.extract_weight("mere paas 100g sona hai").unwrap();
        assert!((weight - 100.0).abs() < 0.1);

        // Tola weights
        let (weight, _) = extractor.extract_weight("5 tola gold").unwrap();
        assert!((weight - 58.3).abs() < 0.1); // 5 * 11.66
    }

    #[test]
    fn test_phone_extraction() {
        let extractor = SlotExtractor::new();

        let (phone, _) = extractor.extract_phone("my number is 9876543210").unwrap();
        assert_eq!(phone, "9876543210");

        let (phone, _) = extractor.extract_phone("call me at +91 8765432109").unwrap();
        assert_eq!(phone, "8765432109");
    }

    #[test]
    fn test_pincode_extraction() {
        let extractor = SlotExtractor::new();

        let (pincode, _) = extractor.extract_pincode("pincode is 400001").unwrap();
        assert_eq!(pincode, "400001");

        let (pincode, _) = extractor.extract_pincode("I'm in 560001").unwrap();
        assert_eq!(pincode, "560001");
    }

    #[test]
    fn test_lender_extraction() {
        // Create extractor with config-driven lender patterns
        let mut lenders = HashMap::new();
        lenders.insert("lender_a".to_string(), vec!["lender a".to_string(), "lender_a".to_string()]);
        lenders.insert("bank_b".to_string(), vec!["bank b".to_string(), "bank_b".to_string()]);
        let extractor = SlotExtractor::with_lenders(lenders);

        // Test with config-provided patterns
        let (lender, _) = extractor.extract_lender("I have loan from lender A").unwrap();
        assert_eq!(lender, "lender_a");

        let (lender, _) = extractor.extract_lender("with Bank B").unwrap();
        assert_eq!(lender, "bank_b");

        // Test that unrecognized lenders return None
        let empty_extractor = SlotExtractor::new();
        assert!(empty_extractor.extract_lender("from unknown provider").is_none());
    }

    #[test]
    fn test_purity_extraction() {
        let extractor = SlotExtractor::new();

        let (purity, _) = extractor.extract_purity("24k gold").unwrap();
        assert_eq!(purity, "24");

        let (purity, _) = extractor.extract_purity("22 karat jewelry").unwrap();
        assert_eq!(purity, "22");
    }

    #[test]
    fn test_purpose_extraction() {
        let extractor = SlotExtractor::new();

        let (purpose, _) = extractor.extract_purpose("for medical treatment").unwrap();
        assert_eq!(purpose, "medical");

        let (purpose, _) = extractor.extract_purpose("business ke liye").unwrap();
        assert_eq!(purpose, "business");

        let (purpose, _) = extractor.extract_purpose("wedding expenses").unwrap();
        assert_eq!(purpose, "wedding");
    }

    #[test]
    fn test_location_extraction() {
        let extractor = SlotExtractor::new();

        let (location, _) = extractor.extract_location("I'm in Mumbai").unwrap();
        assert_eq!(location, "Mumbai");

        let (location, _) = extractor.extract_location("from Bangalore").unwrap();
        assert_eq!(location, "Bangalore");
    }

    #[test]
    fn test_tenure_extraction() {
        let extractor = SlotExtractor::new();

        let (tenure, _) = extractor.extract_tenure("for 12 months").unwrap();
        assert_eq!(tenure, 12);

        let (tenure, _) = extractor.extract_tenure("2 years loan").unwrap();
        assert_eq!(tenure, 24);
    }

    #[test]
    fn test_combined_extraction() {
        let extractor = SlotExtractor::new();

        let utterance = "I want a gold loan of 5 lakh for my 50 grams of 22k gold";
        let slots = extractor.extract(utterance);

        assert!(slots.contains_key("loan_amount"));
        assert!(slots.contains_key("gold_weight"));
        assert!(slots.contains_key("gold_purity"));
    }

    #[test]
    fn test_hindi_extraction() {
        let extractor = SlotExtractor::new();

        let (amount, _) = extractor.extract_amount("mujhe 5 lakh chahiye").unwrap();
        assert!((amount - 500_000.0).abs() < 1.0);

        let (weight, _) = extractor.extract_weight("mere paas 50 gram sona hai").unwrap();
        assert!((weight - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_intent_extraction() {
        let extractor = SlotExtractor::new();

        let (intent, _) = extractor.extract_intent("I want balance transfer").unwrap();
        assert_eq!(intent, "balance_transfer");

        let (intent, _) = extractor.extract_intent("what documents required").unwrap();
        assert_eq!(intent, "document_inquiry");
    }
}
