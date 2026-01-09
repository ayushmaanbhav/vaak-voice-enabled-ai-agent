//! Edit distance-based word correction for domain vocabulary
//!
//! Uses Levenshtein distance to correct ASR errors to domain-specific terms.
//! This runs as a pre-processing step before LLM grammar correction.

use std::collections::HashMap;

/// Configuration for edit distance correction
#[derive(Debug, Clone)]
pub struct EditDistanceConfig {
    /// Maximum edit distance to consider a match
    pub max_distance: usize,
    /// Minimum word length to attempt correction
    pub min_word_length: usize,
    /// Whether to preserve case of original word
    pub preserve_case: bool,
}

impl Default for EditDistanceConfig {
    fn default() -> Self {
        Self {
            max_distance: 2,
            min_word_length: 3,
            preserve_case: false,
        }
    }
}

/// Edit distance based corrector for domain vocabulary
#[derive(Clone)]
pub struct EditDistanceCorrector {
    /// Domain vocabulary (lowercase -> original)
    vocabulary: HashMap<String, String>,
    /// Common ASR phonetic substitutions (misspelling -> correct)
    phonetic_map: HashMap<String, String>,
    config: EditDistanceConfig,
}

impl EditDistanceCorrector {
    /// Create a new empty corrector
    ///
    /// NOTE: This creates an empty corrector with no vocabulary or phonetic rules.
    /// Use `from_domain_config()` for production use with config-driven data.
    pub fn new(config: EditDistanceConfig) -> Self {
        Self {
            vocabulary: HashMap::new(),
            phonetic_map: HashMap::new(),
            config,
        }
    }

    /// Create corrector from domain configuration
    ///
    /// This is the preferred way to create an EditDistanceCorrector - all values
    /// come from config files rather than hardcoded defaults.
    ///
    /// # Arguments
    /// * `vocabulary` - Domain-specific terms to correct towards
    /// * `phonetic_map` - ASR error corrections (misspelling -> correct)
    /// * `config` - Configuration settings
    pub fn from_domain_config(
        vocabulary: Vec<String>,
        phonetic_map: HashMap<String, String>,
        config: EditDistanceConfig,
    ) -> Self {
        let mut vocab_map = HashMap::new();
        for term in vocabulary {
            // Store as lowercase for matching, keep original for output
            vocab_map.insert(term.to_lowercase(), term);
        }

        Self {
            vocabulary: vocab_map,
            phonetic_map,
            config,
        }
    }

    /// Add vocabulary terms dynamically
    pub fn add_vocabulary(&mut self, terms: Vec<String>) {
        for term in terms {
            self.vocabulary.insert(term.to_lowercase(), term);
        }
    }

    /// Add phonetic corrections dynamically
    pub fn add_phonetic_rules(&mut self, rules: HashMap<String, String>) {
        self.phonetic_map.extend(rules);
    }

    /// Calculate Levenshtein edit distance between two strings
    pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        let len1 = s1_chars.len();
        let len2 = s2_chars.len();

        // Early exit for empty strings
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        // Use two rows instead of full matrix for space efficiency
        let mut prev_row: Vec<usize> = (0..=len2).collect();
        let mut curr_row: Vec<usize> = vec![0; len2 + 1];

        for i in 1..=len1 {
            curr_row[0] = i;
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1].to_lowercase().next()
                    == s2_chars[j - 1].to_lowercase().next()
                {
                    0
                } else {
                    1
                };

