//! Domain context for text processing
//!
//! Provides domain-specific vocabulary and context for grammar correction,
//! translation, and other text processing tasks.

use serde::{Deserialize, Serialize};

/// Domain-specific context for grammar correction and processing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DomainContext {
    /// Domain name (e.g., "gold_loan")
    pub domain: String,
    /// Domain-specific vocabulary that should be preserved
    pub vocabulary: Vec<String>,
    /// Common phrases in this domain
    pub phrases: Vec<String>,
    /// Entity types to preserve (names, numbers, etc.)
    pub preserve_entities: Vec<String>,
    /// Abbreviations and their expansions
    #[serde(default)]
    pub abbreviations: Vec<Abbreviation>,
    /// Competitor names to handle specially
    #[serde(default)]
    pub competitors: Vec<String>,
}

/// Abbreviation with expansion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Abbreviation {
    pub short: String,
    pub full: String,
}

impl DomainContext {
    /// Create a new empty domain context
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            ..Default::default()
        }
    }

    /// Create DomainContext from config values
    ///
    /// This is the preferred way to create a DomainContext - all values
    /// come from config files rather than hardcoded defaults.
    ///
    /// # Arguments
    /// * `domain` - Domain identifier (e.g., "gold_loan")
    /// * `vocabulary` - Domain-specific terms to preserve
    /// * `phrases` - Common phrases in this domain
    /// * `abbreviations` - Abbreviation mappings as (short, full) tuples
    /// * `preserve_entities` - Entity types to preserve
    /// * `competitors` - Competitor names for detection
    pub fn from_config(
        domain: &str,
        vocabulary: Vec<String>,
        phrases: Vec<String>,
        abbreviations: Vec<(String, String)>,
        preserve_entities: Vec<String>,
        competitors: Vec<String>,
    ) -> Self {
        Self {
            domain: domain.to_string(),
            vocabulary,
            phrases,
            preserve_entities,
            abbreviations: abbreviations
                .into_iter()
                .map(|(short, full)| Abbreviation { short, full })
                .collect(),
            competitors,
        }
    }

    /// Create a test fixture context (for tests only)
    ///
    /// Production code should use `from_config()` with config-driven values.
    #[cfg(test)]
    pub(crate) fn test_fixture() -> Self {
        Self {
            domain: "test_domain".to_string(),
            vocabulary: vec![
                "term1".to_string(),
                "term2".to_string(),
                "LTV".to_string(),
            ],
            phrases: vec!["test phrase".to_string()],
            preserve_entities: vec!["PersonName".to_string()],
            abbreviations: vec![Abbreviation {
                short: "LTV".to_string(),
                full: "Loan to Value".to_string(),
            }],
            competitors: vec!["Competitor A".to_string(), "Competitor B".to_string()],
        }
    }

    /// Add vocabulary term
    pub fn add_vocabulary(mut self, term: impl Into<String>) -> Self {
        self.vocabulary.push(term.into());
        self
    }

    /// Add phrase
    pub fn add_phrase(mut self, phrase: impl Into<String>) -> Self {
        self.phrases.push(phrase.into());
        self
    }

    /// Add competitor
    pub fn add_competitor(mut self, competitor: impl Into<String>) -> Self {
        self.competitors.push(competitor.into());
        self
    }

    /// Check if a word is domain vocabulary
    pub fn is_vocabulary(&self, word: &str) -> bool {
        let word_lower = word.to_lowercase();
        self.vocabulary
            .iter()
            .any(|v| v.to_lowercase() == word_lower)
    }

    /// Check if text contains a competitor name
    pub fn contains_competitor(&self, text: &str) -> Option<&str> {
        let text_lower = text.to_lowercase();
        self.competitors
            .iter()
            .find(|c| text_lower.contains(&c.to_lowercase()))
            .map(|s| s.as_str())
    }

    /// Get expansion for abbreviation
    pub fn expand_abbreviation(&self, abbrev: &str) -> Option<&str> {
        self.abbreviations
            .iter()
            .find(|a| a.short.eq_ignore_ascii_case(abbrev))
            .map(|a| a.full.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_context_fixture() {
        let ctx = DomainContext::test_fixture();
        assert_eq!(ctx.domain, "test_domain");
        assert!(ctx.is_vocabulary("term1"));
        assert!(ctx.is_vocabulary("LTV"));
    }

    #[test]
    fn test_competitor_detection() {
        let ctx = DomainContext::test_fixture();
        let competitor = ctx.contains_competitor("I talked to Competitor A");
        assert!(competitor.is_some());
        assert!(competitor.unwrap().contains("Competitor A"));
        assert!(ctx.contains_competitor("no competitor here").is_none());
    }

    #[test]
    fn test_abbreviation_expansion() {
        let ctx = DomainContext::test_fixture();
        assert_eq!(ctx.expand_abbreviation("LTV"), Some("Loan to Value"));
        assert_eq!(ctx.expand_abbreviation("ltv"), Some("Loan to Value"));
        assert!(ctx.expand_abbreviation("XYZ").is_none());
    }

    #[test]
    fn test_builder_pattern() {
        let ctx = DomainContext::new("insurance")
            .add_vocabulary("premium")
            .add_vocabulary("coverage")
            .add_competitor("LIC");

        assert_eq!(ctx.domain, "insurance");
        assert!(ctx.is_vocabulary("premium"));
        assert!(ctx.contains_competitor("LIC policy").is_some());
    }

    #[test]
    fn test_from_config() {
        let ctx = DomainContext::from_config(
            "test",
            vec!["vocab1".to_string()],
            vec!["phrase1".to_string()],
            vec![("ABC".to_string(), "Always Be Closing".to_string())],
            vec!["Entity".to_string()],
            vec!["CompX".to_string()],
        );
        assert_eq!(ctx.domain, "test");
        assert!(ctx.is_vocabulary("vocab1"));
        assert_eq!(ctx.expand_abbreviation("ABC"), Some("Always Be Closing"));
        assert!(ctx.contains_competitor("CompX is good").is_some());
    }
}
