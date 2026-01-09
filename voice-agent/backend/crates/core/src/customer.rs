//! Customer profile and segmentation types

use serde::{Deserialize, Serialize};

/// Customer segment for personalization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomerSegment {
    /// High-value customers (large asset value, sophisticated)
    HighValue,
    /// Safety-focused, may have had issues with other providers
    TrustSeeker,
    /// New to this service type
    FirstTime,
    /// Rate-focused comparison shoppers
    PriceSensitive,
    /// Women customers (special segment programs)
    Women,
    /// Young urban professionals
    Professional,
}

impl CustomerSegment {
    /// Get segment display name
    pub fn display_name(&self) -> &'static str {
        match self {
            CustomerSegment::HighValue => "High Value",
            CustomerSegment::TrustSeeker => "Trust Seeker",
            CustomerSegment::FirstTime => "First Time",
            CustomerSegment::PriceSensitive => "Price Sensitive",
            CustomerSegment::Women => "Women (Shakti)",
            CustomerSegment::Professional => "Young Professional",
        }
    }

    /// Get key messaging points for this segment
    pub fn key_messages(&self) -> Vec<&'static str> {
        match self {
            CustomerSegment::HighValue => vec![
                "Dedicated relationship manager",
                "Priority processing",
                "Higher loan limits",
                "Exclusive rates",
            ],
            CustomerSegment::TrustSeeker => vec![
                "RBI-regulated scheduled bank",
                "Bank-grade security vaults",
                "Full insurance coverage",
                "Digital tracking of gold",
            ],
            CustomerSegment::FirstTime => vec![
                "Simple process",
                "No hidden charges",
                "Friendly support",
                "Clear documentation",
            ],
            CustomerSegment::PriceSensitive => vec![
                "Lowest interest rates",
                "Zero foreclosure charges",
                "Transparent pricing",
                "Savings calculator",
            ],
            CustomerSegment::Women => vec![
                "Special Shakti Gold program",
                "Preferential rates",
                "Women-only branches available",
                "Flexible repayment",
            ],
            CustomerSegment::Professional => vec![
                "Quick digital process",
                "Mobile app tracking",
                "Instant approval",
                "Flexible tenure options",
            ],
        }
    }

    /// Get suggested persona warmth level (0.0 - 1.0)
    pub fn suggested_warmth(&self) -> f32 {
        match self {
            CustomerSegment::HighValue => 0.9,
            CustomerSegment::TrustSeeker => 0.95,
            CustomerSegment::FirstTime => 0.9,
            CustomerSegment::PriceSensitive => 0.7,
            CustomerSegment::Women => 0.95,
            CustomerSegment::Professional => 0.75,
        }
    }
}

impl std::fmt::Display for CustomerSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Customer profile information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerProfile {
    /// Customer ID (if known)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Customer name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Phone number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,

    /// Customer segment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub segment: Option<CustomerSegment>,

    /// Current provider (competitor)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_lender: Option<String>,

    /// Collateral weight in units (e.g., grams for gold)
    #[serde(skip_serializing_if = "Option::is_none", alias = "gold_weight")]
    pub collateral_weight: Option<f64>,

    /// Collateral variant/grade (e.g., "22K", "24K" for gold)
    #[serde(skip_serializing_if = "Option::is_none", alias = "gold_purity")]
    pub collateral_variant: Option<String>,

    /// Current/desired loan amount
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loan_amount: Option<f64>,

    /// Preferred language (ISO 639-1)
    #[serde(default = "default_language")]
    pub preferred_language: String,

    /// Existing relationship with company
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_with_company: Option<CompanyRelationship>,

    /// City
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,

    /// Pincode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pincode: Option<String>,
}

// Legacy aliases for backwards compatibility
impl CustomerProfile {
    /// Legacy accessor for gold_weight
    pub fn gold_weight(&self) -> Option<f64> {
        self.collateral_weight
    }

    /// Legacy accessor for gold_purity
    pub fn gold_purity(&self) -> Option<&str> {
        self.collateral_variant.as_deref()
    }

}

fn default_language() -> String {
    "en".to_string()
}

