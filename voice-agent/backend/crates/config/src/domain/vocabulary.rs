//! Full Vocabulary Configuration
//!
//! P22 FIX: Loads the complete vocabulary configuration from vocabulary.yaml
//! This provides domain-specific terms, abbreviations, phonetic corrections,
//! and multilingual number words for speech recognition and text processing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Domain term with boost factor for speech recognition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainTerm {
    /// The term to recognize
    pub term: String,
    /// Boost factor for ASR (higher = more likely to recognize)
    #[serde(default = "default_boost")]
    pub boost: f64,
    /// Alternative spellings/aliases
    #[serde(default)]
    pub aliases: Vec<String>,
}

fn default_boost() -> f64 {
    1.0
}

/// Full vocabulary configuration loaded from vocabulary.yaml
///
/// This is separate from the simple VocabularyConfig in master.rs,
/// which contains inline vocabulary from domain.yaml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullVocabularyConfig {
    /// Brand/company abbreviations
    #[serde(default)]
    pub abbreviations: HashMap<String, String>,
    /// Competitor abbreviations (for recognition)
    #[serde(default)]
    pub competitor_abbreviations: HashMap<String, String>,
    /// Domain-specific terms for speech recognition boost
    #[serde(default)]
    pub domain_terms: Vec<DomainTerm>,
    /// Phonetic corrections for common ASR errors
    #[serde(default)]
    pub phonetic_corrections: HashMap<String, String>,
    /// Hindi number words mapping (word -> numeric value)
    #[serde(default)]
    pub hindi_numbers: HashMap<String, i64>,
}

impl Default for FullVocabularyConfig {
    fn default() -> Self {
        Self {
            abbreviations: HashMap::new(),
            competitor_abbreviations: HashMap::new(),
            domain_terms: Vec::new(),
            phonetic_corrections: HashMap::new(),
            hindi_numbers: HashMap::new(),
        }
    }
}

impl FullVocabularyConfig {
    /// Load from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, FullVocabularyConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            FullVocabularyConfigError::FileNotFound(
                path.as_ref().display().to_string(),
                e.to_string(),
            )
        })?;

        serde_yaml::from_str(&content)
            .map_err(|e| FullVocabularyConfigError::ParseError(e.to_string()))
    }

    /// Get all terms (main terms + aliases) for ASR boosting
    pub fn all_terms(&self) -> Vec<&str> {
        let mut terms = Vec::new();
        for dt in &self.domain_terms {
            terms.push(dt.term.as_str());
            terms.extend(dt.aliases.iter().map(|s| s.as_str()));
        }
        terms
    }

    /// Get terms with their boost factors
    pub fn terms_with_boost(&self) -> Vec<(&str, f64)> {
        let mut result = Vec::new();
        for dt in &self.domain_terms {
            result.push((dt.term.as_str(), dt.boost));
            for alias in &dt.aliases {
                result.push((alias.as_str(), dt.boost));
            }
        }
        result
    }

    /// Get term boost factor
    pub fn term_boost(&self, term: &str) -> f64 {
        let term_lower = term.to_lowercase();
        self.domain_terms
            .iter()
            .find(|dt| {
                dt.term.to_lowercase() == term_lower
                    || dt.aliases.iter().any(|a| a.to_lowercase() == term_lower)
            })
            .map(|dt| dt.boost)
            .unwrap_or(1.0)
    }

    /// Get all abbreviations (brand + competitor)
    pub fn all_abbreviations(&self) -> HashMap<String, String> {
        let mut all = self.abbreviations.clone();
        all.extend(self.competitor_abbreviations.clone());
        all
    }

    /// Expand an abbreviation to its full form
    pub fn expand_abbreviation(&self, abbrev: &str) -> Option<&str> {
        self.abbreviations
            .get(abbrev)
            .or_else(|| self.competitor_abbreviations.get(abbrev))
            .map(|s| s.as_str())
    }

    /// Convert Hindi number word to value
    pub fn hindi_to_number(&self, word: &str) -> Option<i64> {
        self.hindi_numbers.get(word).copied()
    }

    /// Get phonetic correction for a word
    pub fn phonetic_correction(&self, word: &str) -> Option<&str> {
        self.phonetic_corrections.get(word).map(|s| s.as_str())
    }

    /// Apply phonetic corrections to text
    pub fn apply_phonetic_corrections(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (error, correction) in &self.phonetic_corrections {
            // Case-insensitive replacement
            let re = regex::RegexBuilder::new(&regex::escape(error))
                .case_insensitive(true)
                .build();
            if let Ok(re) = re {
                result = re.replace_all(&result, correction.as_str()).to_string();
            }
        }
        result
    }

    /// Get all phonetic corrections
    pub fn phonetic_corrections_list(&self) -> &HashMap<String, String> {
        &self.phonetic_corrections
    }

    /// Check if a term exists in domain vocabulary
    pub fn has_term(&self, term: &str) -> bool {
        let term_lower = term.to_lowercase();
        self.domain_terms.iter().any(|dt| {
            dt.term.to_lowercase() == term_lower
                || dt.aliases.iter().any(|a| a.to_lowercase() == term_lower)
        })
    }
}