                curr_row[j] = std::cmp::min(
                    std::cmp::min(
                        prev_row[j] + 1,     // deletion
                        curr_row[j - 1] + 1, // insertion
                    ),
                    prev_row[j - 1] + cost, // substitution
                );
            }
            std::mem::swap(&mut prev_row, &mut curr_row);
        }

        prev_row[len2]
    }

    /// Find closest match in vocabulary for a word
    pub fn find_closest_match(&self, word: &str) -> Option<(String, usize)> {
        if word.len() < self.config.min_word_length {
            return None;
        }

        let word_lower = word.to_lowercase();

        // First check phonetic map for exact matches
        if let Some(correct) = self.phonetic_map.get(&word_lower) {
            return Some((correct.clone(), 0));
        }

        // Find best match in vocabulary
        let mut best_match: Option<(String, usize)> = None;

        for (vocab_lower, vocab_original) in &self.vocabulary {
            // Skip if lengths are too different
            let len_diff = (word.len() as isize - vocab_lower.len() as isize).unsigned_abs();
            if len_diff > self.config.max_distance {
                continue;
            }

            let distance = Self::levenshtein_distance(&word_lower, vocab_lower);

            // Also check similarity ratio - distance should be less than half the word length
            // to avoid false positives like "world" -> "gold"
            let max_allowed = (word.len().max(vocab_lower.len()) / 2).max(1);
            let effective_max = self.config.max_distance.min(max_allowed);

            if distance <= effective_max {
                match &best_match {
                    None => {
                        best_match = Some((vocab_original.clone(), distance));
                    }
                    Some((_, best_dist)) if distance < *best_dist => {
                        best_match = Some((vocab_original.clone(), distance));
                    }
                    _ => {}
                }
            }
        }

        best_match
    }

    /// Correct text using edit distance matching
    ///
    /// Returns the corrected text and a list of corrections made.
    pub fn correct(&self, text: &str) -> (String, Vec<Correction>) {
        let mut corrections = Vec::new();
        let mut result = String::new();
        let mut last_end = 0;

        // Simple word tokenization
        let word_chars: Vec<char> = text.chars().collect();
        let mut i = 0;

        while i < word_chars.len() {
            // Skip non-alphanumeric
            if !word_chars[i].is_alphanumeric() {
                result.push(word_chars[i]);
                i += 1;
                last_end = i;
                continue;
            }

            // Collect word
            let word_start = i;
            while i < word_chars.len() && word_chars[i].is_alphanumeric() {
                i += 1;
            }
            let word: String = word_chars[word_start..i].iter().collect();

            // Special case: "Why" at sentence start -> "I"
            let is_sentence_start =
                word_start == 0 || (word_start > 0 && result.trim_end().ends_with('.'));
            if is_sentence_start && word.to_lowercase() == "why" {
                // Check if next word suggests this should be "I" (e.g., "Why need" -> "I need")
                let rest: String = word_chars[i..].iter().collect();
                let next_word = rest.trim_start().split_whitespace().next().unwrap_or("");
                if next_word.to_lowercase() == "need"
                    || next_word.to_lowercase() == "want"
                    || next_word.to_lowercase() == "have"
                    || next_word.to_lowercase() == "am"
                {
                    corrections.push(Correction {
                        original: word.clone(),
                        corrected: "I".to_string(),
                        distance: 0,
                        reason: "Sentence start 'Why' -> 'I' before verb".to_string(),
                    });
                    result.push('I');
                    continue;
                }
            }

            // Check for corrections
            if let Some((corrected, distance)) = self.find_closest_match(&word) {
                // Only correct if distance > 0 (actual change)
                if distance > 0 || corrected.to_lowercase() != word.to_lowercase() {
                    corrections.push(Correction {
                        original: word.clone(),
                        corrected: corrected.clone(),
                        distance,
                        reason: format!("Edit distance {} from '{}'", distance, word),
                    });
                    result.push_str(&corrected);
                } else {
                    result.push_str(&word);
                }
            } else {
                result.push_str(&word);
            }
        }

        (result, corrections)
    }

    /// Correct text, only returning the corrected string
    pub fn correct_text(&self, text: &str) -> String {
        self.correct(text).0
    }
}

