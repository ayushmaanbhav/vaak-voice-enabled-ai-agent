//! Domain-specific term boosting for retrieval
//!
//! P16 FIX: Terms are now config-driven via MasterDomainConfig.domain_boost.
//! No hardcoded domain-specific terminology.

use parking_lot::RwLock;
use std::collections::HashMap;

/// Domain booster configuration
/// P16 FIX: Now uses config values, not hardcoded defaults
#[derive(Debug, Clone)]
pub struct DomainBoostConfig {
    /// Base boost multiplier for domain terms
    pub base_boost: f32,
    /// Additional boost for exact matches
    pub exact_match_boost: f32,
    /// Boost for brand terms
    pub brand_boost: f32,
    /// Enable category-based boosting
    pub category_boost_enabled: bool,
}

impl Default for DomainBoostConfig {
    fn default() -> Self {
        Self {
            base_boost: 1.5,
            exact_match_boost: 2.0,
            brand_boost: 1.3,
            category_boost_enabled: true,
        }
    }
}

/// Domain term with metadata
#[derive(Debug, Clone)]
pub struct DomainTerm {
    /// The term
    pub term: String,
    /// Category (e.g., "product", "rate", "process")
    pub category: TermCategory,
    /// Boost multiplier
    pub boost: f32,
    /// Related terms for context matching
    pub related: Vec<String>,
}

/// Term category for domain boosting
/// P16 FIX: Generic categories, not domain-specific
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermCategory {
    /// Product/service terms
    Product,
    /// Rate/pricing terms (interest, emi, fees)
    Rate,
    /// Process terms (apply, documents, disbursal)
    Process,
    /// Eligibility terms (criteria, requirements)
    Eligibility,
    /// Brand terms
    Brand,
    /// Asset-specific terms (for valuation)
    Asset,
    /// Customer terms (account, kyc)
    Customer,
    /// Competitor terms
    Competitor,
    /// General/other
    General,
}

impl TermCategory {
    /// Parse category from config string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "product" => Self::Product,
            "rate" => Self::Rate,
            "process" => Self::Process,
            "eligibility" => Self::Eligibility,
            "brand" => Self::Brand,
            "asset" | "gold" => Self::Asset,
            "customer" => Self::Customer,
            "competitor" => Self::Competitor,
            _ => Self::General,
        }
    }

    /// Get default category boost multiplier
    pub fn default_boost_multiplier(&self) -> f32 {
        match self {
            Self::Product => 1.5,
            Self::Rate => 1.4,
            Self::Eligibility => 1.3,
            Self::Process => 1.2,
            Self::Brand => 1.5,
            Self::Asset => 1.4,
            Self::Customer => 1.1,
            Self::Competitor => 1.3,
            Self::General => 1.0,
        }
    }
}

/// Boost result for a query
#[derive(Debug, Clone)]
pub struct BoostResult {
    /// Matched domain terms
    pub matched_terms: Vec<MatchedTerm>,
    /// Overall boost factor
    pub total_boost: f32,
    /// Detected categories
    pub categories: Vec<TermCategory>,
    /// Query intent (if determinable)
    pub intent: Option<QueryIntent>,
}

/// A matched domain term
#[derive(Debug, Clone)]
pub struct MatchedTerm {
    /// The term that matched
    pub term: String,
    /// Position in query
    pub position: usize,
    /// Boost applied
    pub boost: f32,
    /// Category
    pub category: TermCategory,
}

/// Detected query intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryIntent {
    /// Asking about rates/pricing
    RateInquiry,
    /// Checking eligibility
    EligibilityCheck,
    /// Application process
    ApplicationProcess,
    /// Amount calculation
    AmountCalculation,
    /// Branch/location inquiry
    BranchLocator,
    /// Document requirements
    DocumentInquiry,
    /// Competitor comparison
    CompetitorComparison,
    /// Repayment query
    RepaymentInquiry,
    /// General inquiry
    General,
}

/// Domain booster for retrieval relevance
/// P16 FIX: Now accepts terms from config instead of hardcoding
pub struct DomainBooster {
    config: DomainBoostConfig,
    /// Domain terms dictionary
    terms: RwLock<HashMap<String, DomainTerm>>,
    /// Category boost multipliers from config
    category_boosts: HashMap<String, f32>,
    /// Intent patterns (config-driven in the future)
    intent_patterns: Vec<(Vec<String>, QueryIntent)>,
}

impl DomainBooster {
    /// Create a new domain booster with local config
    pub fn new(config: DomainBoostConfig) -> Self {
        Self {
            config,
            terms: RwLock::new(HashMap::new()),
            category_boosts: HashMap::new(),
            intent_patterns: Self::default_intent_patterns(),
        }
    }

    /// P16 FIX: Create from MasterDomainConfig
    pub fn from_config(master: &voice_agent_config::MasterDomainConfig) -> Self {
        let config_boost = &master.domain_boost;

        let config = DomainBoostConfig {
            base_boost: config_boost.default_boost as f32,
            exact_match_boost: 2.0,
            brand_boost: 1.3,
            category_boost_enabled: true,
        };

        let mut booster = Self {
            config,
            terms: RwLock::new(HashMap::new()),
            category_boosts: config_boost.category_boosts.iter()
                .map(|(k, v)| (k.clone(), *v as f32))
                .collect(),
            intent_patterns: Self::default_intent_patterns(),
        };

        // Load terms from config
        booster.load_terms_from_config(&config_boost.terms);

        booster
    }