/// Errors when loading vocabulary configuration
#[derive(Debug)]
pub enum FullVocabularyConfigError {
    FileNotFound(String, String),
    ParseError(String),
}

impl std::fmt::Display for FullVocabularyConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path, err) => {
                write!(f, "Vocabulary config not found at {}: {}", path, err)
            }
            Self::ParseError(err) => write!(f, "Failed to parse vocabulary config: {}", err),
        }
    }
}

impl std::error::Error for FullVocabularyConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vocabulary_config_deserialization() {
        let yaml = r#"
abbreviations:
  KMBL: "Kotak Mahindra Bank Limited"
  KMB: "Kotak Mahindra Bank"

competitor_abbreviations:
  MFL: "Muthoot Finance Limited"

domain_terms:
  - term: "gold"
    boost: 2.0
    aliases: ["sona", "swarna"]
  - term: "loan"
    boost: 2.0
    aliases: ["karz", "rin"]

phonetic_corrections:
  "gol lone": "gold loan"
  "kotuk": "Kotak"

hindi_numbers:
  ek: 1
  do: 2
  lakh: 100000
"#;
        let config: FullVocabularyConfig = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(config.abbreviations.len(), 2);
        assert_eq!(config.competitor_abbreviations.len(), 1);
        assert_eq!(config.domain_terms.len(), 2);
        assert_eq!(config.phonetic_corrections.len(), 2);
        assert_eq!(config.hindi_numbers.len(), 3);

        // Test term boost
        assert_eq!(config.term_boost("gold"), 2.0);
        assert_eq!(config.term_boost("sona"), 2.0); // alias
        assert_eq!(config.term_boost("unknown"), 1.0); // default

        // Test abbreviation expansion
        assert_eq!(
            config.expand_abbreviation("KMBL"),
            Some("Kotak Mahindra Bank Limited")
        );
        assert_eq!(
            config.expand_abbreviation("MFL"),
            Some("Muthoot Finance Limited")
        );

        // Test Hindi numbers
        assert_eq!(config.hindi_to_number("lakh"), Some(100000));
        assert_eq!(config.hindi_to_number("unknown"), None);

        // Test phonetic correction
        assert_eq!(config.phonetic_correction("gol lone"), Some("gold loan"));
    }

    #[test]
    fn test_all_terms() {
        let config = FullVocabularyConfig {
            domain_terms: vec![
                DomainTerm {
                    term: "gold".to_string(),
                    boost: 2.0,
                    aliases: vec!["sona".to_string()],
                },
                DomainTerm {
                    term: "loan".to_string(),
                    boost: 1.5,
                    aliases: vec![],
                },
            ],
            ..Default::default()
        };

        let terms = config.all_terms();
        assert!(terms.contains(&"gold"));
        assert!(terms.contains(&"sona"));
        assert!(terms.contains(&"loan"));
        assert_eq!(terms.len(), 3);
    }
}
