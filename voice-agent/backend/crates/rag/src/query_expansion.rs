//! Query Expansion for improved retrieval
//!
//! Expands queries with:
//! - Domain-specific synonyms
//! - Hindi/Hinglish transliterations
//! - Related terms and concepts
//!
//! All domain data is loaded from configuration files.

use parking_lot::RwLock;
use std::collections::HashMap;

/// Query expansion configuration
#[derive(Debug, Clone)]
pub struct QueryExpansionConfig {
    /// Enable synonym expansion
    pub enable_synonyms: bool,
    /// Enable transliteration expansion (Hindi <-> Roman)
    pub enable_transliteration: bool,
    /// Maximum expansion terms per query term
    pub max_expansions_per_term: usize,
    /// Boost factor for original terms vs expansions
    pub original_term_boost: f32,
    /// Domain for term expansion
    pub domain: String,
}

impl Default for QueryExpansionConfig {
    fn default() -> Self {
        Self {
            enable_synonyms: true,
            enable_transliteration: true,
            max_expansions_per_term: 3,
            original_term_boost: 2.0,
            domain: String::new(), // No default domain - must be config-driven
        }
    }
}

/// Expanded query result
#[derive(Debug, Clone)]
pub struct ExpandedQuery {
    /// Original query
    pub original: String,
    /// Expanded query terms with weights
    pub terms: Vec<WeightedTerm>,
    /// Whether any expansion occurred
    pub was_expanded: bool,
    /// Expansion statistics
    pub stats: ExpansionStats,
}

/// A weighted query term
#[derive(Debug, Clone)]
pub struct WeightedTerm {
    /// The term
    pub term: String,
    /// Weight/boost factor
    pub weight: f32,
    /// Source of this term
    pub source: TermSource,
}

/// Source of an expanded term
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermSource {
    /// Original query term
    Original,
    /// Synonym expansion
    Synonym,
    /// Transliteration
    Transliteration,
    /// Domain-specific expansion
    Domain,
}

/// Expansion statistics
#[derive(Debug, Clone, Default)]
pub struct ExpansionStats {
    /// Original term count
    pub original_terms: usize,
    /// Synonym expansions added
    pub synonym_expansions: usize,
    /// Transliteration expansions added
    pub transliteration_expansions: usize,
    /// Domain expansions added
    pub domain_expansions: usize,
}

/// Query expander for RAG
pub struct QueryExpander {
    config: QueryExpansionConfig,
    /// Domain synonym dictionary
    synonyms: RwLock<HashMap<String, Vec<String>>>,
    /// Transliteration mappings (Hindi <-> Roman)
    transliterations: RwLock<HashMap<String, Vec<String>>>,
    /// Domain-specific term expansions
    domain_terms: RwLock<HashMap<String, Vec<String>>>,
    /// Stopwords (common words to filter out)
    stopwords: RwLock<std::collections::HashSet<String>>,
}

impl QueryExpander {
    /// Create a new empty query expander
    ///
    /// NOTE: This creates an empty expander with no dictionaries loaded.
    /// Use `from_domain_config()` for production use with config-driven data.
    pub fn new(config: QueryExpansionConfig) -> Self {
        Self {
            config,
            synonyms: RwLock::new(HashMap::new()),
            transliterations: RwLock::new(HashMap::new()),
            domain_terms: RwLock::new(HashMap::new()),
            stopwords: RwLock::new(std::collections::HashSet::new()),
        }
    }

    /// Create from domain configuration
    ///
    /// This is the preferred way to create a QueryExpander - all values
    /// come from config files rather than hardcoded defaults.
    ///
    /// # Arguments
    /// * `domain` - Domain identifier (from DOMAIN_ID env var)
    /// * `stopwords` - List of stopwords for the domain
    /// * `synonyms` - Synonym mappings (term -> alternatives)
    /// * `transliterations` - Transliteration mappings (Hindi <-> Roman)
    pub fn from_domain_config(
        domain: &str,
        stopwords: Vec<String>,
        synonyms: HashMap<String, Vec<String>>,
        transliterations: HashMap<String, Vec<String>>,
    ) -> Self {
        let config = QueryExpansionConfig {
            domain: domain.to_string(),
            ..Default::default()
        };

        Self {
            config,
            synonyms: RwLock::new(synonyms),
            transliterations: RwLock::new(transliterations),
            domain_terms: RwLock::new(HashMap::new()),
            stopwords: RwLock::new(stopwords.into_iter().collect()),
        }
    }

