//! Hindi Language Utilities
//!
//! P2.2 FIX: Shared utilities for Hindi text processing.
//! Consolidates duplicate Hindi number conversion code from entities and intent modules.

/// Convert Hindi number word (Devanagari script) to numeric value
///
/// Handles common Hindi number words in Devanagari script.
/// For romanized Hindi (ek, do, teen), see vocabulary.yaml config.
///
/// # Examples
/// ```
/// use voice_agent_text_processing::hindi::word_to_number;
/// assert_eq!(word_to_number("पांच"), Some(5.0));
/// assert_eq!(word_to_number("दस"), Some(10.0));
/// assert_eq!(word_to_number("सौ"), Some(100.0));
/// ```
pub fn word_to_number(word: &str) -> Option<f64> {
    match word {
        // Basic numbers 1-10
        "एक" => Some(1.0),
        "दो" => Some(2.0),
        "तीन" => Some(3.0),
        "चार" => Some(4.0),
        "पांच" | "पाँच" => Some(5.0),
        "छह" | "छः" | "छे" => Some(6.0), // All common variants
        "सात" => Some(7.0),
        "आठ" => Some(8.0),
        "नौ" => Some(9.0),
        "दस" => Some(10.0),

        // Tens
        "बीस" => Some(20.0),
        "पच्चीस" => Some(25.0),
        "तीस" => Some(30.0),
        "पैंतीस" => Some(35.0),
        "चालीस" => Some(40.0),
        "पचास" => Some(50.0),
        "साठ" => Some(60.0),
        "सत्तर" => Some(70.0),
        "अस्सी" => Some(80.0),
        "नब्बे" => Some(90.0),
        "सौ" => Some(100.0),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_numbers() {
        assert_eq!(word_to_number("एक"), Some(1.0));
        assert_eq!(word_to_number("पांच"), Some(5.0));
        assert_eq!(word_to_number("पाँच"), Some(5.0)); // Alternate spelling
        assert_eq!(word_to_number("दस"), Some(10.0));
    }

    #[test]
    fn test_six_variants() {
        assert_eq!(word_to_number("छह"), Some(6.0));
        assert_eq!(word_to_number("छः"), Some(6.0));
        assert_eq!(word_to_number("छे"), Some(6.0));
    }

    #[test]
    fn test_tens() {
        assert_eq!(word_to_number("बीस"), Some(20.0));
        assert_eq!(word_to_number("पच्चीस"), Some(25.0));
        assert_eq!(word_to_number("पचास"), Some(50.0));
        assert_eq!(word_to_number("सौ"), Some(100.0));
    }

    #[test]
    fn test_unknown() {
        assert_eq!(word_to_number("unknown"), None);
        assert_eq!(word_to_number("hello"), None);
    }
}