impl CustomerProfile {
    /// Create a new empty customer profile
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            phone: None,
            segment: None,
            current_lender: None,
            collateral_weight: None,
            collateral_variant: None,
            loan_amount: None,
            preferred_language: "en".to_string(),
            relationship_with_company: None,
            city: None,
            pincode: None,
        }
    }

    /// Create profile with phone number
    pub fn with_phone(phone: impl Into<String>) -> Self {
        let mut profile = Self::new();
        profile.phone = Some(phone.into());
        profile
    }

    /// Set customer name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set segment
    pub fn segment(mut self, segment: CustomerSegment) -> Self {
        self.segment = Some(segment);
        self
    }

    /// Set current lender/provider
    pub fn current_lender(mut self, lender: impl Into<String>) -> Self {
        self.current_lender = Some(lender.into());
        self
    }

    /// Set collateral details (weight/quantity and variant/grade)
    pub fn collateral(mut self, weight: f64, variant: impl Into<String>) -> Self {
        self.collateral_weight = Some(weight);
        self.collateral_variant = Some(variant.into());
        self
    }

    /// Legacy alias for setting gold details
    pub fn gold(self, weight: f64, purity: impl Into<String>) -> Self {
        self.collateral(weight, purity)
    }

    /// Set preferred language
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        self.preferred_language = lang.into();
        self
    }

    /// Check if customer is a switcher (has current lender)
    pub fn is_switcher(&self) -> bool {
        self.current_lender.is_some()
    }

    /// Check if we have collateral details
    pub fn has_collateral_details(&self) -> bool {
        self.collateral_weight.is_some() && self.collateral_variant.is_some()
    }

    /// Legacy alias for has_collateral_details
    pub fn has_gold_details(&self) -> bool {
        self.has_collateral_details()
    }

    /// Default asset price per unit (INR). Should be fetched from API/config.
    pub const DEFAULT_ASSET_PRICE_PER_UNIT: f64 = 7500.0;

    /// Legacy alias for DEFAULT_ASSET_PRICE_PER_UNIT
    pub const DEFAULT_GOLD_PRICE_PER_GRAM: f64 = Self::DEFAULT_ASSET_PRICE_PER_UNIT;

    /// Estimate collateral value
    ///
    /// P2 FIX: Now accepts configurable asset_price_per_unit parameter.
    /// Use `CustomerProfile::DEFAULT_ASSET_PRICE_PER_UNIT` or pass value from config.
    pub fn estimated_collateral_value(&self, asset_price_per_unit: Option<f64>) -> Option<f64> {
        let weight = self.collateral_weight?;
        let variant = self.collateral_variant.as_ref()?;

        let base_price = asset_price_per_unit.unwrap_or(Self::DEFAULT_ASSET_PRICE_PER_UNIT);

        // Variant factors - these should ideally come from config
        let variant_factor = match variant.to_uppercase().as_str() {
            "24K" | "HIGH_GRADE" => 1.0,
            "22K" | "STANDARD_GRADE" => 0.916,
            "18K" | "LOWER_GRADE" => 0.75,
            "14K" => 0.585,
            _ => 0.75, // Default to lower grade
        };

        Some(weight * base_price * variant_factor)
    }

    /// Legacy alias for estimated_collateral_value
    pub fn estimated_gold_value(&self, gold_price_per_gram: Option<f64>) -> Option<f64> {
        self.estimated_collateral_value(gold_price_per_gram)
    }

    /// Get display name (name or "Customer")
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("Customer")
    }

    /// Infer segment from profile data
    pub fn infer_segment(&self) -> Option<CustomerSegment> {
        // Already has segment
        if self.segment.is_some() {
            return self.segment;
        }

        // High value: >100 units collateral OR loan amount > 5 lakhs
        if let Some(weight) = self.collateral_weight {
            if weight >= 100.0 {
                return Some(CustomerSegment::HighValue);
            }
        }
        if let Some(amount) = self.loan_amount {
            if amount >= 500_000.0 {
                return Some(CustomerSegment::HighValue);
            }
        }

        // Trust seeker: switching from NBFC due to issues
        // Note: For config-driven competitor detection, use primary_segment_with_config()
        if self.current_lender.is_some() {
            // Having any current lender suggests potential for TrustSeeker segment
            // Full competitor matching done via primary_segment_with_config()
            return Some(CustomerSegment::TrustSeeker);
        }

        // First time: no current lender and no collateral details
        if self.current_lender.is_none() && !self.has_collateral_details() {
            return Some(CustomerSegment::FirstTime);
        }

        None
    }
}

