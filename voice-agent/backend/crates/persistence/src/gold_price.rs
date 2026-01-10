//! Asset price service with ScyllaDB persistence
//!
//! Provides realistic asset price simulation with:
//! - Daily fluctuation within configurable bounds
//! - Price caching in ScyllaDB
//! - Historical price tracking
//! - Dynamic tier support (any number of tiers with config-driven names)

use crate::{PersistenceError, ScyllaClient};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Timelike, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Asset price data with dynamic tier support
///
/// Supports any number of pricing tiers with config-driven names.
/// Examples:
/// - Gold: {"24K": 7500, "22K": 6870, "18K": 5625}
/// - Diamonds: {"VVS1": 50000, "VS1": 30000, "SI1": 15000}
/// - Vehicles: {"Excellent": 100000, "Good": 80000, "Fair": 60000}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPrice {
    /// Base price per unit (default/reference price)
    pub base_price_per_unit: f64,
    /// Dynamic tier prices keyed by tier code from config
    pub tier_prices: HashMap<String, f64>,
    /// Source of the price (e.g., "api", "simulated", "manual")
    pub source: String,
    /// When the price was last updated
    pub updated_at: DateTime<Utc>,
}


impl AssetPrice {
    /// Create a new AssetPrice with the given base price
    pub fn new(base_price_per_unit: f64, source: &str) -> Self {
        Self {
            base_price_per_unit,
            tier_prices: HashMap::new(),
            source: source.to_string(),
            updated_at: Utc::now(),
        }
    }

    /// Add a tier price
    pub fn with_tier(mut self, tier_code: &str, price: f64) -> Self {
        self.tier_prices.insert(tier_code.to_string(), price);
        self
    }

    /// Get price for a specific tier code, falls back to base price
    pub fn price_for_tier(&self, tier_code: &str) -> f64 {
        self.tier_prices
            .get(tier_code)
            .copied()
            .unwrap_or(self.base_price_per_unit)
    }

    /// Calculate maximum loan amount based on asset quantity and LTV ratio
    pub fn calculate_max_loan(&self, quantity: f64, tier_code: &str, ltv_ratio: f64) -> f64 {
        let price = self.price_for_tier(tier_code);
        quantity * price * ltv_ratio
    }

    /// Get all tier codes
    pub fn tier_codes(&self) -> Vec<&str> {
        self.tier_prices.keys().map(|s| s.as_str()).collect()
    }

    /// Get the base price per unit
    #[inline]
    pub fn base_price_per_unit(&self) -> f64 {
        self.base_price_per_unit
    }
}

/// Tier definition for price calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDefinition {
    /// Tier code (e.g., "24K", "VVS1", "Grade A")
    pub code: String,
    /// Factor relative to base price (e.g., 1.0 for highest, 0.916 for 22K gold)
    pub factor: f64,
    /// Human-readable description
    pub description: String,
}

/// Asset price service trait (domain-agnostic interface)
#[async_trait]
pub trait AssetPriceService: Send + Sync {
    /// Get the current asset price with all tiers
    async fn get_current_price(&self) -> Result<AssetPrice, PersistenceError>;

    /// Get historical price for a specific date
    async fn get_historical_price(
        &self,
        date: NaiveDate,
    ) -> Result<Option<AssetPrice>, PersistenceError>;

    /// Force refresh the price (even if cache is valid)
    async fn refresh_price(&self) -> Result<AssetPrice, PersistenceError>;
}


/// Simulated asset price service with configurable tiers
#[derive(Clone)]
pub struct SimulatedAssetPriceService {
    client: ScyllaClient,
    base_price: f64,
    tiers: Vec<TierDefinition>,
    fluctuation_percent: f64,
    cache_ttl_seconds: i64,
}


impl SimulatedAssetPriceService {
    /// Create a new simulated asset price service
    ///
    /// # Arguments
    /// * `client` - ScyllaDB client
    /// * `base_price` - Base price for highest tier per unit
    /// * `tiers` - Tier definitions from config
    pub fn new(client: ScyllaClient, base_price: f64, tiers: Vec<TierDefinition>) -> Self {
        Self {
            client,
            base_price,
            tiers,
            fluctuation_percent: 2.0, // ±2% daily fluctuation
            cache_ttl_seconds: 300,   // 5 minute cache
        }
    }

    /// Create from domain view configuration (preferred)
    ///
    /// P23 FIX: This is the preferred way to create asset price service.
    /// All tier information comes from domain config (slots.yaml).
    ///
    /// # Arguments
    /// * `view` - The ToolsDomainView providing config-driven tier definitions
    /// * `client` - ScyllaDB client
    ///
    /// # Example
    /// ```ignore
    /// let view = ToolsDomainView::new(config);
    /// let service = SimulatedAssetPriceService::from_tiers(
    ///     client,
    ///     view.asset_price_per_unit(),
    ///     view.quality_tiers_full(),
    /// );
    /// ```
    pub fn from_tiers(
        client: ScyllaClient,
        base_price: f64,
        tier_data: Vec<(String, f64, String)>,
    ) -> Self {
        let tiers = tier_data
            .into_iter()
            .map(|(code, factor, description)| TierDefinition {
                code,
                factor,
                description,
            })
            .collect();
        Self::new(client, base_price, tiers)
    }

