//! Asset price service with ScyllaDB persistence
//!
//! Provides realistic asset price simulation with:
//! - Daily fluctuation within configurable bounds
//! - Price caching in ScyllaDB
//! - Historical price tracking
//!
//! Note: Currently implemented for gold pricing but designed to be
//! domain-agnostic. The struct field names use domain-specific terminology
//! for gold variants (24K, 22K, 18K) but are accessed through the
//! AssetPriceService trait.

use crate::{PersistenceError, ScyllaClient};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Timelike, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Asset price data (generic struct for collateral pricing)
///
/// Field names use gold terminology for backwards compatibility,
/// but the struct represents any collateral asset pricing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPrice {
    /// Price per unit (default variant)
    pub price_per_gram: f64,
    /// Highest purity/grade price per unit
    pub price_24k: f64,
    /// Standard purity/grade price per unit
    pub price_22k: f64,
    /// Lower purity/grade price per unit
    pub price_18k: f64,
    /// Source of the price
    pub source: String,
    /// When the price was last updated
    pub updated_at: DateTime<Utc>,
}

/// Legacy alias for backwards compatibility
pub type GoldPrice = AssetPrice;

impl AssetPrice {
    /// Calculate maximum loan amount based on asset weight and LTV ratio
    pub fn calculate_max_loan(
        &self,
        weight: f64,
        variant: AssetVariant,
        ltv_ratio: f64,
    ) -> f64 {
        let price = match variant {
            AssetVariant::HighGrade => self.price_24k,
            AssetVariant::StandardGrade => self.price_22k,
            AssetVariant::LowerGrade => self.price_18k,
        };
        weight * price * ltv_ratio
    }

    /// Legacy method for backwards compatibility
    pub fn calculate_max_loan_gold(
        &self,
        gold_weight_grams: f64,
        purity: AssetVariant,
        ltv_ratio: f64,
    ) -> f64 {
        self.calculate_max_loan(gold_weight_grams, purity, ltv_ratio)
    }
}

/// Asset variant/grade levels (generic for any collateral type)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetVariant {
    /// Highest grade (e.g., 24K gold, 99.9% pure)
    HighGrade,
    /// Standard grade (e.g., 22K gold, 91.6% pure)
    StandardGrade,
    /// Lower grade (e.g., 18K gold, 75% pure)
    LowerGrade,
}

/// Legacy alias for backwards compatibility
pub type GoldPurity = AssetVariant;

impl AssetVariant {
    /// Alias constants for gold purity (backwards compatibility)
    pub const K24: AssetVariant = AssetVariant::HighGrade;
    pub const K22: AssetVariant = AssetVariant::StandardGrade;
    pub const K18: AssetVariant = AssetVariant::LowerGrade;
}

/// Asset price service trait (domain-agnostic interface)
#[async_trait]
pub trait AssetPriceService: Send + Sync {
    /// Get the current asset price
    async fn get_current_price(&self) -> Result<AssetPrice, PersistenceError>;

    /// Get historical price for a specific date
    async fn get_historical_price(
        &self,
        date: NaiveDate,
    ) -> Result<Option<AssetPrice>, PersistenceError>;

    /// Force refresh the price (even if cache is valid)
    async fn refresh_price(&self) -> Result<AssetPrice, PersistenceError>;
}

/// Legacy alias for backwards compatibility
pub trait GoldPriceService: AssetPriceService {}

/// Simulated asset price service (currently configured for gold)
#[derive(Clone)]
pub struct SimulatedAssetPriceService {
    client: ScyllaClient,
    base_price_high_grade: f64,
    fluctuation_percent: f64,
    cache_ttl_seconds: i64,
}

/// Legacy alias for backwards compatibility
pub type SimulatedGoldPriceService = SimulatedAssetPriceService;

impl SimulatedAssetPriceService {
    /// Create a new simulated asset price service
    ///
    /// # Arguments
    /// * `client` - ScyllaDB client
    /// * `base_price_high_grade` - Base price for highest grade asset per unit
    pub fn new(client: ScyllaClient, base_price_high_grade: f64) -> Self {
        Self {
            client,
            base_price_high_grade,
            fluctuation_percent: 2.0, // ±2% daily fluctuation
            cache_ttl_seconds: 300,   // 5 minute cache
        }
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
        generate_price_with_params(self.base_price_high_grade, self.fluctuation_percent)
    }
}

/// Generate a simulated asset price (used by tests)
fn generate_price_with_params(base_price_high_grade: f64, fluctuation_percent: f64) -> AssetPrice {
    let mut rng = rand::thread_rng();

    // Generate fluctuation: -fluctuation_percent% to +fluctuation_percent%
    let fluctuation = (rng.gen::<f64>() - 0.5) * 2.0 * (fluctuation_percent / 100.0);
    let price_24k = base_price_high_grade * (1.0 + fluctuation);

    // Calculate other grades based on high grade
    // These factors could be made configurable in future
    let price_22k = price_24k * 0.916; // Standard grade
    let price_18k = price_24k * 0.75; // Lower grade

    AssetPrice {
        price_per_gram: price_22k, // Default to standard grade
        price_24k,
        price_22k,
        price_18k,
        source: "simulated".to_string(),
        updated_at: Utc::now(),
    }
}

