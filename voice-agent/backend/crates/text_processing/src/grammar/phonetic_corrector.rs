//! Deterministic Phonetic Error Corrector
//!
//! Uses a combination of:
//! 1. Domain-specific confusion rules (hardcoded phonetic mappings)
//! 2. SymSpell for fast fuzzy matching against domain vocabulary
//! 3. Double Metaphone for phonetic similarity scoring
//!
//! This provides consistent, deterministic corrections unlike LLM-based approaches.

use std::collections::HashMap;
use symspell::{SymSpell, SymSpellBuilder, UnicodeStringStrategy, Verbosity};

/// Configuration for the phonetic corrector
#[derive(Debug, Clone)]
pub struct PhoneticCorrectorConfig {
    /// Maximum edit distance for SymSpell lookups
    pub max_edit_distance: i64,
    /// Minimum word length to attempt correction
    pub min_word_length: usize,
    /// Whether to apply sentence-start corrections (e.g., "Why" -> "I")
    pub fix_sentence_start: bool,
}

impl Default for PhoneticCorrectorConfig {
    fn default() -> Self {
        Self {
            max_edit_distance: 2,
            min_word_length: 3,
            fix_sentence_start: true,
        }
    }
}

/// Deterministic phonetic corrector for ASR errors
pub struct PhoneticCorrector {
    /// SymSpell instance for fuzzy matching
    symspell: SymSpell<UnicodeStringStrategy>,
    /// Direct phonetic confusion mappings (lowercase key -> correction)
    confusion_rules: HashMap<String, String>,
    /// Contextual rules: (context_word, error_word) -> correction
    contextual_rules: HashMap<(String, String), String>,
    /// Phrase-level corrections (lowercase phrase -> correction)
    phrase_rules: Vec<(String, String)>,
    /// Double Metaphone codes for domain vocabulary
    metaphone_index: HashMap<String, Vec<String>>,
    /// Configuration
    config: PhoneticCorrectorConfig,
}

impl PhoneticCorrector {
    /// Create a new phonetic corrector with domain vocabulary only
    ///
    /// NOTE: This creates a corrector with vocabulary for SymSpell but NO
    /// confusion/contextual/phrase rules. Use `from_domain_config()` for full
    /// production config-driven setup.
    pub fn new(vocabulary: Vec<String>, config: PhoneticCorrectorConfig) -> Self {
        // Build SymSpell dictionary from vocabulary
        let mut symspell: SymSpell<UnicodeStringStrategy> = SymSpellBuilder::default()
            .max_dictionary_edit_distance(config.max_edit_distance)
            .prefix_length(7)
            .build()
            .expect("Failed to build SymSpell");

        // Add vocabulary words with high frequency
        // Format: "word,frequency" with comma separator
        for word in &vocabulary {
            // Use high frequency (1_000_000) for domain terms
            // Skip multi-word phrases for SymSpell (they need special handling)
            let word_lower = word.to_lowercase();
            if !word_lower.contains(' ') && !word_lower.contains('-') {
                let line = format!("{},1000000", word_lower);
                symspell.load_dictionary_line(&line, 0, 1, ",");
            }
        }

        // Build metaphone index
        let mut metaphone_index: HashMap<String, Vec<String>> = HashMap::new();
        for word in &vocabulary {
            let code = double_metaphone(&word.to_lowercase());
            metaphone_index
                .entry(code)
                .or_default()
                .push(word.clone());
        }

        // No hardcoded rules - use from_domain_config() for config-driven rules
        Self {
            symspell,
            confusion_rules: HashMap::new(),
            contextual_rules: HashMap::new(),
            phrase_rules: Vec::new(),
            metaphone_index,
            config,
        }
    }