/// P3-2 FIX: Segment detector that analyzes conversation signals
///
/// Uses multiple signals to auto-detect customer segment:
/// - Loan amount mentioned → High Value
/// - Urgency signals → marks as urgent (influences persona)
/// - Price/rate focus → Price Sensitive
/// - Safety concerns → Trust Seeker
/// - Comparison mentions → Price Sensitive
#[derive(Debug, Clone)]
pub struct SegmentDetector {
    /// Weight for high-value detection (default: 100g)
    pub high_value_gold_threshold: f64,
    /// Amount for high-value detection (default: 5 lakhs)
    pub high_value_amount_threshold: f64,
}

impl Default for SegmentDetector {
    fn default() -> Self {
        Self {
            high_value_gold_threshold: 100.0,
            high_value_amount_threshold: 500_000.0,
        }
    }
}

impl SegmentDetector {
    /// Create a new segment detector
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom thresholds
    pub fn with_thresholds(gold_grams: f64, amount_inr: f64) -> Self {
        Self {
            high_value_gold_threshold: gold_grams,
            high_value_amount_threshold: amount_inr,
        }
    }

    /// Detect segment from text content (conversation transcript)
    pub fn detect_from_text(&self, text: &str) -> Option<CustomerSegment> {
        let lower = text.to_lowercase();

        // High Value: Large amounts mentioned
        if self.detect_high_value_amount(&lower) {
            return Some(CustomerSegment::HighValue);
        }

        // Price Sensitive: Rate/price focused
        if self.detect_price_sensitivity(&lower) {
            return Some(CustomerSegment::PriceSensitive);
        }

        // Trust Seeker: Safety/security concerns
        if self.detect_trust_seeking(&lower) {
            return Some(CustomerSegment::TrustSeeker);
        }

        // First Time: New to gold loans
        if self.detect_first_time(&lower) {
            return Some(CustomerSegment::FirstTime);
        }

        None
    }

