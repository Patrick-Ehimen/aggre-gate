//! # Arbitrage Module
//!
//! This module provides tools for detecting arbitrage opportunities in cryptocurrency markets.
//! It includes functionalities for identifying simple, triangular, and more complex arbitrage
//! scenarios.

use aggregator_core::{ArbitrageOpportunity, Summary, TradingPair};
use chrono::Utc;
use std::collections::HashMap;

/// # Arbitrage Detector
///
/// A struct that encapsulates the logic for detecting arbitrage opportunities. It holds
/// configurable thresholds for minimum profit and volume to filter out insignificant
/// opportunities.
///
/// ## Fields
///
/// - `min_profit_threshold`: The minimum profit percentage required to consider an
///   opportunity as valid.
/// - `min_volume_threshold`: The minimum trade volume required for an opportunity.
pub struct ArbitrageDetector {
    min_profit_threshold: f64,
    min_volume_threshold: f64,
}

impl ArbitrageDetector {
    /// ## New
    ///
    /// Creates a new `ArbitrageDetector` with specified profit and volume thresholds.
    ///
    /// ### Arguments
    ///
    /// - `min_profit_threshold`: A `f64` representing the minimum profit percentage.
    /// - `min_volume_threshold`: A `f64` representing the minimum trade volume.
    pub fn new(min_profit_threshold: f64, min_volume_threshold: f64) -> Self {
        Self {
            min_profit_threshold,
            min_volume_threshold,
        }
    }

    /// ## Detect Opportunities
    ///
    /// Detects simple arbitrage opportunities by comparing the best bid and ask prices across
    /// multiple exchanges for a given trading pair.
    ///
    /// ### Arguments
    ///
    /// - `summaries`: A `HashMap` where the key is a `TradingPair` and the value is a `Vec`
    ///   of `Summary` objects from different exchanges.
    ///
    /// ### Returns
    ///
    /// A `Vec` of `ArbitrageOpportunity` structs, each representing a profitable arbitrage
    /// opportunity.
    pub async fn detect_opportunities(
        &self,
        summaries: &HashMap<TradingPair, Vec<Summary>>,
    ) -> Vec<ArbitrageOpportunity> {
        let mut opportunities = Vec::new();

        for (pair, exchange_summaries) in summaries {
            if exchange_summaries.len() < 2 {
                continue; // Need at least 2 exchanges for arbitrage
            }

            // Find the best bid and ask across all exchanges
            let mut best_bid: Option<(&Summary, f64)> = None;
            let mut best_ask: Option<(&Summary, f64)> = None;

            for summary in exchange_summaries {
                if let Some(bid) = summary.bids.first() {
                    if best_bid.is_none() || bid.price > best_bid.unwrap().1 {
                        best_bid = Some((summary, bid.price));
                    }
                }

                if let Some(ask) = summary.asks.first() {
                    if best_ask.is_none() || ask.price < best_ask.unwrap().1 {
                        best_ask = Some((summary, ask.price));
                    }
                }
            }

            // Check if there's an arbitrage opportunity
            if let (Some((bid_summary, bid_price)), Some((ask_summary, ask_price))) =
                (best_bid, best_ask)
            {
                if bid_price > ask_price {
                    let profit = bid_price - ask_price;
                    let profit_percentage = (profit / ask_price) * 100.0;

                    if profit_percentage >= self.min_profit_threshold {
                        // Calculate available volume
                        let bid_volume =
                            bid_summary.bids.first().map(|b| b.quantity).unwrap_or(0.0);
                        let ask_volume =
                            ask_summary.asks.first().map(|a| a.quantity).unwrap_or(0.0);
                        let available_volume = bid_volume.min(ask_volume);

                        if available_volume >= self.min_volume_threshold {
                            opportunities.push(ArbitrageOpportunity {
                                buy_exchange: ask_summary.asks.first().unwrap().exchange.clone(),
                                sell_exchange: bid_summary.bids.first().unwrap().exchange.clone(),
                                symbol: pair.to_string(),
                                buy_price: ask_price,
                                sell_price: bid_price,
                                profit_percentage,
                                volume: available_volume,
                                timestamp: Utc::now(),
                            });
                        }
                    }
                }
            }
        }

        opportunities
    }

    /// ## Detect Triangular Arbitrage
    ///
    /// Placeholder for detecting triangular arbitrage opportunities. This involves finding
    /// price discrepancies between three different trading pairs.
    ///
    /// ### Arguments
    ///
    /// - `_summaries`: A `HashMap` of market summaries.
    ///
    /// ### Returns
    ///
    /// An empty `Vec` as this feature is not yet implemented.
    pub async fn detect_triangular_arbitrage(
        &self,
        _summaries: &HashMap<TradingPair, Vec<Summary>>,
    ) -> Vec<ArbitrageOpportunity> {
        // TODO: Implement triangular arbitrage detection
        // This would look for opportunities like BTC/USD -> ETH/BTC -> USD/ETH
        vec![]
    }

    /// ## Detect Negative Cycles
    ///
    /// Placeholder for detecting arbitrage opportunities using the Bellman-Ford algorithm
    /// to find negative cycles in the exchange graph.
    ///
    /// ### Arguments
    ///
    /// - `_summaries`: A `HashMap` of market summaries.
    ///
    /// ### Returns
    ///
    /// An empty `Vec` as this feature is not yet implemented.
    pub async fn detect_negative_cycles(
        &self,
        _summaries: &HashMap<TradingPair, Vec<Summary>>,
    ) -> Vec<ArbitrageOpportunity> {
        // TODO: Implement negative cycle detection using Bellman-Ford algorithm
        // This is useful for detecting complex arbitrage paths
        vec![]
    }
}

impl Default for ArbitrageDetector {
    fn default() -> Self {
        Self::new(0.1, 0.01) // 0.1% profit threshold, 0.01 volume threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aggregator_core::{Exchange, PriceLevel};

    #[tokio::test]
    async fn test_simple_arbitrage_detection() {
        let detector = ArbitrageDetector::new(0.1, 0.01);
        let mut summaries = HashMap::new();

        let pair = TradingPair::new("BTC", "USDT");

        // Create mock summaries with arbitrage opportunity
        let summary1 = Summary {
            symbol: "BTCUSDT".to_string(),
            spread: 1.0,
            bids: vec![PriceLevel {
                price: 50000.0,
                quantity: 1.0,
                exchange: Exchange::Binance,
                timestamp: Utc::now(),
            }],
            asks: vec![PriceLevel {
                price: 50001.0,
                quantity: 1.0,
                exchange: Exchange::Binance,
                timestamp: Utc::now(),
            }],
            timestamp: Utc::now(),
        };

        let summary2 = Summary {
            symbol: "BTCUSDT".to_string(),
            spread: 1.0,
            bids: vec![PriceLevel {
                price: 49900.0,
                quantity: 1.0,
                exchange: Exchange::Bybit,
                timestamp: Utc::now(),
            }],
            asks: vec![PriceLevel {
                price: 49950.0, // Lower ask price creates arbitrage opportunity
                quantity: 1.0,
                exchange: Exchange::Bybit,
                timestamp: Utc::now(),
            }],
            timestamp: Utc::now(),
        };

        summaries.insert(pair, vec![summary1, summary2]);

        let opportunities = detector.detect_opportunities(&summaries).await;
        assert!(!opportunities.is_empty());

        let opportunity = &opportunities[0];
        assert_eq!(opportunity.buy_exchange, Exchange::Bybit);
        assert_eq!(opportunity.sell_exchange, Exchange::Binance);
        assert!(opportunity.profit_percentage > 0.1);
    }
}