    /// Create corrector from domain configuration
    ///
    /// This is the preferred way to create a PhoneticCorrector - config-driven.
    /// Pass in vocabulary terms and phonetic corrections from MasterDomainConfig.
    pub fn from_domain_config(
        vocabulary_terms: &[String],
        confusion_rules: std::collections::HashMap<String, String>,
        contextual_rules: Vec<(String, String, String)>, // (context, error, correction)
        phrase_rules: std::collections::HashMap<String, String>,
        config: PhoneticCorrectorConfig,
    ) -> Self {
        // Build SymSpell dictionary from vocabulary
        let mut symspell: SymSpell<UnicodeStringStrategy> = SymSpellBuilder::default()
            .max_dictionary_edit_distance(config.max_edit_distance)
            .prefix_length(7)
            .build()
            .expect("Failed to build SymSpell");

        // Add vocabulary words with high frequency
        for word in vocabulary_terms {
            let word_lower = word.to_lowercase();
            if !word_lower.contains(' ') && !word_lower.contains('-') {
                let line = format!("{},1000000", word_lower);
                symspell.load_dictionary_line(&line, 0, 1, ",");
            }
        }

        // Build metaphone index
        let mut metaphone_index: HashMap<String, Vec<String>> = HashMap::new();
        for word in vocabulary_terms {
            let code = double_metaphone(&word.to_lowercase());
            metaphone_index
                .entry(code)
                .or_default()
                .push(word.clone());
        }

        // Convert contextual rules to HashMap<(String, String), String>
        let mut contextual_rules_map = HashMap::new();
        for (context, error, correction) in contextual_rules {
            contextual_rules_map.insert((context.to_lowercase(), error.to_lowercase()), correction);
        }

        // Convert phrase_rules to Vec for ordered processing
        let phrase_rules_vec: Vec<(String, String)> = phrase_rules
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), v))
            .collect();

        Self {
            symspell,
            confusion_rules,
            contextual_rules: contextual_rules_map,
            phrase_rules: phrase_rules_vec,
            metaphone_index,
            config,
        }
    }

    /// Create an empty corrector with no domain-specific rules
    /// Used when no domain config is available
    pub fn empty() -> Self {
        Self::new(Vec::new(), PhoneticCorrectorConfig::default())
    }

    /// Apply phrase-level corrections to text
    fn apply_phrase_corrections(&self, text: &str) -> (String, Vec<Correction>) {
        let mut result = text.to_string();
        let mut corrections = Vec::new();
        let text_lower = text.to_lowercase();

        for (phrase, replacement) in &self.phrase_rules {
            if text_lower.contains(phrase) {
                // Find the position and preserve original case where possible
                if let Some(pos) = text_lower.find(phrase) {
                    let original = &text[pos..pos + phrase.len()];
                    // Replace in result (case-insensitive)
                    let result_lower = result.to_lowercase();
                    if let Some(result_pos) = result_lower.find(phrase) {
                        let before = &result[..result_pos];
                        let after = &result[result_pos + phrase.len()..];

                        // Preserve capitalization of first letter
                        let replacement_cased = if original.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                            capitalize_first(replacement)
                        } else {
                            replacement.clone()
                        };

                        result = format!("{}{}{}", before, replacement_cased, after);
                        corrections.push(Correction {
                            original: original.to_string(),
                            corrected: replacement_cased,
                            position: 0, // Phrase position
                            rule: "phrase".to_string(),
                        });
                    }
                }
            }
        }

        (result, corrections)
    }

    /// Correct a single word using confusion rules and SymSpell
    fn correct_word(&self, word: &str) -> Option<String> {
        let word_lower = word.to_lowercase();

        // Skip short words
        if word_lower.len() < self.config.min_word_length {
            return None;
        }

        // 1. Check direct confusion rules first (highest priority)
        if let Some(correction) = self.confusion_rules.get(&word_lower) {
            if correction.to_lowercase() != word_lower {
                return Some(correction.clone());
            }
        }

        // 2. Check SymSpell for fuzzy matches (conservative: distance <= 1)
        // We use max_edit_distance=2 for lookup but only accept distance <= 1 to avoid false positives
        let suggestions = self
            .symspell
            .lookup(&word_lower, Verbosity::Top, self.config.max_edit_distance);

        if let Some(suggestion) = suggestions.first() {
            // Only correct if edit distance is very small (1) to avoid "world" -> "gold" type errors
            if suggestion.term.to_lowercase() != word_lower && suggestion.distance <= 1 {
                // Preserve original case style
                let corrected = if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    capitalize_first(&suggestion.term)
                } else {
                    suggestion.term.clone()
                };
                return Some(corrected);
            }
        }

        // 3. Check metaphone similarity as fallback (very conservative)
        // Only use metaphone if the words are very similar (edit distance 1)
        // This prevents false positives like "world" -> "gold"
        let word_metaphone = double_metaphone(&word_lower);
        if let Some(matches) = self.metaphone_index.get(&word_metaphone) {
            for m in matches {
                let m_lower = m.to_lowercase();
                if m_lower != word_lower {
                    // Only accept metaphone match if edit distance is very small
                    let edit_dist = Self::levenshtein_distance(&word_lower, &m_lower);
                    if edit_dist <= 1 {
                        return Some(m.clone());
                    }
                }
            }
        }

        None
    }

    /// Calculate Levenshtein edit distance
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let len1 = s1_chars.len();
        let len2 = s2_chars.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut prev_row: Vec<usize> = (0..=len2).collect();
        let mut curr_row: Vec<usize> = vec![0; len2 + 1];

        for i in 1..=len1 {
            curr_row[0] = i;
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                curr_row[j] = std::cmp::min(
                    std::cmp::min(prev_row[j] + 1, curr_row[j - 1] + 1),
                    prev_row[j - 1] + cost,
                );
            }
            std::mem::swap(&mut prev_row, &mut curr_row);
        }

        prev_row[len2]
    }

    /// Correct text with contextual awareness
    pub fn correct(&self, text: &str) -> (String, Vec<Correction>) {
        let mut corrections = Vec::new();

        // 1. Apply phrase-level corrections first (highest priority)
        let (text_after_phrases, phrase_corrections) = self.apply_phrase_corrections(text);
        corrections.extend(phrase_corrections);

        // 2. Now apply word-level corrections
        let mut result_words: Vec<String> = Vec::new();

        // Tokenize into words while preserving punctuation
        let tokens: Vec<&str> = text_after_phrases.split_whitespace().collect();

        for (i, token) in tokens.iter().enumerate() {
            // Separate word from trailing punctuation
            let (word, punctuation) = split_punctuation(token);

            let word_lower = word.to_lowercase();

            // Handle sentence-start "Why" -> "I" correction
            if self.config.fix_sentence_start && i == 0 && word_lower == "why" {
                // Check if next word suggests this should be "I"
                if let Some(next) = tokens.get(1) {
                    let next_lower = next.to_lowercase();
                    if ["need", "want", "have", "am", "would", "could", "should", "will", "can"]
                        .contains(&next_lower.as_str())
                    {
                        corrections.push(Correction {
                            original: word.to_string(),
                            corrected: "I".to_string(),
                            position: i,
                            rule: "sentence_start_why_to_i".to_string(),
                        });
                        result_words.push(format!("I{}", punctuation));
                        continue;
                    }
                }
            }

            // Check contextual rules (look at previous word)
            if i > 0 {
                let prev_word = result_words
                    .last()
                    .map(|w| w.trim_end_matches(|c: char| c.is_ascii_punctuation()).to_lowercase())
                    .unwrap_or_default();

                if let Some(correction) = self
                    .contextual_rules
                    .get(&(prev_word.clone(), word_lower.clone()))
                {
                    corrections.push(Correction {
                        original: word.to_string(),
                        corrected: correction.clone(),
                        position: i,
                        rule: format!("contextual:{}+{}", prev_word, word_lower),
                    });
                    result_words.push(format!("{}{}", correction, punctuation));
                    continue;
                }
            }

            // Try single-word correction
            if let Some(corrected) = self.correct_word(word) {
                corrections.push(Correction {
                    original: word.to_string(),
                    corrected: corrected.clone(),
                    position: i,
                    rule: "phonetic".to_string(),
                });
                result_words.push(format!("{}{}", corrected, punctuation));
            } else {
                result_words.push(token.to_string());
            }
        }

        (result_words.join(" "), corrections)
    }

    /// Correct text, returning only the corrected string
    pub fn correct_text(&self, text: &str) -> String {
        self.correct(text).0
    }
}