    /// P16 FIX: Load terms from config
    fn load_terms_from_config(&self, config_terms: &[voice_agent_config::domain::DomainBoostTermEntry]) {
        let mut term_map = self.terms.write();

        for entry in config_terms {
            let category = TermCategory::from_str(&entry.category);

            let term = DomainTerm {
                term: entry.term.clone(),
                category,
                boost: entry.boost as f32,
                related: entry.related.clone(),
            };

            // Add main term
            term_map.insert(term.term.to_lowercase(), term.clone());

            // Add related terms with lower boost
            for related in &entry.related {
                if let std::collections::hash_map::Entry::Vacant(e) =
                    term_map.entry(related.to_lowercase())
                {
                    e.insert(DomainTerm {
                        term: related.clone(),
                        category,
                        boost: entry.boost as f32 * 0.8,
                        related: vec![entry.term.clone()],
                    });
                }
            }
        }
    }

    /// Create with default config (no terms loaded - use from_config for production)
    pub fn with_defaults() -> Self {
        Self::new(DomainBoostConfig::default())
    }

    /// Default intent patterns (generic, not domain-specific)
    fn default_intent_patterns() -> Vec<(Vec<String>, QueryIntent)> {
        vec![
            // Rate inquiry
            (vec!["interest".into(), "rate".into()], QueryIntent::RateInquiry),
            (vec!["emi".into(), "calculate".into()], QueryIntent::RateInquiry),
            // Eligibility
            (vec!["eligible".into()], QueryIntent::EligibilityCheck),
            (vec!["eligibility".into()], QueryIntent::EligibilityCheck),
            (vec!["criteria".into()], QueryIntent::EligibilityCheck),
            // Application
            (vec!["apply".into()], QueryIntent::ApplicationProcess),
            (vec!["application".into(), "process".into()], QueryIntent::ApplicationProcess),
            // Amount
            (vec!["amount".into()], QueryIntent::AmountCalculation),
            (vec!["maximum".into()], QueryIntent::AmountCalculation),
            (vec!["how".into(), "much".into()], QueryIntent::AmountCalculation),
            // Branch
            (vec!["branch".into()], QueryIntent::BranchLocator),
            (vec!["nearest".into()], QueryIntent::BranchLocator),
            (vec!["location".into()], QueryIntent::BranchLocator),
            // Documents
            (vec!["document".into()], QueryIntent::DocumentInquiry),
            (vec!["papers".into()], QueryIntent::DocumentInquiry),
            (vec!["kyc".into()], QueryIntent::DocumentInquiry),
            // Competitor
            (vec!["compare".into()], QueryIntent::CompetitorComparison),
            (vec!["better".into(), "than".into()], QueryIntent::CompetitorComparison),
            (vec!["switch".into()], QueryIntent::CompetitorComparison),
            // Repayment
            (vec!["repay".into()], QueryIntent::RepaymentInquiry),
            (vec!["prepay".into()], QueryIntent::RepaymentInquiry),
            (vec!["foreclosure".into()], QueryIntent::RepaymentInquiry),
        ]
    }

    /// Get category boost multiplier (from config or default)
    fn category_boost_multiplier(&self, category: TermCategory) -> f32 {
        let category_name = match category {
            TermCategory::Product => "product",
            TermCategory::Rate => "rate",
            TermCategory::Process => "process",
            TermCategory::Eligibility => "eligibility",
            TermCategory::Brand => "brand",
            TermCategory::Asset => "asset",
            TermCategory::Customer => "customer",
            TermCategory::Competitor => "competitor",
            TermCategory::General => "general",
        };

        self.category_boosts
            .get(category_name)
            .copied()
            .unwrap_or_else(|| category.default_boost_multiplier())
    }

    /// Calculate boost for a query
    pub fn boost(&self, query: &str) -> BoostResult {
        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        let terms = self.terms.read();
        let mut matched_terms = Vec::new();
        let mut categories = Vec::new();

        // Match single words
        for (pos, word) in words.iter().enumerate() {
            if let Some(domain_term) = terms.get(*word) {
                matched_terms.push(MatchedTerm {
                    term: domain_term.term.clone(),
                    position: pos,
                    boost: domain_term.boost * self.config.base_boost,
                    category: domain_term.category,
                });
                if !categories.contains(&domain_term.category) {
                    categories.push(domain_term.category);
                }
            }
        }

        // Match multi-word terms
        for (term, domain_term) in terms.iter() {
            if term.contains(' ') && query_lower.contains(term) {
                // Avoid double counting if individual words already matched
                if !matched_terms.iter().any(|m| m.term == domain_term.term) {
                    matched_terms.push(MatchedTerm {
                        term: domain_term.term.clone(),
                        position: query_lower.find(term).unwrap_or(0),
                        boost: domain_term.boost * self.config.exact_match_boost,
                        category: domain_term.category,
                    });
                    if !categories.contains(&domain_term.category) {
                        categories.push(domain_term.category);
                    }
                }
            }
        }

        // Detect intent
        let intent = self.detect_intent(&query_lower);

        // Calculate total boost
        let total_boost = if matched_terms.is_empty() {
            1.0
        } else {
            let base: f32 =
                matched_terms.iter().map(|m| m.boost).sum::<f32>() / matched_terms.len() as f32;
            let category_bonus: f32 = if self.config.category_boost_enabled {
                categories.iter().map(|c| self.category_boost_multiplier(*c)).sum::<f32>()
                    / categories.len().max(1) as f32
            } else {
                1.0
            };
            base * category_bonus
        };

        BoostResult {
            matched_terms,
            total_boost,
            categories,
            intent,
        }
    }

