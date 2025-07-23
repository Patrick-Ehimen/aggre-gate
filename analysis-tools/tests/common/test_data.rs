//! Test data factory for generating consistent test data across all test files

use aggregator_core::{ArbitrageOpportunity, Exchange, PriceLevel, Summary, TradingPair};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Factory for creating test data with consistent parameters
pub struct TestDataFactory;

impl TestDataFactory {
    /// Generate a Summary with configurable parameters
    pub fn create_summary(
        symbol: &str,
        exchange: Exchange,
        best_bid: f64,
        best_ask: f64,
        bid_quantity: f64,
        ask_quantity: f64,
    ) -> Summary {
        let timestamp = Utc::now();

        Summary {
            symbol: symbol.to_string(),
            spread: best_ask - best_bid,
            bids: vec![PriceLevel {
                price: best_bid,
                quantity: bid_quantity,
                exchange: exchange.clone(),
                timestamp,
            }],
            asks: vec![PriceLevel {
                price: best_ask,
                quantity: ask_quantity,
                exchange,
                timestamp,
            }],
            timestamp,
        }
    }

    /// Generate a Summary with multiple price levels
    pub fn create_summary_with_depth(
        symbol: &str,
        exchange: Exchange,
        bid_levels: Vec<(f64, f64)>, // (price, quantity) pairs
        ask_levels: Vec<(f64, f64)>, // (price, quantity) pairs
    ) -> Summary {
        let timestamp = Utc::now();

        let bids: Vec<PriceLevel> = bid_levels
            .into_iter()
            .map(|(price, quantity)| PriceLevel {
                price,
                quantity,
                exchange: exchange.clone(),
                timestamp,
            })
            .collect();

        let asks: Vec<PriceLevel> = ask_levels
            .into_iter()
            .map(|(price, quantity)| PriceLevel {
                price,
                quantity,
                exchange: exchange.clone(),
                timestamp,
            })
            .collect();

        let spread = asks.first().map(|a| a.price).unwrap_or(0.0)
            - bids.first().map(|b| b.price).unwrap_or(0.0);

        Summary {
            symbol: symbol.to_string(),
            spread,
            bids,
            asks,
            timestamp,
        }
    }

    /// Generate TradingPair
    pub fn create_trading_pair(base: &str, quote: &str) -> TradingPair {
        TradingPair::new(base, quote)
    }

    /// Generate multiple PriceLevels for realistic order books
    pub fn create_price_levels(
        base_price: f64,
        count: usize,
        is_bid: bool,
        exchange: Exchange,
    ) -> Vec<PriceLevel> {
        let timestamp = Utc::now();
        let mut levels = Vec::new();

        for i in 0..count {
            let price_offset = (i as f64) * 0.01; // 1 cent increments
            let price = if is_bid {
                base_price - price_offset // Bids decrease from best price
            } else {
                base_price + price_offset // Asks increase from best price
            };

            levels.push(PriceLevel {
                price,
                quantity: 1.0 + (i as f64 * 0.1), // Increasing quantities
                exchange: exchange.clone(),
                timestamp,
            });
        }

        levels
    }

    /// Generate arbitrage scenario data
    pub fn create_arbitrage_scenario(
        symbol: &str,
        buy_exchange: Exchange,
        sell_exchange: Exchange,
        buy_price: f64,
        sell_price: f64,
        volume: f64,
    ) -> HashMap<TradingPair, Vec<Summary>> {
        let mut summaries = HashMap::new();
        let pair = TradingPair::new(
            &symbol[..3], // Assume first 3 chars are base
            &symbol[3..], // Rest is quote
        );

        let buy_summary = Self::create_summary(
            symbol,
            buy_exchange,
            buy_price - 1.0, // Bid slightly lower than ask
            buy_price,       // Ask price for buying
            volume,
            volume,
        );

        let sell_summary = Self::create_summary(
            symbol,
            sell_exchange,
            sell_price,       // Bid price for selling
            sell_price + 1.0, // Ask slightly higher than bid
            volume,
            volume,
        );

        summaries.insert(pair, vec![buy_summary, sell_summary]);
        summaries
    }

    /// Create a scenario with no arbitrage opportunities
    pub fn create_no_arbitrage_scenario(symbol: &str) -> HashMap<TradingPair, Vec<Summary>> {
        let mut summaries = HashMap::new();
        let pair = TradingPair::new(&symbol[..3], &symbol[3..]);

        let summary1 = Self::create_summary(
            symbol,
            Exchange::Binance,
            50000.0, // bid
            50100.0, // ask
            1.0,
            1.0,
        );

        let summary2 = Self::create_summary(
            symbol,
            Exchange::Bybit,
            50050.0, // bid (lower than first exchange's ask)
            50150.0, // ask
            1.0,
            1.0,
        );

        summaries.insert(pair, vec![summary1, summary2]);
        summaries
    }

    /// Create a scenario with identical prices (edge case)
    pub fn create_identical_prices_scenario(symbol: &str) -> HashMap<TradingPair, Vec<Summary>> {
        let mut summaries = HashMap::new();
        let pair = TradingPair::new(&symbol[..3], &symbol[3..]);

        let summary1 = Self::create_summary(
            symbol,
            Exchange::Binance,
            50000.0, // bid
            50100.0, // ask
            1.0,
            1.0,
        );

        let summary2 = Self::create_summary(
            symbol,
            Exchange::Bybit,
            50000.0, // identical bid
            50100.0, // identical ask
            1.0,
            1.0,
        );

        summaries.insert(pair, vec![summary1, summary2]);
        summaries
    }