/// Record of a correction made
#[derive(Debug, Clone)]
pub struct Correction {
    pub original: String,
    pub corrected: String,
    pub position: usize,
    pub rule: String,
}

/// Simple Double Metaphone implementation for phonetic encoding
/// Returns primary code only for simplicity
fn double_metaphone(word: &str) -> String {
    let word = word.to_lowercase();
    let chars: Vec<char> = word.chars().collect();

    if chars.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut i = 0;

    // Skip leading vowels
    if is_vowel(chars[0]) {
        result.push('A');
        i = 1;
    }

    while i < chars.len() && result.len() < 4 {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        let next2 = chars.get(i + 2).copied();

        match c {
            'b' => {
                result.push('P');
                i += if next == Some('b') { 2 } else { 1 };
            }
            'c' => {
                if next == Some('h') {
                    result.push('X'); // CH -> X
                    i += 2;
                } else if next == Some('i') || next == Some('e') || next == Some('y') {
                    result.push('S'); // soft C
                    i += 1;
                } else {
                    result.push('K'); // hard C
                    i += 1;
                }
            }
            'd' => {
                if next == Some('g')
                    && (next2 == Some('e') || next2 == Some('i') || next2 == Some('y'))
                {
                    result.push('J'); // DGE, DGI, DGY
                    i += 3;
                } else {
                    result.push('T');
                    i += 1;
                }
            }
            'f' | 'v' => {
                result.push('F');
                i += if next == Some(c) { 2 } else { 1 };
            }
            'g' => {
                if next == Some('h') {
                    i += 2; // GH is often silent
                } else if next == Some('n') && next2.is_none() {
                    i += 2; // GN at end is silent
                } else if next == Some('i') || next == Some('e') || next == Some('y') {
                    result.push('J'); // soft G
                    i += 1;
                } else {
                    result.push('K'); // hard G
                    i += 1;
                }
            }
            'h' => {
                // H is only pronounced if between vowels or at start before vowel
                if i == 0 || (i > 0 && is_vowel(chars[i - 1]) && next.map(is_vowel).unwrap_or(false))
                {
                    result.push('H');
                }
                i += 1;
            }
            'j' => {
                result.push('J');
                i += 1;
            }
            'k' => {
                result.push('K');
                i += if next == Some('k') { 2 } else { 1 };
            }
            'l' => {
                result.push('L');
                i += if next == Some('l') { 2 } else { 1 };
            }
            'm' => {
                result.push('M');
                i += if next == Some('m') { 2 } else { 1 };
            }
            'n' => {
                result.push('N');
                i += if next == Some('n') { 2 } else { 1 };
            }
            'p' => {
                if next == Some('h') {
                    result.push('F'); // PH -> F
                    i += 2;
                } else {
                    result.push('P');
                    i += if next == Some('p') { 2 } else { 1 };
                }
            }
            'q' => {
                result.push('K');
                i += 1;
            }
            'r' => {
                result.push('R');
                i += if next == Some('r') { 2 } else { 1 };
            }
            's' => {
                if next == Some('h') {
                    result.push('X'); // SH -> X
                    i += 2;
                } else {
                    result.push('S');
                    i += if next == Some('s') { 2 } else { 1 };
                }
            }
            't' => {
                if next == Some('h') {
                    result.push('0'); // TH -> 0 (theta)
                    i += 2;
                } else if next == Some('i') && next2 == Some('o') {
                    result.push('X'); // TIO -> X
                    i += 3;
                } else {
                    result.push('T');
                    i += if next == Some('t') { 2 } else { 1 };
                }
            }
            'w' => {
                if next.map(is_vowel).unwrap_or(false) {
                    result.push('W');
                }
                i += 1;
            }
            'x' => {
                result.push('K');
                result.push('S');
                i += 1;
            }
            'y' => {
                if next.map(is_vowel).unwrap_or(false) {
                    result.push('Y');
                }
                i += 1;
            }
            'z' => {
                result.push('S');
                i += if next == Some('z') { 2 } else { 1 };
            }
            _ => {
                // Skip vowels and other characters
                i += 1;
            }
        }
    }

    result
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// Split word from trailing punctuation
fn split_punctuation(token: &str) -> (&str, &str) {
    let punct_start = token
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_ascii_punctuation())
        .last()
        .map(|(i, _)| i)
        .unwrap_or(token.len());

    (&token[..punct_start], &token[punct_start..])
}

