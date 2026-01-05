//! Slot Value Extractor for Gold Loan Conversations
//!
//! Implements rule-based and pattern-based slot extraction from user utterances.
//! Supports Hindi, Hinglish, and English utterances.

use regex::Regex;
use std::collections::HashMap;
use voice_agent_text_processing::intent::{Slot, SlotType};

/// Slot extractor for gold loan domain
pub struct SlotExtractor {
    /// Regex patterns for amount extraction
    amount_patterns: Vec<(Regex, AmountMultiplier)>,
    /// Regex patterns for weight extraction
    weight_patterns: Vec<Regex>,
    /// Regex patterns for phone extraction
    phone_patterns: Vec<Regex>,
    /// Regex patterns for pincode extraction
    pincode_patterns: Vec<Regex>,
    /// Regex patterns for time extraction
    time_patterns: Vec<Regex>,
    /// Lender name patterns
    lender_patterns: HashMap<String, Vec<String>>,
}

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

impl SlotExtractor {
    /// Create a new slot extractor
    pub fn new() -> Self {
        Self {
            amount_patterns: Self::build_amount_patterns(),
            weight_patterns: Self::build_weight_patterns(),
            phone_patterns: Self::build_phone_patterns(),
            pincode_patterns: Self::build_pincode_patterns(),
            time_patterns: Self::build_time_patterns(),
            lender_patterns: Self::build_lender_patterns(),
        }
    }

    fn build_amount_patterns() -> Vec<(Regex, AmountMultiplier)> {
        vec![
            // Crore patterns
            (Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:crore|cr|करोड़)").unwrap(), AmountMultiplier::Crore),
            // Lakh patterns (English and Hindi)
            (Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:lakh|lac|लाख)").unwrap(), AmountMultiplier::Lakh),
            // Thousand patterns
            (Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:thousand|k|हज़ार|hazar)").unwrap(), AmountMultiplier::Thousand),
            // Direct rupee amounts
            (Regex::new(r"(?:₹|rs\.?|rupees?)\s*(\d+(?:,\d+)*)").unwrap(), AmountMultiplier::Unit),
            // Plain large numbers (likely amounts)
            (Regex::new(r"\b(\d{5,7})\b").unwrap(), AmountMultiplier::Unit),
        ]
    }