/// Record of a correction made
#[derive(Debug, Clone)]
pub struct Correction {
    pub original: String,
    pub corrected: String,
    pub distance: usize,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test fixture corrector with sample data
    fn test_fixture() -> EditDistanceCorrector {
        let vocabulary = vec![
            "gold".to_string(),
            "loan".to_string(),
            "gold loan".to_string(),
            "BrandX".to_string(),
            "BrandX Bank".to_string(),
            "interest".to_string(),
            "EMI".to_string(),
        ];

        let mut phonetic_map = HashMap::new();
        // Common ASR errors
        phonetic_map.insert("alone".to_string(), "loan".to_string());
        phonetic_map.insert("along".to_string(), "loan".to_string());
        phonetic_map.insert("lone".to_string(), "loan".to_string());
        phonetic_map.insert("gol".to_string(), "gold".to_string());
        // Brand name corrections
        phonetic_map.insert("brandex".to_string(), "BrandX".to_string());
        phonetic_map.insert("brandix".to_string(), "BrandX".to_string());
        // Sentence start correction
        phonetic_map.insert("why".to_string(), "I".to_string());

        EditDistanceCorrector::from_domain_config(
            vocabulary,
            phonetic_map,
            EditDistanceConfig::default(),
        )
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(EditDistanceCorrector::levenshtein_distance("", ""), 0);
        assert_eq!(EditDistanceCorrector::levenshtein_distance("abc", "abc"), 0);
        assert_eq!(EditDistanceCorrector::levenshtein_distance("abc", ""), 3);
        assert_eq!(EditDistanceCorrector::levenshtein_distance("", "abc"), 3);
        // kitten -> sitting = 3 (substitute k→s, e→i, add g)
        assert_eq!(EditDistanceCorrector::levenshtein_distance("kitten", "sitting"), 3);
        // lone -> loan: substitute n→a, e→n = 2
        assert_eq!(
            EditDistanceCorrector::levenshtein_distance("lone", "loan"),
            2
        );
        // gol -> gold: insert 'd' = 1
        assert_eq!(
            EditDistanceCorrector::levenshtein_distance("gol", "gold"),
            1
        );
    }

    #[test]
    fn test_phonetic_map() {
        let corrector = test_fixture();

        // Test phonetic map matches
        let result = corrector.find_closest_match("alone");
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "loan");

        let result = corrector.find_closest_match("brandex");
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "BrandX");
    }

    #[test]
    fn test_alone_correction() {
        let corrector = test_fixture();

        // "gold alone" should become "gold loan"
        let (corrected, corrections) = corrector.correct("gold alone");
        assert_eq!(corrected, "gold loan");
        assert!(!corrections.is_empty());
    }

    #[test]
    fn test_sentence_correction() {
        let corrector = test_fixture();

        // "Why need help regarding gold alone" -> "I need help regarding gold loan"
        let (corrected, corrections) = corrector.correct("Why need help regarding gold alone");
        assert!(corrected.contains("I need"));
        assert!(corrected.contains("gold loan"));
        assert!(corrections.len() >= 2);
    }

    #[test]
    fn test_brand_correction() {
        let corrector = test_fixture();

        let (corrected, _) = corrector.correct("brandex bank gold lone");
        assert!(corrected.contains("BrandX"));
        assert!(corrected.contains("loan"));
    }

    #[test]
    fn test_edit_distance_match() {
        let corrector = test_fixture();

        // "lone" should correct to "loan" (distance 0 because it's in phonetic map)
        let result = corrector.find_closest_match("lone");
        assert!(result.is_some());
        let (word, dist) = result.unwrap();
        assert_eq!(word, "loan");
        assert_eq!(dist, 0); // 0 because it's in phonetic map
    }

    #[test]
    fn test_no_correction_needed() {
        let corrector = test_fixture();

        // Words that should not be corrected (far from any vocabulary)
        let (corrected, corrections) = corrector.correct("please check tomorrow");
        assert_eq!(corrected, "please check tomorrow");
        assert!(corrections.is_empty());
    }

    #[test]
    fn test_empty_corrector() {
        let corrector = EditDistanceCorrector::new(EditDistanceConfig::default());
        let (corrected, corrections) = corrector.correct("some text");
        assert_eq!(corrected, "some text");
        assert!(corrections.is_empty());
    }

    #[test]
    fn test_add_vocabulary_dynamically() {
        let mut corrector = EditDistanceCorrector::new(EditDistanceConfig::default());
        corrector.add_vocabulary(vec!["custom".to_string(), "term".to_string()]);

        // Should now match "custom" in vocabulary
        let result = corrector.find_closest_match("custm");
        assert!(result.is_some());
    }

    #[test]
    fn test_add_phonetic_rules_dynamically() {
        let mut corrector = EditDistanceCorrector::new(EditDistanceConfig::default());
        let mut rules = HashMap::new();
        rules.insert("typo".to_string(), "type".to_string());
        corrector.add_phonetic_rules(rules);

        let result = corrector.find_closest_match("typo");
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "type");
    }
}