/// Capitalize first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test fixture corrector with sample rules
    fn test_fixture() -> PhoneticCorrector {
        let vocabulary = vec![
            "gold".to_string(),
            "loan".to_string(),
            "interest".to_string(),
            "rate".to_string(),
            "lakh".to_string(),
            "BrandX".to_string(),
        ];

        let mut confusion_rules = HashMap::new();
        confusion_rules.insert("gol".to_string(), "gold".to_string());
        confusion_rules.insert("lone".to_string(), "loan".to_string());
        confusion_rules.insert("intrest".to_string(), "interest".to_string());
        confusion_rules.insert("lac".to_string(), "lakh".to_string());
        confusion_rules.insert("brandex".to_string(), "BrandX".to_string());

        let contextual_rules = vec![
            ("gold".to_string(), "alone".to_string(), "loan".to_string()),
        ];

        let mut phrase_rules = HashMap::new();
        phrase_rules.insert("gold alone".to_string(), "gold loan".to_string());
        phrase_rules.insert("gold loon".to_string(), "gold loan".to_string());
        phrase_rules.insert("it's me about".to_string(), "tell me about".to_string());

        PhoneticCorrector::from_domain_config(
            &vocabulary,
            confusion_rules,
            contextual_rules,
            phrase_rules,
            PhoneticCorrectorConfig::default(),
        )
    }

    #[test]
    fn test_gold_alone_correction() {
        let corrector = test_fixture();
        let (corrected, corrections) = corrector.correct("gold alone");
        assert_eq!(corrected, "gold loan");
        assert!(!corrections.is_empty());
    }

    #[test]
    fn test_confusion_rule() {
        let corrector = test_fixture();
        let (corrected, _) = corrector.correct("brandex store");
        assert!(corrected.contains("BrandX"));
    }

    #[test]
    fn test_no_false_positives() {
        let corrector = test_fixture();

        // Words that should NOT be corrected
        let (corrected, corrections) = corrector.correct("hello world");
        assert_eq!(corrected, "hello world");
        assert!(corrections.is_empty());
    }

    #[test]
    fn test_interest_rate() {
        let corrector = test_fixture();
        let (corrected, _) = corrector.correct("what is the intrest rate");
        assert!(corrected.contains("interest"));
    }

    #[test]
    fn test_lakh_correction() {
        let corrector = test_fixture();
        let (corrected, _) = corrector.correct("I need 5 lac rupees");
        assert!(corrected.contains("lakh"));
    }

    #[test]
    fn test_double_metaphone() {
        // Test that similar sounding words have same/similar codes
        let code1 = double_metaphone("loan");
        let code2 = double_metaphone("lone");
        // Both should start with L and have similar structure
        assert!(code1.starts_with('L'));
        assert!(code2.starts_with('L'));
    }

    #[test]
    fn test_preserves_punctuation() {
        let corrector = test_fixture();
        let (corrected, _) = corrector.correct("gold alone?");
        assert!(corrected.ends_with('?'));
        assert!(corrected.contains("loan"));
    }

    #[test]
    fn test_phrase_correction() {
        let corrector = test_fixture();
        let (corrected, corrections) = corrector.correct("it's me about gold loan");
        assert!(corrected.to_lowercase().contains("tell me about"));
        assert!(!corrections.is_empty());
    }

    #[test]
    fn test_loon_to_loan() {
        let corrector = test_fixture();
        let (corrected, _) = corrector.correct("gold loon");
        assert!(corrected.to_lowercase().contains("gold loan"));
    }

    #[test]
    fn test_empty_corrector() {
        let corrector = PhoneticCorrector::empty();
        let (corrected, corrections) = corrector.correct("hello world");
        assert_eq!(corrected, "hello world");
        assert!(corrections.is_empty());
    }

    #[test]
    fn test_from_domain_config() {
        let vocabulary = vec!["custom".to_string()];
        let mut confusion_rules = HashMap::new();
        confusion_rules.insert("custum".to_string(), "custom".to_string());

        let corrector = PhoneticCorrector::from_domain_config(
            &vocabulary,
            confusion_rules,
            vec![],
            HashMap::new(),
            PhoneticCorrectorConfig::default(),
        );

        let (corrected, _) = corrector.correct("custum word");
        assert!(corrected.contains("custom"));
    }
}