    fn build_weight_patterns() -> Vec<Regex> {
        vec![
            // Grams patterns
            Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:grams?|gm|g|ग्राम)").unwrap(),
            // Tola patterns (1 tola ≈ 11.66g)
            Regex::new(r"(?i)(\d+(?:\.\d+)?)\s*(?:tola|तोला)").unwrap(),
            // Contextual weight (e.g., "I have 50 grams gold")
            Regex::new(r"(?i)(?:have|hai|है)\s*(\d+(?:\.\d+)?)\s*(?:grams?|g)?\s*(?:gold|sona|सोना)").unwrap(),
        ]
    }

    fn build_phone_patterns() -> Vec<Regex> {
        vec![
            // Indian mobile numbers (10 digits starting with 6-9)
            Regex::new(r"\b([6-9]\d{9})\b").unwrap(),
            // With country code
            Regex::new(r"(?:\+91|91)?[-\s]?([6-9]\d{9})\b").unwrap(),
            // Formatted numbers
            Regex::new(r"\b([6-9]\d{2})[-\s]?(\d{3})[-\s]?(\d{4})\b").unwrap(),
        ]
    }

    fn build_pincode_patterns() -> Vec<Regex> {
        vec![
            // Indian pincodes (6 digits, first digit 1-9)
            Regex::new(r"\b([1-9]\d{5})\b").unwrap(),
            // With "pincode" keyword
            Regex::new(r"(?i)(?:pincode|pin|पिनकोड)\s*(?:is|hai|है)?\s*(\d{6})").unwrap(),
        ]
    }

    fn build_time_patterns() -> Vec<Regex> {
        vec![
            // Time formats
            Regex::new(r"(?i)(\d{1,2})(?::(\d{2}))?\s*(am|pm|बजे)").unwrap(),
            // Time slots
            Regex::new(r"(?i)(morning|afternoon|evening|subah|dopahar|shaam)").unwrap(),
        ]
    }

    fn build_lender_patterns() -> HashMap<String, Vec<String>> {
        let mut patterns = HashMap::new();

        patterns.insert("muthoot".to_string(), vec![
            "muthoot".to_string(),
            "muthut".to_string(),
            "muthoot finance".to_string(),
        ]);

        patterns.insert("manappuram".to_string(), vec![
            "manappuram".to_string(),
            "manapuram".to_string(),
            "manappuram gold".to_string(),
        ]);

        patterns.insert("hdfc".to_string(), vec![
            "hdfc".to_string(),
            "hdfc bank".to_string(),
        ]);

        patterns.insert("icici".to_string(), vec![
            "icici".to_string(),
            "icici bank".to_string(),
        ]);

        patterns.insert("sbi".to_string(), vec![
            "sbi".to_string(),
            "state bank".to_string(),
        ]);

        patterns.insert("kotak".to_string(), vec![
            "kotak".to_string(),
            "kotak mahindra".to_string(),
        ]);

        patterns.insert("axis".to_string(), vec![
            "axis".to_string(),
            "axis bank".to_string(),
        ]);

        patterns.insert("federal".to_string(), vec![
            "federal".to_string(),
            "federal bank".to_string(),
        ]);

        patterns.insert("iifl".to_string(), vec![
            "iifl".to_string(),
            "india infoline".to_string(),
        ]);

        patterns
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

        slots
    }

    /// Extract amount from utterance
    pub fn extract_amount(&self, utterance: &str) -> Option<(f64, f32)> {
        let lower = utterance.to_lowercase();

        for (pattern, multiplier) in &self.amount_patterns {
            if let Some(caps) = pattern.captures(&lower) {
                if let Some(num_match) = caps.get(1) {
                    let num_str = num_match.as_str().replace(',', "");
                    if let Ok(num) = num_str.parse::<f64>() {
                        let amount = num * multiplier.value();

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

        for pattern in &self.weight_patterns {
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
        for pattern in &self.phone_patterns {
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
        for pattern in &self.pincode_patterns {
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
    pub fn extract_lender(&self, utterance: &str) -> Option<(String, f32)> {
        let lower = utterance.to_lowercase();

        for (canonical, variants) in &self.lender_patterns {
            for variant in variants {
                if lower.contains(variant) {
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

        None
    }

    /// Extract gold purity from utterance
    pub fn extract_purity(&self, utterance: &str) -> Option<(String, f32)> {
        let lower = utterance.to_lowercase();

        // Direct karat mentions
        let purity_patterns = [
            (r"24\s*(?:k|karat|carat|kt)", "24"),
            (r"22\s*(?:k|karat|carat|kt)", "22"),
            (r"18\s*(?:k|karat|carat|kt)", "18"),
            (r"14\s*(?:k|karat|carat|kt)", "14"),
            // Descriptive
            (r"pure\s*gold", "24"),
            (r"hallmark(?:ed)?", "22"), // Hallmarked is typically 22k in India
        ];

        for (pattern, purity) in &purity_patterns {
            if let Ok(re) = Regex::new(&format!("(?i){}", pattern)) {
                if re.is_match(&lower) {
                    return Some((purity.to_string(), 0.85));
                }
            }
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

        // Try to extract location after keywords
        let location_patterns = [
            Regex::new(r"(?i)(?:from|in|at|near|mein|में)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)?)").unwrap(),
        ];

        for pattern in &location_patterns {
            if let Some(caps) = pattern.captures(utterance) {
                if let Some(m) = caps.get(1) {
                    let location = m.as_str().to_string();
                    if location.len() >= 3 && location.len() <= 30 {
                        return Some((location, 0.6));
                    }
                }
            }
        }

        None
    }

    /// Extract tenure from utterance
    pub fn extract_tenure(&self, utterance: &str) -> Option<(u32, f32)> {
        let lower = utterance.to_lowercase();

        // Month patterns
        let month_pattern = Regex::new(r"(\d+)\s*(?:months?|mahine|महीने)").unwrap();
        if let Some(caps) = month_pattern.captures(&lower) {
            if let Some(m) = caps.get(1) {
                if let Ok(months) = m.as_str().parse::<u32>() {
                    if months >= 1 && months <= 60 {
                        return Some((months, 0.85));
                    }
                }
            }
        }

        // Year patterns
        let year_pattern = Regex::new(r"(\d+)\s*(?:years?|saal|साल)").unwrap();
        if let Some(caps) = year_pattern.captures(&lower) {
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

        let rate_pattern = Regex::new(r"(\d+(?:\.\d+)?)\s*(?:%|percent|प्रतिशत)").unwrap();
        if let Some(caps) = rate_pattern.captures(&lower) {
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
}

impl Default for SlotExtractor {
    fn default() -> Self {
        Self::new()
    }
}

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
        let extractor = SlotExtractor::new();

        let (lender, _) = extractor.extract_lender("I have loan from Muthoot").unwrap();
        assert_eq!(lender, "muthoot");

        let (lender, _) = extractor.extract_lender("with HDFC bank").unwrap();
        assert_eq!(lender, "hdfc");
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
}