impl SimulatedAssetPriceService {
    /// Get cached price from ScyllaDB
    async fn get_cached_price(&self) -> Result<Option<AssetPrice>, PersistenceError> {
        let query = format!(
            "SELECT price_per_gram, price_24k, price_22k, price_18k, updated_at, source
             FROM {}.gold_price_latest WHERE singleton = 1",
            self.client.keyspace()
        );

        let result = self.client.session().query_unpaged(query, &[]).await?;

        if let Some(rows) = result.rows {
            if let Some(row) = rows.into_iter().next() {
                let (price_per_gram, price_24k, price_22k, price_18k, updated_at, source): (
                    f64,
                    f64,
                    f64,
                    f64,
                    i64,
                    String,
                ) = row
                    .into_typed()
                    .map_err(|e| PersistenceError::InvalidData(e.to_string()))?;

                return Ok(Some(AssetPrice {
                    price_per_gram,
                    price_24k,
                    price_22k,
                    price_18k,
                    source,
                    updated_at: DateTime::from_timestamp_millis(updated_at)
                        .unwrap_or_else(Utc::now),
                }));
            }
        }

        Ok(None)
    }

    /// Update the latest price cache
    async fn update_cache(&self, price: &AssetPrice) -> Result<(), PersistenceError> {
        let query = format!(
            "INSERT INTO {}.gold_price_latest (
                singleton, price_per_gram, price_24k, price_22k, price_18k, updated_at, source
            ) VALUES (1, ?, ?, ?, ?, ?, ?)",
            self.client.keyspace()
        );

        self.client
            .session()
            .query_unpaged(
                query,
                (
                    price.price_per_gram,
                    price.price_24k,
                    price.price_22k,
                    price.price_18k,
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

        let query = format!(
            "INSERT INTO {}.gold_prices (
                date, hour, price_per_gram, price_24k, price_22k, price_18k, source, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            self.client.keyspace()
        );

        self.client
            .session()
            .query_unpaged(
                query,
                (
                    date.to_string(),
                    hour,
                    price.price_per_gram,
                    price.price_24k,
                    price.price_22k,
                    price.price_18k,
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
            price_high_grade = price.price_24k,
            price_standard = price.price_22k,
            "Generated new simulated asset price"
        );

        Ok(price)
    }

    async fn get_historical_price(
        &self,
        date: NaiveDate,
    ) -> Result<Option<AssetPrice>, PersistenceError> {
        let query = format!(
            "SELECT price_per_gram, price_24k, price_22k, price_18k, source, created_at
             FROM {}.gold_prices WHERE date = ? LIMIT 1",
            self.client.keyspace()
        );

        let result = self
            .client
            .session()
            .query_unpaged(query, (date.to_string(),))
            .await?;

        if let Some(rows) = result.rows {
            if let Some(row) = rows.into_iter().next() {
                let (price_per_gram, price_24k, price_22k, price_18k, source, created_at): (
                    f64,
                    f64,
                    f64,
                    f64,
                    String,
                    i64,
                ) = row
                    .into_typed()
                    .map_err(|e| PersistenceError::InvalidData(e.to_string()))?;

                return Ok(Some(AssetPrice {
                    price_per_gram,
                    price_24k,
                    price_22k,
                    price_18k,
                    source,
                    updated_at: DateTime::from_timestamp_millis(created_at)
                        .unwrap_or_else(Utc::now),
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

// Blanket implementation for backwards compatibility
impl<T: AssetPriceService> GoldPriceService for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_price_calculation() {
        let price = AssetPrice {
            price_per_gram: 6870.0,
            price_24k: 7500.0,
            price_22k: 6870.0,
            price_18k: 5625.0,
            source: "test".to_string(),
            updated_at: Utc::now(),
        };

        // 100 units of standard grade asset at 75% LTV
        let max_loan = price.calculate_max_loan(100.0, AssetVariant::StandardGrade, 0.75);
        assert!((max_loan - 515250.0).abs() < 1.0); // 100 * 6870 * 0.75 = 515250

        // Test with legacy constants
        let max_loan_legacy = price.calculate_max_loan(100.0, AssetVariant::K22, 0.75);
        assert!((max_loan_legacy - 515250.0).abs() < 1.0);
    }

    #[test]
    fn test_price_generation_bounds() {
        // Generate 100 prices and check they're within bounds
        for _ in 0..100 {
            let price = generate_price_with_params(7500.0, 2.0);
            assert!(price.price_24k >= 7350.0 && price.price_24k <= 7650.0); // ±2%
            assert!(price.price_22k < price.price_24k);
            assert!(price.price_18k < price.price_22k);
        }
    }

    #[test]
    fn test_type_aliases() {
        // Verify that type aliases work for backwards compatibility
        let _price: GoldPrice = AssetPrice {
            price_per_gram: 6870.0,
            price_24k: 7500.0,
            price_22k: 6870.0,
            price_18k: 5625.0,
            source: "test".to_string(),
            updated_at: Utc::now(),
        };

        let _purity: GoldPurity = AssetVariant::StandardGrade;
        let _k22: AssetVariant = AssetVariant::K22;
    }
}
