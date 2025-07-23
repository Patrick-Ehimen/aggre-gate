//! Test to verify the test infrastructure is working correctly

mod common;

use aggregator_core::Exchange;
use common::{assert_arbitrage_opportunity, assert_no_arbitrage_opportunities, TestDataFactory};

#[tokio::test]
async fn test_infrastructure_basic_functionality() {
    // Test TestDataFactory
    let summary =
        TestDataFactory::create_summary("BTCUSDT", Exchange::Binance, 50000.0, 50100.0, 1.0, 1.0);

    assert_eq!(summary.symbol, "BTCUSDT");
    assert_eq!(summary.bids.len(), 1);
    assert_eq!(summary.asks.len(), 1);
    assert_eq!(summary.bids[0].price, 50000.0);
    assert_eq!(summary.asks[0].price, 50100.0);
    assert_eq!(summary.spread, 100.0);
}

#[tokio::test]
async fn test_arbitrage_scenario_creation() {
    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        Exchange::Bybit,   // Buy here (lower price)
        Exchange::Binance, // Sell here (higher price)
        49950.0,           // Buy price
        50050.0,           // Sell price
        1.0,               // Volume
    );

    assert_eq!(scenarios.len(), 1);
    let summaries = scenarios.values().next().unwrap();
    assert_eq!(summaries.len(), 2);
}

#[tokio::test]
async fn test_no_arbitrage_scenario() {
    let scenarios = TestDataFactory::create_no_arbitrage_scenario("BTCUSDT");

    assert_eq!(scenarios.len(), 1);
    let summaries = scenarios.values().next().unwrap();
    assert_eq!(summaries.len(), 2);

    // Verify no arbitrage exists (first exchange ask > second exchange bid)
    let summary1 = &summaries[0];
    let summary2 = &summaries[1];

    let ask1 = summary1.asks.first().unwrap().price;
    let bid2 = summary2.bids.first().unwrap().price;

    assert!(ask1 > bid2, "Should not have arbitrage opportunity");
}

#[tokio::test]
async fn test_assertion_helpers() {
    // Test empty opportunities assertion
    let empty_opportunities = vec![];
    assert_no_arbitrage_opportunities(&empty_opportunities);

    // This test verifies our assertion helpers work without actually
    // creating arbitrage opportunities (which would require the full detector)
}

#[tokio::test]
async fn test_edge_case_scenarios() {
    // Test empty scenario
    let empty = TestDataFactory::create_empty_scenario();
    assert!(empty.is_empty());

    // Test single exchange scenario
    let single = TestDataFactory::create_single_exchange_scenario("BTCUSDT");
    assert_eq!(single.len(), 1);
    let summaries = single.values().next().unwrap();
    assert_eq!(summaries.len(), 1);

    // Test missing bid scenario
    let missing_bid = TestDataFactory::create_missing_bid_scenario("BTCUSDT");
    assert!(missing_bid.bids.is_empty());
    assert!(!missing_bid.asks.is_empty());

    // Test missing ask scenario
    let missing_ask = TestDataFactory::create_missing_ask_scenario("BTCUSDT");
    assert!(!missing_ask.bids.is_empty());
    assert!(missing_ask.asks.is_empty());

    // Test zero quantity scenario
    let zero_qty = TestDataFactory::create_zero_quantity_scenario("BTCUSDT");
    assert_eq!(zero_qty.bids[0].quantity, 0.0);
    assert_eq!(zero_qty.asks[0].quantity, 0.0);
}

#[tokio::test]
async fn test_realistic_market_scenario() {
    let market = TestDataFactory::create_realistic_market_scenario("BTCUSDT");

    assert_eq!(market.len(), 1);
    let summaries = market.values().next().unwrap();
    assert_eq!(summaries.len(), 3); // Binance, Bybit, Coinbase

    // Verify each summary has multiple price levels
    for summary in summaries {
        assert!(summary.bids.len() >= 3);
        assert!(summary.asks.len() >= 3);
    }
}

#[tokio::test]
async fn test_config_creation() {
    let default_config = TestDataFactory::create_test_config();
    assert_eq!(default_config.profit_threshold, 0.1);
    assert_eq!(default_config.volume_threshold, 0.01);

    let custom_config = TestDataFactory::create_custom_test_config(0.5, 0.1, 0.001, 200);
    assert_eq!(custom_config.profit_threshold, 0.5);
    assert_eq!(custom_config.volume_threshold, 0.1);
    assert_eq!(custom_config.time_tolerance.as_millis(), 200);
}