    /// Set custom fluctuation percentage
    pub fn with_fluctuation(mut self, percent: f64) -> Self {
        self.fluctuation_percent = percent;
        self
    }

    /// Set custom cache TTL
    pub fn with_cache_ttl(mut self, seconds: i64) -> Self {
        self.cache_ttl_seconds = seconds;
        self
    }

    /// Generate a simulated price with realistic fluctuation
    fn generate_price(&self) -> AssetPrice {
        let mut rng = rand::thread_rng();

        // Generate fluctuation: -fluctuation_percent% to +fluctuation_percent%
        let fluctuation = (rng.gen::<f64>() - 0.5) * 2.0 * (self.fluctuation_percent / 100.0);
        let base_with_fluctuation = self.base_price * (1.0 + fluctuation);

        let mut price = AssetPrice::new(base_with_fluctuation, "simulated");

        // Calculate price for each tier
        for tier in &self.tiers {
            let tier_price = base_with_fluctuation * tier.factor;
            price.tier_prices.insert(tier.code.clone(), tier_price);
        }

        // Set base to the standard tier if available (typically the most common)
        if let Some(standard_price) = price.tier_prices.get("22K").or_else(|| {
            // Find tier with factor closest to 0.9 as "standard"
            self.tiers
                .iter()
                .filter(|t| t.factor > 0.8 && t.factor < 1.0)
                .min_by(|a, b| {
                    (a.factor - 0.9)
                        .abs()
                        .partial_cmp(&(b.factor - 0.9).abs())
                        .unwrap()
                })
                .and_then(|t| price.tier_prices.get(&t.code))
        }) {
            price.base_price_per_unit = *standard_price;
        }

        price
    }

    /// Get cached price from ScyllaDB
    async fn get_cached_price(&self) -> Result<Option<AssetPrice>, PersistenceError> {
        // Query the latest price - DB stores in JSON format for flexibility
        let query = format!(
            "SELECT base_price, tier_prices_json, updated_at, source
             FROM {}.asset_price_latest WHERE singleton = 1",
            self.client.keyspace()
        );

        let result = self.client.session().query_unpaged(query, &[]).await?;

        if let Some(rows) = result.rows {
            if let Some(row) = rows.into_iter().next() {
                let (base_price, tier_prices_json, updated_at, source): (f64, String, i64, String) =
                    row.into_typed()
                        .map_err(|e| PersistenceError::InvalidData(e.to_string()))?;

                let tier_prices: HashMap<String, f64> = serde_json::from_str(&tier_prices_json)
                    .map_err(|e| PersistenceError::InvalidData(e.to_string()))?;

                return Ok(Some(AssetPrice {
                    base_price_per_unit: base_price,
                    tier_prices,
                    source,
                    updated_at: DateTime::from_timestamp_millis(updated_at).unwrap_or_else(Utc::now),
                }));
            }
        }

        Ok(None)
    }

    /// Update the latest price cache
    async fn update_cache(&self, price: &AssetPrice) -> Result<(), PersistenceError> {
        let tier_prices_json = serde_json::to_string(&price.tier_prices)
            .map_err(|e| PersistenceError::InvalidData(e.to_string()))?;

        let query = format!(
            "INSERT INTO {}.asset_price_latest (
                singleton, base_price, tier_prices_json, updated_at, source
            ) VALUES (1, ?, ?, ?, ?)",
            self.client.keyspace()
        );

        self.client
            .session()
            .query_unpaged(
                query,
                (
                    price.base_price_per_unit,
                    &tier_prices_json,
                    price.updated_at.timestamp_millis(),
                    &price.source,
                ),
            )
            .await?;

        Ok(())
    }

    /// Record price in history table
    async fn record_history(&self, price: &AssetPrice) -> Result<(), PersistenceError> {
        let now = Utc::now();
        let date = now.date_naive();
        let hour = now.hour() as i32;

        let tier_prices_json = serde_json::to_string(&price.tier_prices)
            .map_err(|e| PersistenceError::InvalidData(e.to_string()))?;

        let query = format!(
            "INSERT INTO {}.asset_prices (
                date, hour, base_price, tier_prices_json, source, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)",
            self.client.keyspace()
        );

        self.client
            .session()
            .query_unpaged(
                query,
                (
                    date.to_string(),
                    hour,
                    price.base_price_per_unit,
                    &tier_prices_json,
                    &price.source,
                    now.timestamp_millis(),
                ),
            )
            .await?;

        Ok(())
    }
}