    /// Detect high-value customer from text
    fn detect_high_value_amount(&self, text: &str) -> bool {
        // Hindi: lakh, crore
        // Look for amounts like "5 lakh", "10 lakh", "1 crore"
        let high_value_patterns = [
            "lakh",
            "lakhs",
            "lac",
            "lacs",
            "crore",
            "crores",
            "500000",
            "1000000",
            "पाँच लाख",
            "दस लाख",
            "करोड़",
        ];

        // Check for large amounts
        for pattern in &high_value_patterns {
            if text.contains(pattern) {
                // Extract number before lakh/crore
                if pattern.contains("lakh") || pattern.contains("lac") {
                    // Check if >= 5 lakhs
                    if self.extract_lakh_amount(text) >= 5.0 {
                        return true;
                    }
                } else if pattern.contains("crore") || pattern.contains("करोड़") {
                    return true; // Any crore amount is high value
                }
            }
        }

        // Check for large gold weights
        let gold_patterns = ["100 gram", "150 gram", "200 gram", "सौ ग्राम"];
        for pattern in &gold_patterns {
            if text.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Extract lakh amount from text
    fn extract_lakh_amount(&self, text: &str) -> f64 {
        // Simple extraction: look for "N lakh" pattern
        let words: Vec<&str> = text.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if (word.contains("lakh") || word.contains("lac")) && i > 0 {
                if let Ok(n) = words[i - 1].parse::<f64>() {
                    return n;
                }
                // Hindi number words
                match words[i - 1] {
                    "एक" | "ek" => return 1.0,
                    "दो" | "do" => return 2.0,
                    "तीन" | "teen" => return 3.0,
                    "चार" | "char" => return 4.0,
                    "पाँच" | "paanch" | "panch" => return 5.0,
                    "दस" | "das" => return 10.0,
                    _ => {},
                }
            }
        }
        0.0
    }

    /// Detect price-sensitive customer
    fn detect_price_sensitivity(&self, text: &str) -> bool {
        let price_patterns = [
            // English
            "interest rate",
            "lowest rate",
            "best rate",
            "cheaper",
            "how much interest",
            "what rate",
            "rate kitna",
            "compare rate",
            "other bank",
            "better rate",
            "processing fee",
            "hidden charge",
            "total cost",
            // Hindi
            "kitna byaj",
            "byaj dar",
            "sasta",
            "mehnga",
            "sabse kam",
            "ब्याज दर",
            "सस्ता",
            "महंगा",
        ];

        price_patterns.iter().any(|p| text.contains(p))
    }

    /// Detect trust-seeking customer (config-driven)
    ///
    /// # Arguments
    /// * `text` - The text to analyze
    /// * `competitor_names` - Optional list of competitor names to check
    pub fn detect_trust_seeking_with_config(&self, text: &str, competitor_names: &[String]) -> bool {
        // Check safety patterns
        if self.detect_trust_seeking(text) {
            return true;
        }
        // Check competitor mentions
        competitor_names.iter().any(|c| text.contains(&c.to_lowercase()))
    }

    /// Detect trust-seeking customer (basic patterns only, no competitor matching)
    fn detect_trust_seeking(&self, text: &str) -> bool {
        let trust_patterns = [
            // Safety concerns
            "gold safe",
            "safe hai",
            "surakshit",
            "insurance",
            "vault",
            "locker",
            "rbi",
            "regulated",
            "government",
            // Past issues
            "problem",
            "issue",
            "fraud",
            "cheat",
            "lost gold",
            "gold missing",
            // Hindi
            "सोना सुरक्षित",
            "भरोसा",
            "विश्वास",
        ];

        trust_patterns.iter().any(|p| text.contains(p))
    }

    /// Detect first-time customer
    fn detect_first_time(&self, text: &str) -> bool {
        let first_time_patterns = [
            "first time",
            "pehli baar",
            "pahli bar",
            "never taken",
            "new to",
            "how does it work",
            "kaise hota hai",
            "process kya hai",
            "what is",      // Generic: "what is [product]"
            "kya hai",      // Generic: "[product] kya hai"
            "पहली बार",
            "कैसे होता है",
        ];

        first_time_patterns.iter().any(|p| text.contains(p))
    }

    /// Detect urgency in text (doesn't map to segment but influences persona)
    pub fn detect_urgency(&self, text: &str) -> bool {
        let urgency_patterns = [
            // English
            "urgent",
            "emergency",
            "asap",
            "today",
            "right now",
            "immediately",
            "quick",
            "fast",
            "need money",
            "hospital",
            "medical",
            // Hindi
            "jaldi",
            "abhi",
            "turant",
            "fauran",
            "जल्दी",
            "अभी",
            "तुरंत",
            "फौरन",
            "paise chahiye",
            "पैसे चाहिए",
        ];

        urgency_patterns.iter().any(|p| text.contains(p))
    }

    /// Combined detection from profile and conversation text
    pub fn detect_combined(
        &self,
        profile: &CustomerProfile,
        conversation_text: Option<&str>,
    ) -> Option<CustomerSegment> {
        // First try profile-based detection
        if let Some(segment) = profile.infer_segment() {
            return Some(segment);
        }

        // Then try text-based detection
        if let Some(text) = conversation_text {
            if let Some(segment) = self.detect_from_text(text) {
                return Some(segment);
            }
        }

        None
    }
}

impl Default for CustomerProfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Existing relationship with company (domain-agnostic)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyRelationship {
    /// Is existing customer
    pub is_customer: bool,
    /// Products held
    #[serde(default)]
    pub products: Vec<String>,
    /// Account vintage in months
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vintage_months: Option<u32>,
    /// Is priority/premium customer
    #[serde(default)]
    pub is_priority: bool,
}

impl CompanyRelationship {
    pub fn new_customer() -> Self {
        Self {
            is_customer: false,
            products: Vec::new(),
            vintage_months: None,
            is_priority: false,
        }
    }

