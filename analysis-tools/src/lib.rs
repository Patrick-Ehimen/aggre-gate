//! Analysis tools for arbitrage detection and market analytics

//! Analysis tools for crypto orderbook aggregator

pub mod arbitrage;

use aggregator_core::{ArbitrageOpportunity, Result, Summary};
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
/// Trait representing an analysis engine for processing market summaries and extracting insights.
///
/// Implementors of this trait are expected to provide asynchronous methods for analyzing
/// collections of summaries, calculating spreads, and computing volume-weighted prices.
///
/// # Required Methods
///
/// - `analyze_summaries`: Analyzes a set of summaries and returns a list of arbitrage opportunities.
/// - `calculate_spread`: Calculates the spread for a given summary, if possible.
/// - `calculate_volume_weighted_price`: Computes the volume-weighted price for a given summary, if possible.
pub trait AnalysisEngine: Send + Sync {
    async fn analyze_summaries(
        &self,
        summaries: &HashMap<String, Summary>,
    ) -> Result<Vec<ArbitrageOpportunity>>;
    async fn calculate_spread(&self, summary: &Summary) -> Option<f64>;
    async fn calculate_volume_weighted_price(&self, summary: &Summary) -> Option<f64>;
}

pub struct DefaultAnalysisEngine;

/// Creates a new instance of `DefaultAnalysisEngine`.
///
/// # Examples
///
/// let engine = DefaultAnalysisEngine::new();
impl DefaultAnalysisEngine {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
/// Implementation of the `AnalysisEngine` trait for `DefaultAnalysisEngine`.
///
/// This implementation provides methods for analyzing market summaries to identify arbitrage opportunities,
/// calculate the spread between best bid and ask prices, and compute the volume-weighted average price.
///
/// # Methods
///
/// - `analyze_summaries`: Asynchronously analyzes a collection of market summaries grouped by symbol to find
///   potential arbitrage opportunities between exchanges. It checks for profitable buy and sell pairs where
///   the profit percentage exceeds a minimum threshold (0.1%). Returns a vector of `ArbitrageOpportunity`.
///
/// - `calculate_spread`: Asynchronously calculates the spread between the best ask and best bid prices in a
///   given summary. Returns the spread as an `Option<f64>`, or `None` if bids or asks are missing.
///
/// - `calculate_volume_weighted_price`: Asynchronously computes the volume-weighted average price for a given
///   summary, considering both bids and asks. Returns the weighted price as an `Option<f64>`, or `None` if
///   there is no volume.
///
/// # Arguments
///
/// - `summaries`: A reference to a `HashMap` mapping symbol strings to their corresponding `Summary` objects.
/// - `summary`: A reference to a `Summary` object representing the order book for a particular symbol.
///
/// # Returns
///
/// - `analyze_summaries`: `Result<Vec<ArbitrageOpportunity>>` containing all found arbitrage opportunities.
/// - `calculate_spread`: `Option<f64>` representing the spread, or `None` if not available.
/// - `calculate_volume_weighted_price`: `Option<f64>` representing the weighted price, or `None` if not available.
///
/// # Example
///
/// let engine = DefaultAnalysisEngine::new();
/// let opportunities = engine.analyze_summaries(&summaries).await?;
/// let spread = engine.calculate_spread(&summary).await;
/// let vwap = engine.calculate_volume_weighted_price(&summary).await;
impl AnalysisEngine for DefaultAnalysisEngine {
    async fn analyze_summaries(
        &self,
        summaries: &HashMap<String, Summary>,
    ) -> Result<Vec<ArbitrageOpportunity>> {
        let mut opportunities = Vec::new();

        // Group summaries by symbol
        let mut symbol_summaries: HashMap<String, Vec<&Summary>> = HashMap::new();
        for summary in summaries.values() {
            symbol_summaries
                .entry(summary.symbol.clone())
                .or_insert_with(Vec::new)
                .push(summary);
        }

        // Find arbitrage opportunities for each symbol
        for (symbol, summaries) in symbol_summaries {
            if summaries.len() < 2 {
                continue; // Need at least 2 exchanges for arbitrage
            }

            for i in 0..summaries.len() {
                for j in i + 1..summaries.len() {
                    let summary1 = summaries[i];
                    let summary2 = summaries[j];

                    if let (Some(best_bid1), Some(best_ask1), Some(best_bid2), Some(best_ask2)) = (
                        summary1.bids.first(),
                        summary1.asks.first(),
                        summary2.bids.first(),
                        summary2.asks.first(),
                    ) {
                        // Check if we can buy on exchange 1 and sell on exchange 2
                        if best_ask1.price < best_bid2.price {
                            let profit = best_bid2.price - best_ask1.price;
                            let profit_percentage = (profit / best_ask1.price) * 100.0;

                            if profit_percentage > 0.1 {
                                // Minimum 0.1% profit
                                opportunities.push(ArbitrageOpportunity {
                                    buy_exchange: best_ask1.exchange.clone(),
                                    sell_exchange: best_bid2.exchange.clone(),
                                    symbol: symbol.clone(),
                                    buy_price: best_ask1.price,
                                    sell_price: best_bid2.price,
                                    profit_percentage,
                                    volume: best_ask1.quantity.min(best_bid2.quantity),
                                    timestamp: chrono::Utc::now(),
                                });
                            }
                        }

                        // Check if we can buy on exchange 2 and sell on exchange 1
                        if best_ask2.price < best_bid1.price {
                            let profit = best_bid1.price - best_ask2.price;
                            let profit_percentage = (profit / best_ask2.price) * 100.0;

                            if profit_percentage > 0.1 {
                                // Minimum 0.1% profit
                                opportunities.push(ArbitrageOpportunity {
                                    buy_exchange: best_ask2.exchange.clone(),
                                    sell_exchange: best_bid1.exchange.clone(),
                                    symbol: symbol.clone(),
                                    buy_price: best_ask2.price,
                                    sell_price: best_bid1.price,
                                    profit_percentage,
                                    volume: best_ask2.quantity.min(best_bid1.quantity),
                                    timestamp: chrono::Utc::now(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(opportunities)
    }

    async fn calculate_spread(&self, summary: &Summary) -> Option<f64> {
        let best_bid = summary.bids.first()?;
        let best_ask = summary.asks.first()?;
        Some(best_ask.price - best_bid.price)
    }

    async fn calculate_volume_weighted_price(&self, summary: &Summary) -> Option<f64> {
        let mut total_volume = 0.0;
        let mut weighted_sum = 0.0;

        // Calculate for bids
        for bid in &summary.bids {
            total_volume += bid.quantity;
            weighted_sum += bid.price * bid.quantity;
        }

        // Calculate for asks
        for ask in &summary.asks {
            total_volume += ask.quantity;
            weighted_sum += ask.price * ask.quantity;
        }

        if total_volume > 0.0 {
            Some(weighted_sum / total_volume)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aggregator_core::{Exchange, PriceLevel, Summary};
    use chrono::Utc;

    #[tokio::test]
    async fn test_analysis_engine_arbitrage_detection() {
        let engine = DefaultAnalysisEngine::new();

        // Create test summaries with arbitrage opportunity
        let mut summaries = HashMap::new();

        let summary1 = Summary {
            symbol: "BTCUSDT".to_string(),
            spread: 100.0,
            bids: vec![PriceLevel {
                price: 50100.0,
                quantity: 1.0,
                exchange: Exchange::Binance,
                timestamp: Utc::now(),
            }],
            asks: vec![PriceLevel {
                price: 50200.0,
                quantity: 1.0,
                exchange: Exchange::Binance,
                timestamp: Utc::now(),
            }],
            timestamp: Utc::now(),
        };

        let summary2 = Summary {
            symbol: "BTCUSDT".to_string(),
            spread: 100.0,
            bids: vec![PriceLevel {
                price: 50300.0, // Higher bid on second exchange
                quantity: 1.0,
                exchange: Exchange::Bybit,
                timestamp: Utc::now(),
            }],
            asks: vec![PriceLevel {
                price: 50400.0,
                quantity: 1.0,
                exchange: Exchange::Bybit,
                timestamp: Utc::now(),
            }],
            timestamp: Utc::now(),
        };

        summaries.insert("binance_btcusdt".to_string(), summary1);
        summaries.insert("bybit_btcusdt".to_string(), summary2);

        let opportunities = engine.analyze_summaries(&summaries).await.unwrap();

        assert!(
            !opportunities.is_empty(),
            "Should find arbitrage opportunities"
        );
        let opp = &opportunities[0];
        assert_eq!(opp.symbol, "BTCUSDT");
        assert!(opp.profit_percentage > 0.0);
    }

    #[tokio::test]
    async fn test_spread_calculation() {
        let engine = DefaultAnalysisEngine::new();

        let summary = Summary {
            symbol: "BTCUSDT".to_string(),
            spread: 100.0,
            bids: vec![PriceLevel {
                price: 50000.0,
                quantity: 1.0,
                exchange: Exchange::Binance,
                timestamp: Utc::now(),
            }],
            asks: vec![PriceLevel {
                price: 50100.0,
                quantity: 1.0,
                exchange: Exchange::Binance,
                timestamp: Utc::now(),
            }],
            timestamp: Utc::now(),
        };

        let spread = engine.calculate_spread(&summary).await;
        assert_eq!(spread, Some(100.0));
    }

    #[tokio::test]
    async fn test_volume_weighted_price() {
        let engine = DefaultAnalysisEngine::new();

        let summary = Summary {
            symbol: "BTCUSDT".to_string(),
            spread: 100.0,
            bids: vec![
                PriceLevel {
                    price: 50000.0,
                    quantity: 1.0,
                    exchange: Exchange::Binance,
                    timestamp: Utc::now(),
                },
                PriceLevel {
                    price: 49900.0,
                    quantity: 2.0,
                    exchange: Exchange::Binance,
                    timestamp: Utc::now(),
                },
            ],
            asks: vec![
                PriceLevel {
                    price: 50100.0,
                    quantity: 1.0,
                    exchange: Exchange::Binance,
                    timestamp: Utc::now(),
                },
                PriceLevel {
                    price: 50200.0,
                    quantity: 2.0,
                    exchange: Exchange::Binance,
                    timestamp: Utc::now(),
                },
            ],
            timestamp: Utc::now(),
        };

        let vwap = engine.calculate_volume_weighted_price(&summary).await;
        assert!(vwap.is_some());

        // Manual calculation: (50000*1 + 49900*2 + 50100*1 + 50200*2) / (1+2+1+2) = 50000
        let expected = (50000.0 * 1.0 + 49900.0 * 2.0 + 50100.0 * 1.0 + 50200.0 * 2.0) / 6.0;
        assert!((vwap.unwrap() - expected).abs() < 0.01);
    }
}

pub use arbitrage::*;