#[async_trait]
impl AssetPriceService for SimulatedAssetPriceService {
    async fn get_current_price(&self) -> Result<AssetPrice, PersistenceError> {
        // Check cache first
        if let Some(cached) = self.get_cached_price().await? {
            let age = Utc::now() - cached.updated_at;
            if age.num_seconds() < self.cache_ttl_seconds {
                tracing::debug!(
                    age_seconds = age.num_seconds(),
                    "Returning cached asset price"
                );
                return Ok(cached);
            }
        }

        // Generate new price
        let price = self.generate_price();

        // Update cache and history
        self.update_cache(&price).await?;
        self.record_history(&price).await?;

        tracing::info!(
            base_price = price.base_price_per_unit,
            tier_count = price.tier_prices.len(),
            "Generated new simulated asset price"
        );

        Ok(price)
    }

    async fn get_historical_price(
        &self,
        date: NaiveDate,
    ) -> Result<Option<AssetPrice>, PersistenceError> {
        let query = format!(
            "SELECT base_price, tier_prices_json, source, created_at
             FROM {}.asset_prices WHERE date = ? LIMIT 1",
            self.client.keyspace()
        );

        let result = self
            .client
            .session()
            .query_unpaged(query, (date.to_string(),))
            .await?;

        if let Some(rows) = result.rows {
            if let Some(row) = rows.into_iter().next() {
                let (base_price, tier_prices_json, source, created_at): (f64, String, String, i64) =
                    row.into_typed()
                        .map_err(|e| PersistenceError::InvalidData(e.to_string()))?;

                let tier_prices: HashMap<String, f64> = serde_json::from_str(&tier_prices_json)
                    .map_err(|e| PersistenceError::InvalidData(e.to_string()))?;

                return Ok(Some(AssetPrice {
                    base_price_per_unit: base_price,
                    tier_prices,
                    source,
                    updated_at: DateTime::from_timestamp_millis(created_at).unwrap_or_else(Utc::now),
                }));
            }
        }

        Ok(None)
    }

    async fn refresh_price(&self) -> Result<AssetPrice, PersistenceError> {
        let price = self.generate_price();
        self.update_cache(&price).await?;
        self.record_history(&price).await?;
        Ok(price)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_price_dynamic_tiers() {
        // P23 FIX: Use generic tier names - actual values come from domain config
        let price = AssetPrice::new(100.0, "test")
            .with_tier("tier_1", 100.0)
            .with_tier("tier_2", 91.6)
            .with_tier("tier_3", 75.0);

        // Test tier lookups
        assert!((price.price_for_tier("tier_1") - 100.0).abs() < 0.01);
        assert!((price.price_for_tier("tier_2") - 91.6).abs() < 0.01);
        assert!((price.price_for_tier("tier_3") - 75.0).abs() < 0.01);

        // Test unknown tier falls back to base
        assert!((price.price_for_tier("tier_4") - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_asset_price_calculation() {
        // P23 FIX: Use generic tier names - actual values come from domain config
        let price = AssetPrice::new(91.6, "test")
            .with_tier("tier_1", 100.0)
            .with_tier("tier_2", 91.6)
            .with_tier("tier_3", 75.0);

        // 100 units of tier_2 at 75% LTV
        let max_loan = price.calculate_max_loan(100.0, "tier_2", 0.75);
        assert!((max_loan - 6870.0).abs() < 1.0); // 100 * 91.6 * 0.75 = 6870
    }


    #[test]
    fn test_diamond_domain_example() {
        // Example: Diamond pricing with different tier structure
        let price = AssetPrice::new(30000.0, "test")
            .with_tier("VVS1", 50000.0)
            .with_tier("VS1", 30000.0)
            .with_tier("SI1", 15000.0);

        assert!((price.price_for_tier("VVS1") - 50000.0).abs() < 0.01);
        assert!((price.price_for_tier("VS1") - 30000.0).abs() < 0.01);
        assert!((price.price_for_tier("SI1") - 15000.0).abs() < 0.01);

        // Loan calculation works with any tier code
        let max_loan = price.calculate_max_loan(2.0, "VS1", 0.60);
        assert!((max_loan - 36000.0).abs() < 1.0); // 2 * 30000 * 0.60 = 36000
    }

    #[test]
    fn test_tier_codes() {
        let price = AssetPrice::new(100.0, "test")
            .with_tier("A", 100.0)
            .with_tier("B", 80.0)
            .with_tier("C", 60.0);

        let codes = price.tier_codes();
        assert!(codes.contains(&"A"));
        assert!(codes.contains(&"B"));
        assert!(codes.contains(&"C"));
        assert_eq!(codes.len(), 3);
    }
}