    pub fn existing(products: Vec<String>) -> Self {
        Self {
            is_customer: true,
            products,
            vintage_months: None,
            is_priority: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_profile() {
        let profile = CustomerProfile::with_phone("9876543210")
            .name("Raj Kumar")
            .current_lender("Muthoot Finance")
            .gold(50.0, "22K")
            .language("hi");

        assert!(profile.is_switcher());
        assert!(profile.has_gold_details());
        assert!(profile.estimated_gold_value(None).is_some());
    }

    #[test]
    fn test_segment_inference() {
        // High value
        let profile = CustomerProfile::new().gold(150.0, "22K");
        assert_eq!(profile.infer_segment(), Some(CustomerSegment::HighValue));

        // Trust seeker (IIFL customer)
        let profile = CustomerProfile::new().current_lender("IIFL Finance");
        assert_eq!(profile.infer_segment(), Some(CustomerSegment::TrustSeeker));

        // First time
        let profile = CustomerProfile::new();
        assert_eq!(profile.infer_segment(), Some(CustomerSegment::FirstTime));
    }

    #[test]
    fn test_segment_messages() {
        let segment = CustomerSegment::TrustSeeker;
        let messages = segment.key_messages();
        assert!(!messages.is_empty());
        assert!(messages.iter().any(|m| m.contains("RBI")));
    }

    // P3-2 FIX: Tests for SegmentDetector
    #[test]
    fn test_segment_detector_high_value() {
        let detector = SegmentDetector::new();

        // Amount mention in lakhs
        assert_eq!(
            detector.detect_from_text("I need 10 lakh rupees loan"),
            Some(CustomerSegment::HighValue)
        );

        // Crore mention
        assert_eq!(
            detector.detect_from_text("mujhe 1 crore chahiye"),
            Some(CustomerSegment::HighValue)
        );

        // Large gold weight
        assert_eq!(
            detector.detect_from_text("I have 150 gram gold"),
            Some(CustomerSegment::HighValue)
        );
    }

    #[test]
    fn test_segment_detector_price_sensitive() {
        let detector = SegmentDetector::new();

        assert_eq!(
            detector.detect_from_text("What is your interest rate?"),
            Some(CustomerSegment::PriceSensitive)
        );

        assert_eq!(
            detector.detect_from_text("other bank is offering better rate"),
            Some(CustomerSegment::PriceSensitive)
        );

        // Hindi
        assert_eq!(
            detector.detect_from_text("byaj dar kitna hai"),
            Some(CustomerSegment::PriceSensitive)
        );
    }

    #[test]
    fn test_segment_detector_trust_seeker() {
        let detector = SegmentDetector::new();

        assert_eq!(
            detector.detect_from_text("Is my gold safe with you?"),
            Some(CustomerSegment::TrustSeeker)
        );

        assert_eq!(
            detector.detect_from_text("I had issues with Muthoot before"),
            Some(CustomerSegment::TrustSeeker)
        );

        // RBI mention
        assert_eq!(
            detector.detect_from_text("Are you RBI regulated?"),
            Some(CustomerSegment::TrustSeeker)
        );
    }

    #[test]
    fn test_segment_detector_first_time() {
        let detector = SegmentDetector::new();

        assert_eq!(
            detector.detect_from_text("This is my first time taking a gold loan"),
            Some(CustomerSegment::FirstTime)
        );

        assert_eq!(
            detector.detect_from_text("Gold loan kaise hota hai?"),
            Some(CustomerSegment::FirstTime)
        );
    }

    #[test]
    fn test_segment_detector_urgency() {
        let detector = SegmentDetector::new();

        assert!(detector.detect_urgency("I need money urgently for hospital"));
        assert!(detector.detect_urgency("paise jaldi chahiye"));
        assert!(detector.detect_urgency("emergency hai"));
        assert!(!detector.detect_urgency("I'm just exploring options"));
    }

    #[test]
    fn test_segment_detector_combined() {
        let detector = SegmentDetector::new();

        // Profile takes precedence
        let profile = CustomerProfile::new().gold(150.0, "22K");
        assert_eq!(
            detector.detect_combined(&profile, Some("what is your rate")),
            Some(CustomerSegment::HighValue)
        );

        // Falls back to text when profile doesn't determine segment
        // (profile with gold details but not enough to be high value)
        let profile = CustomerProfile::new().gold(20.0, "22K");
        assert_eq!(
            detector.detect_combined(&profile, Some("what is your interest rate")),
            Some(CustomerSegment::PriceSensitive)
        );
    }

    #[test]
    fn test_high_value_from_loan_amount() {
        // Test that loan_amount field also triggers high value
        let profile = CustomerProfile {
            loan_amount: Some(600_000.0),
            ..CustomerProfile::new()
        };
        assert_eq!(profile.infer_segment(), Some(CustomerSegment::HighValue));
    }
}
