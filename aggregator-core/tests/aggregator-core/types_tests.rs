// aggregator-core/tests/aggregator-core/types_tests.rs
// Unit tests for types.rs

use aggregator_core::types::*;
use chrono::{TimeZone, Utc};
use std::str::FromStr;
use uuid::Uuid;

/**
 * @notice Tests the Display and FromStr implementations for the Exchange enum.
 * @dev Verifies that all Exchange variants can be converted to string and parsed back.
 */
#[test]
fn test_exchange_display_and_fromstr() {
    for ex in Exchange::all() {
        let s = ex.to_string();
        let parsed = Exchange::from_str(&s).unwrap();
        assert_eq!(ex, parsed);
    }
    assert!(Exchange::from_str("unknown").is_err());
}

/**
 * @notice Tests TradingPair::new and its Display implementation.
 * @dev Ensures base and quote are uppercased and formatted correctly.
 */
#[test]
fn test_trading_pair_new_and_display() {
    let pair = TradingPair::new("btc", "usd");
    assert_eq!(pair.base, "BTC");
    assert_eq!(pair.quote, "USD");
    assert_eq!(pair.to_string(), "BTC/USD");
}

/**
 * @notice Tests TradingPair::from_str for valid and invalid input.
 * @dev Checks parsing for correct and incorrect trading pair formats.
 */
#[test]
fn test_trading_pair_fromstr() {
    let pair = TradingPair::from_str("ETH/EUR").unwrap();
    assert_eq!(pair.base, "ETH");
    assert_eq!(pair.quote, "EUR");
    assert!(TradingPair::from_str("BADFORMAT").is_err());
}

/**
 * @notice Tests ordering for Bid: higher price is better.
 * @dev Ensures that higher price bids are considered better (less in ordering).
 */
#[test]
fn test_bid_ordering() {
    let b1 = Bid {
        price: 100.0,
        ..Default::default()
    };
    let b2 = Bid {
        price: 101.0,
        ..Default::default()
    };
    assert!(b2 < b1); // Higher price is better for bids
}

/**
 * @notice Tests ordering for Ask: lower price is better.
 * @dev Ensures that lower price asks are considered better (less in ordering).
 */
#[test]
fn test_ask_ordering() {
    let a1 = Ask {
        price: 100.0,
        ..Default::default()
    };
    let a2 = Ask {
        price: 99.0,
        ..Default::default()
    };
    assert!(a2 < a1); // Lower price is better for asks
}

/**
 * @notice Tests the PriceLevel struct fields.
 * @dev Verifies correct assignment of all PriceLevel fields.
 */
#[test]
fn test_price_level() {
    let now = Utc::now();
    let pl = PriceLevel {
        price: 123.4,
        quantity: 1.5,
        exchange: Exchange::Binance,
        timestamp: now,
    };
    assert_eq!(pl.price, 123.4);
    assert_eq!(pl.quantity, 1.5);
    assert_eq!(pl.exchange, Exchange::Binance);
    assert_eq!(pl.timestamp, now);
}

/**
 * @notice Tests the PriceLevelUpdate struct fields and construction.
 * @dev Ensures all fields are set and vectors are initialized.
 */
#[test]
fn test_price_level_update() {
    let id = Uuid::new_v4();
    let now = Utc::now();
    let plu = PriceLevelUpdate {
        id,
        symbol: "BTCUSD".to_string(),
        exchange: Exchange::Kraken,
        bids: vec![Bid::default()],
        asks: vec![Ask::default()],
        timestamp: now,
    };
    assert_eq!(plu.id, id);
    assert_eq!(plu.symbol, "BTCUSD");
    assert_eq!(plu.exchange, Exchange::Kraken);
    assert_eq!(plu.bids.len(), 1);
    assert_eq!(plu.asks.len(), 1);
    assert_eq!(plu.timestamp, now);
}

/**
 * @notice Tests the Summary struct fields and construction.
 * @dev Verifies correct assignment and vector lengths for Summary.
 */
#[test]
fn test_summary() {
    let now = Utc::now();
    let s = Summary {
        symbol: "ETHUSD".to_string(),
        spread: 0.5,
        bids: vec![PriceLevel {
            price: 100.0,
            quantity: 2.0,
            exchange: Exchange::OKX,
            timestamp: now,
        }],
        asks: vec![],
        timestamp: now,
    };
    assert_eq!(s.symbol, "ETHUSD");
    assert_eq!(s.spread, 0.5);
    assert_eq!(s.bids.len(), 1);
    assert_eq!(s.asks.len(), 0);
    assert_eq!(s.timestamp, now);
}

/**
 * @notice Tests the default implementation for OrderBookDepth.
 * @dev Checks that default values are as expected.
 */
#[test]
fn test_order_book_depth_default() {
    let d = OrderBookDepth::default();
    assert_eq!(d.levels, 20);
    assert_eq!(d.market_type, MarketType::Spot);
}

/**
 * @notice Tests the ArbitrageOpportunity struct fields and construction.
 * @dev Verifies correct assignment of all ArbitrageOpportunity fields.
 */
#[test]
fn test_arbitrage_opportunity() {
    let now = Utc::now();
    let arb = ArbitrageOpportunity {
        buy_exchange: Exchange::Binance,
        sell_exchange: Exchange::Bitstamp,
        symbol: "BTCUSD".to_string(),
        buy_price: 100.0,
        sell_price: 105.0,
        profit_percentage: 5.0,
        volume: 1.0,
        timestamp: now,
    };
    assert_eq!(arb.buy_exchange, Exchange::Binance);
    assert_eq!(arb.sell_exchange, Exchange::Bitstamp);
    assert_eq!(arb.symbol, "BTCUSD");
    assert_eq!(arb.buy_price, 100.0);
    assert_eq!(arb.sell_price, 105.0);
    assert_eq!(arb.profit_percentage, 5.0);
    assert_eq!(arb.volume, 1.0);
    assert_eq!(arb.timestamp, now);
}

/**
 * @notice Tests the HealthStatus struct fields and construction.
 * @dev Ensures correct assignment and None for error_message.
 */
#[test]
fn test_health_status() {
    let now = Utc::now();
    let hs = HealthStatus {
        exchange: Exchange::Bybit,
        is_healthy: true,
        last_update: now,
        error_message: None,
    };
    assert_eq!(hs.exchange, Exchange::Bybit);
    assert!(hs.is_healthy);
    assert_eq!(hs.last_update, now);
    assert!(hs.error_message.is_none());
}

/**
 * @notice Tests the Metrics struct fields and construction.
 * @dev Verifies correct assignment of all Metrics fields.
 */
#[test]
fn test_metrics() {
    let now = Utc::now();
    let m = Metrics {
        exchange: Exchange::Coinbase,
        symbol: "BTCUSD".to_string(),
        updates_per_second: 10.0,
        latency_ms: 5.0,
        error_count: 0,
        last_update: now,
    };
    assert_eq!(m.exchange, Exchange::Coinbase);
    assert_eq!(m.symbol, "BTCUSD");
    assert_eq!(m.updates_per_second, 10.0);
    assert_eq!(m.latency_ms, 5.0);
    assert_eq!(m.error_count, 0);
    assert_eq!(m.last_update, now);
}