    /// Create a scenario with single exchange (no arbitrage possible)
    pub fn create_single_exchange_scenario(symbol: &str) -> HashMap<TradingPair, Vec<Summary>> {
        let mut summaries = HashMap::new();
        let pair = TradingPair::new(&symbol[..3], &symbol[3..]);

        let summary = Self::create_summary(symbol, Exchange::Binance, 50000.0, 50100.0, 1.0, 1.0);

        summaries.insert(pair, vec![summary]);
        summaries
    }

    /// Create a scenario with empty summaries
    pub fn create_empty_scenario() -> HashMap<TradingPair, Vec<Summary>> {
        HashMap::new()
    }

    /// Create a scenario with missing bid data
    pub fn create_missing_bid_scenario(symbol: &str) -> Summary {
        let timestamp = Utc::now();

        Summary {
            symbol: symbol.to_string(),
            spread: 0.0,
            bids: vec![], // Empty bids
            asks: vec![PriceLevel {
                price: 50100.0,
                quantity: 1.0,
                exchange: Exchange::Binance,
                timestamp,
            }],
            timestamp,
        }
    }

    /// Create a scenario with missing ask data
    pub fn create_missing_ask_scenario(symbol: &str) -> Summary {
        let timestamp = Utc::now();

        Summary {
            symbol: symbol.to_string(),
            spread: 0.0,
            bids: vec![PriceLevel {
                price: 50000.0,
                quantity: 1.0,
                exchange: Exchange::Binance,
                timestamp,
            }],
            asks: vec![], // Empty asks
            timestamp,
        }
    }

    /// Create a scenario with zero quantities
    pub fn create_zero_quantity_scenario(symbol: &str) -> Summary {
        let timestamp = Utc::now();

        Summary {
            symbol: symbol.to_string(),
            spread: 100.0,
            bids: vec![PriceLevel {
                price: 50000.0,
                quantity: 0.0, // Zero quantity
                exchange: Exchange::Binance,
                timestamp,
            }],
            asks: vec![PriceLevel {
                price: 50100.0,
                quantity: 0.0, // Zero quantity
                exchange: Exchange::Binance,
                timestamp,
            }],
            timestamp,
        }
    }

    /// Create a scenario with extreme price values
    pub fn create_extreme_values_scenario(symbol: &str) -> Summary {
        let timestamp = Utc::now();

        Summary {
            symbol: symbol.to_string(),
            spread: f64::MAX - f64::MIN,
            bids: vec![PriceLevel {
                price: f64::MAX,
                quantity: f64::MAX,
                exchange: Exchange::Binance,
                timestamp,
            }],
            asks: vec![PriceLevel {
                price: f64::MIN,
                quantity: f64::MIN,
                exchange: Exchange::Binance,
                timestamp,
            }],
            timestamp,
        }
    }

    /// Create a realistic market scenario with multiple exchanges and price levels
    pub fn create_realistic_market_scenario(symbol: &str) -> HashMap<TradingPair, Vec<Summary>> {
        let mut summaries = HashMap::new();
        let pair = TradingPair::new(&symbol[..3], &symbol[3..]);

        // Binance - slightly higher prices
        let binance_summary = Self::create_summary_with_depth(
            symbol,
            Exchange::Binance,
            vec![(50000.0, 1.5), (49999.0, 2.0), (49998.0, 1.0)], // bids
            vec![(50001.0, 1.2), (50002.0, 1.8), (50003.0, 2.5)], // asks
        );

        // Bybit - slightly lower prices (potential arbitrage)
        let bybit_summary = Self::create_summary_with_depth(
            symbol,
            Exchange::Bybit,
            vec![(49995.0, 1.0), (49994.0, 1.5), (49993.0, 2.0)], // bids
            vec![(49996.0, 1.3), (49997.0, 1.7), (49998.0, 2.2)], // asks
        );

        // Coinbase - middle ground
        let coinbase_summary = Self::create_summary_with_depth(
            symbol,
            Exchange::Coinbase,
            vec![(49998.0, 1.1), (49997.0, 1.6), (49996.0, 1.9)], // bids
            vec![(49999.0, 1.4), (50000.0, 1.8), (50001.0, 2.1)], // asks
        );

        summaries.insert(pair, vec![binance_summary, bybit_summary, coinbase_summary]);
        summaries
    }

    /// Create test configuration with default values
    pub fn create_test_config() -> TestConfig {
        TestConfig::default()
    }

    /// Create test configuration with custom values
    pub fn create_custom_test_config(
        profit_threshold: f64,
        volume_threshold: f64,
        price_tolerance: f64,
        time_tolerance_ms: u64,
    ) -> TestConfig {
        TestConfig {
            profit_threshold,
            volume_threshold,
            price_tolerance,
            time_tolerance: std::time::Duration::from_millis(time_tolerance_ms),
        }
    }
}

/// Test configuration structure for consistent test parameters
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub profit_threshold: f64,
    pub volume_threshold: f64,
    pub price_tolerance: f64,
    pub time_tolerance: std::time::Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            profit_threshold: 0.1,
            volume_threshold: 0.01,
            price_tolerance: 0.001,
            time_tolerance: std::time::Duration::from_millis(100),
        }
    }
}

/// Test scenario definitions for different market conditions
#[derive(Debug, Clone)]
pub enum TestScenario {
    NoArbitrage,
    SimpleArbitrage { profit_percentage: f64 },
    MultipleOpportunities { count: usize },
    BelowThreshold { profit_percentage: f64 },
    HighVolume { volume_multiplier: f64 },
    EdgeCase { case_type: EdgeCaseType },
}

#[derive(Debug, Clone)]
pub enum EdgeCaseType {
    EmptyOrderBook,
    SingleExchange,
    IdenticalPrices,
    ExtremeValues,
    ZeroQuantities,
}