    /// Detect query intent
    fn detect_intent(&self, query: &str) -> Option<QueryIntent> {
        for (patterns, intent) in &self.intent_patterns {
            if patterns.iter().all(|p| query.contains(p)) {
                return Some(*intent);
            }
        }
        None
    }

    /// Apply boost to search results
    pub fn apply_boost<T>(&self, results: &mut [(T, f32)], query: &str)
    where
        T: AsRef<str>,
    {
        let boost_result = self.boost(query);

        for (doc, score) in results.iter_mut() {
            let doc_text = doc.as_ref().to_lowercase();

            // Calculate document-specific boost
            let mut doc_boost = 1.0f32;

            for matched in &boost_result.matched_terms {
                if doc_text.contains(&matched.term.to_lowercase()) {
                    doc_boost *= matched.boost;
                }
            }

            // Apply brand boost for brand terms in document
            for matched in &boost_result.matched_terms {
                if matched.category == TermCategory::Brand && doc_text.contains(&matched.term.to_lowercase()) {
                    doc_boost *= self.config.brand_boost;
                    break;
                }
            }

            *score *= doc_boost;
        }
    }

    /// Add a custom domain term
    pub fn add_term(&self, term: DomainTerm) {
        let mut terms = self.terms.write();
        terms.insert(term.term.to_lowercase(), term);
    }

    /// Get intent label for display
    pub fn intent_label(intent: QueryIntent) -> &'static str {
        match intent {
            QueryIntent::RateInquiry => "Rate/Pricing Inquiry",
            QueryIntent::EligibilityCheck => "Eligibility Check",
            QueryIntent::ApplicationProcess => "Application Process",
            QueryIntent::AmountCalculation => "Amount Calculation",
            QueryIntent::BranchLocator => "Location Finder",
            QueryIntent::DocumentInquiry => "Document Requirements",
            QueryIntent::CompetitorComparison => "Provider Comparison",
            QueryIntent::RepaymentInquiry => "Repayment Query",
            QueryIntent::General => "General Inquiry",
        }
    }

    /// Check if booster has any terms loaded
    pub fn has_terms(&self) -> bool {
        !self.terms.read().is_empty()
    }

    /// Get number of loaded terms
    pub fn term_count(&self) -> usize {
        self.terms.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_boost() {
        let booster = DomainBooster::with_defaults();
        // Add some test terms
        booster.add_term(DomainTerm {
            term: "service".to_string(),
            category: TermCategory::Product,
            boost: 2.0,
            related: vec![],
        });
        booster.add_term(DomainTerm {
            term: "interest".to_string(),
            category: TermCategory::Rate,
            boost: 1.8,
            related: vec![],
        });

        let result = booster.boost("service interest rate");

        assert!(!result.matched_terms.is_empty());
        assert!(result.total_boost > 1.0);
    }

    #[test]
    fn test_term_category_parsing() {
        assert_eq!(TermCategory::from_str("product"), TermCategory::Product);
        assert_eq!(TermCategory::from_str("rate"), TermCategory::Rate);
        assert_eq!(TermCategory::from_str("gold"), TermCategory::Asset);
        assert_eq!(TermCategory::from_str("unknown"), TermCategory::General);
    }

    #[test]
    fn test_intent_detection() {
        let booster = DomainBooster::with_defaults();

        // Test rate inquiry
        let result = booster.boost("what is the interest rate");
        assert_eq!(result.intent, Some(QueryIntent::RateInquiry));

        // Test eligibility
        let result = booster.boost("am I eligible for this");
        assert_eq!(result.intent, Some(QueryIntent::EligibilityCheck));

        // Test branch
        let result = booster.boost("find nearest branch");
        assert_eq!(result.intent, Some(QueryIntent::BranchLocator));
    }

    #[test]
    fn test_empty_query() {
        let booster = DomainBooster::with_defaults();
        let result = booster.boost("");

        assert!(result.matched_terms.is_empty());
        assert!((result.total_boost - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_custom_category_boosts() {
        let booster = DomainBooster::with_defaults();

        // Check default boost multiplier
        let boost = booster.category_boost_multiplier(TermCategory::Product);
        assert!((boost - 1.5).abs() < 0.001);
    }
}