    /// Create from domain configuration with full config control
    ///
    /// This variant allows specifying all expansion options.
    pub fn from_full_config(
        domain: &str,
        stopwords: Vec<String>,
        synonyms: HashMap<String, Vec<String>>,
        transliterations: HashMap<String, Vec<String>>,
        domain_terms: HashMap<String, Vec<String>>,
        enable_synonyms: bool,
        enable_transliteration: bool,
        max_expansions_per_term: usize,
        original_term_boost: f32,
    ) -> Self {
        let config = QueryExpansionConfig {
            domain: domain.to_string(),
            enable_synonyms,
            enable_transliteration,
            max_expansions_per_term,
            original_term_boost,
        };

        Self {
            config,
            synonyms: RwLock::new(synonyms),
            transliterations: RwLock::new(transliterations),
            domain_terms: RwLock::new(domain_terms),
            stopwords: RwLock::new(stopwords.into_iter().collect()),
        }
    }

    /// Check if a word is a stopword
    pub fn is_stopword(&self, word: &str) -> bool {
        self.stopwords.read().contains(&word.to_lowercase())
    }

    /// Filter stopwords from a query
    pub fn filter_stopwords(&self, query: &str) -> String {
        let stopwords = self.stopwords.read();
        query
            .split_whitespace()
            .filter(|word| !stopwords.contains(&word.to_lowercase()))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Add custom stopwords
    pub fn add_stopwords(&self, words: &[&str]) {
        let mut stopwords = self.stopwords.write();
        for word in words {
            stopwords.insert(word.to_lowercase());
        }
    }

    /// Expand a query
    pub fn expand(&self, query: &str) -> ExpandedQuery {
        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut terms = Vec::new();
        let mut stats = ExpansionStats::default();

        // Add original terms with boost
        for word in &words {
            terms.push(WeightedTerm {
                term: word.to_string(),
                weight: self.config.original_term_boost,
                source: TermSource::Original,
            });
            stats.original_terms += 1;
        }

        // Synonym expansion
        if self.config.enable_synonyms {
            let synonyms = self.synonyms.read();
            for word in &words {
                if let Some(syns) = synonyms.get(*word) {
                    for syn in syns.iter().take(self.config.max_expansions_per_term) {
                        if !terms.iter().any(|t| t.term == *syn) {
                            terms.push(WeightedTerm {
                                term: syn.clone(),
                                weight: 1.0,
                                source: TermSource::Synonym,
                            });
                            stats.synonym_expansions += 1;
                        }
                    }
                }
            }
        }

        // Transliteration expansion
        if self.config.enable_transliteration {
            let trans = self.transliterations.read();
            for word in &words {
                if let Some(translits) = trans.get(*word) {
                    for t in translits.iter().take(self.config.max_expansions_per_term) {
                        if !terms.iter().any(|term| term.term == *t) {
                            terms.push(WeightedTerm {
                                term: t.clone(),
                                weight: 0.8,
                                source: TermSource::Transliteration,
                            });
                            stats.transliteration_expansions += 1;
                        }
                    }
                }
            }
        }

        // Domain-specific multi-word expansion
        let domain_terms = self.domain_terms.read();
        for (pattern, expansions) in domain_terms.iter() {
            if query_lower.contains(pattern) {
                for exp in expansions.iter().take(self.config.max_expansions_per_term) {
                    if !terms.iter().any(|t| t.term == *exp) {
                        terms.push(WeightedTerm {
                            term: exp.clone(),
                            weight: 0.9,
                            source: TermSource::Domain,
                        });
                        stats.domain_expansions += 1;
                    }
                }
            }
        }

        let was_expanded = stats.synonym_expansions > 0
            || stats.transliteration_expansions > 0
            || stats.domain_expansions > 0;

        ExpandedQuery {
            original: query.to_string(),
            terms,
            was_expanded,
            stats,
        }
    }

    /// Get expanded query as a weighted search string
    ///
    /// Format: "term1^2.0 term2^1.0 term3^0.8"
    pub fn expand_to_string(&self, query: &str) -> String {
        let expanded = self.expand(query);

        expanded
            .terms
            .iter()
            .map(|t| {
                if (t.weight - 1.0).abs() < 0.01 {
                    t.term.clone()
                } else {
                    format!("{}^{:.1}", t.term, t.weight)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Add a custom synonym
    pub fn add_synonym(&self, term: &str, synonyms: &[&str]) {
        let mut syn_map = self.synonyms.write();
        syn_map.insert(
            term.to_lowercase(),
            synonyms.iter().map(|s| s.to_lowercase()).collect(),
        );
    }

    /// Add a custom transliteration
    pub fn add_transliteration(&self, term: &str, transliterations: &[&str]) {
        let mut trans_map = self.transliterations.write();
        trans_map.insert(
            term.to_string(),
            transliterations.iter().map(|s| s.to_string()).collect(),
        );
    }

    /// Add a domain term expansion
    pub fn add_domain_term(&self, term: &str, expansions: &[&str]) {
        let mut domain_map = self.domain_terms.write();
        domain_map.insert(
            term.to_lowercase(),
            expansions.iter().map(|s| s.to_string()).collect(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test fixture expander with sample data
    fn test_fixture() -> QueryExpander {
        let synonyms = vec![
            ("gold", vec!["sona", "swarna"]),
            ("loan", vec!["karza", "rin"]),
            ("interest", vec!["byaj", "rate"]),
            ("eligibility", vec!["patrta", "qualification"]),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
        .collect();

        let transliterations = vec![
            ("sona", vec!["सोना", "gold"]),
            ("byaj", vec!["ब्याज", "interest"]),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.iter().map(|s| s.to_string()).collect()))
        .collect();

        let stopwords = vec!["the", "a", "is", "hai", "ka", "ki"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let mut expander = QueryExpander::from_domain_config(
            "test_domain",
            stopwords,
            synonyms,
            transliterations,
        );

        // Add some domain terms for testing
        expander.add_domain_term("kya hai", &["what is", "क्या है"]);
        expander.add_domain_term("interest rate", &["byaj dar", "rate of interest"]);

        expander
    }

    #[test]
    fn test_basic_expansion() {
        let expander = test_fixture();
        let expanded = expander.expand("gold loan interest rate");

        assert!(expanded.was_expanded);
        assert!(expanded.terms.len() > 4); // More than just original terms

        // Should contain original terms
        assert!(expanded.terms.iter().any(|t| t.term == "gold"));
        assert!(expanded.terms.iter().any(|t| t.term == "loan"));
        assert!(expanded.terms.iter().any(|t| t.term == "interest"));
        assert!(expanded.terms.iter().any(|t| t.term == "rate"));
    }

    #[test]
    fn test_synonym_expansion() {
        let expander = test_fixture();
        let expanded = expander.expand("gold eligibility");

        // Should expand "gold" to include "sona"
        assert!(expanded
            .terms
            .iter()
            .any(|t| t.term == "sona" && t.source == TermSource::Synonym));
    }

    #[test]
    fn test_transliteration_expansion() {
        let expander = test_fixture();
        let expanded = expander.expand("sona loan");

        // "sona" should be transliterated
        assert!(expanded
            .terms
            .iter()
            .any(|t| t.source == TermSource::Transliteration));
    }

    #[test]
    fn test_domain_expansion() {
        let expander = test_fixture();
        let expanded = expander.expand("interest rate kya hai");

        // "kya hai" should trigger domain expansion
        assert!(expanded.stats.domain_expansions > 0);
    }

    #[test]
    fn test_expand_to_string() {
        let expander = test_fixture();
        let query_string = expander.expand_to_string("gold loan");

        // Should contain boosted original terms
        assert!(query_string.contains("gold^2.0"));
        assert!(query_string.contains("loan^2.0"));
    }

    #[test]
    fn test_custom_synonym() {
        let expander = test_fixture();
        expander.add_synonym("test", &["custom1", "custom2"]);

        let expanded = expander.expand("test query");
        assert!(expanded.terms.iter().any(|t| t.term == "custom1"));
    }

    #[test]
    fn test_no_duplicate_terms() {
        let expander = test_fixture();
        let expanded = expander.expand("gold sona"); // gold and sona are synonyms

        // Should not have duplicate terms
        let term_strings: Vec<&str> = expanded.terms.iter().map(|t| t.term.as_str()).collect();
        let unique: std::collections::HashSet<&str> = term_strings.iter().copied().collect();
        assert_eq!(term_strings.len(), unique.len());
    }

    #[test]
    fn test_original_term_boost() {
        let expander = test_fixture();
        let expanded = expander.expand("loan");

        // Original term should have higher weight than expansions
        let original = expanded
            .terms
            .iter()
            .find(|t| t.term == "loan" && t.source == TermSource::Original);
        let expansion = expanded
            .terms
            .iter()
            .find(|t| t.source == TermSource::Synonym);

        if let (Some(orig), Some(exp)) = (original, expansion) {
            assert!(orig.weight > exp.weight);
        }
    }

    #[test]
    fn test_from_domain_config() {
        let synonyms: HashMap<String, Vec<String>> =
            vec![("term1".to_string(), vec!["alt1".to_string()])]
                .into_iter()
                .collect();
        let transliterations: HashMap<String, Vec<String>> =
            vec![("word1".to_string(), vec!["शब्द1".to_string()])]
                .into_iter()
                .collect();
        let stopwords = vec!["the".to_string(), "a".to_string()];

        let expander =
            QueryExpander::from_domain_config("custom_domain", stopwords, synonyms, transliterations);

        assert_eq!(expander.config.domain, "custom_domain");
        assert!(expander.is_stopword("the"));
        assert!(!expander.is_stopword("important"));
    }

    #[test]
    fn test_empty_expander() {
        let expander = QueryExpander::new(QueryExpansionConfig::default());
        let expanded = expander.expand("test query");

        // Should still work, just with no expansions
        assert!(!expanded.was_expanded);
        assert_eq!(expanded.terms.len(), 2); // Just original terms
    }
}
