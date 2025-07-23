//! Tests for ArbitrageDetector constructor and configuration

mod common;

use analysis_tools::ArbitrageDetector;
use common::{assert_no_arbitrage_opportunities, TestDataFactory};

#[tokio::test]
async fn test_arbitrage_detector_new_with_custom_thresholds() {
    // Test creating ArbitrageDetector with custom thresholds
    let profit_threshold = 0.5; // 0.5%
    let volume_threshold = 0.1; // 0.1 units

    let detector = ArbitrageDetector::new(profit_threshold, volume_threshold);

    // Create a scenario with profit below the custom threshold
    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,   // Buy here
        aggregator_core::Exchange::Binance, // Sell here
        50000.0,                            // Buy price
        50020.0,                            // Sell price (0.04% profit - below 0.5% threshold)
        1.0,                                // Volume above threshold
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;

    // Should find no opportunities because profit is below custom threshold
    assert_no_arbitrage_opportunities(&opportunities);
}

#[tokio::test]
async fn test_arbitrage_detector_new_with_custom_thresholds_above_threshold() {
    // Test that custom thresholds work when profit is above threshold
    let profit_threshold = 0.1; // 0.1%
    let volume_threshold = 0.01; // 0.01 units

    let detector = ArbitrageDetector::new(profit_threshold, volume_threshold);

    // Create a scenario with profit above the custom threshold
    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,   // Buy here
        aggregator_core::Exchange::Binance, // Sell here
        50000.0,                            // Buy price
        50100.0,                            // Sell price (0.2% profit - above 0.1% threshold)
        1.0,                                // Volume above threshold
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;

    // Should find opportunities because profit is above custom threshold
    assert_eq!(opportunities.len(), 1);
    let opportunity = &opportunities[0];
    assert!(opportunity.profit_percentage >= profit_threshold);
    assert!(opportunity.volume >= volume_threshold);
}

#[tokio::test]
async fn test_arbitrage_detector_new_with_volume_threshold() {
    // Test that volume threshold is properly applied
    let profit_threshold = 0.1; // 0.1%
    let volume_threshold = 2.0; // 2.0 units (high threshold)

    let detector = ArbitrageDetector::new(profit_threshold, volume_threshold);

    // Create a scenario with high profit but low volume
    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,   // Buy here
        aggregator_core::Exchange::Binance, // Sell here
        50000.0,                            // Buy price
        50500.0,                            // Sell price (1% profit - above threshold)
        1.0,                                // Volume below threshold (1.0 < 2.0)
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;

    // Should find no opportunities because volume is below threshold
    assert_no_arbitrage_opportunities(&opportunities);
}

#[tokio::test]
async fn test_arbitrage_detector_default_implementation() {
    // Test that default implementation uses expected thresholds
    let detector = ArbitrageDetector::default();

    // Create a scenario with profit just above default threshold (0.1%)
    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,   // Buy here
        aggregator_core::Exchange::Binance, // Sell here
        50000.0,                            // Buy price
        50060.0, // Sell price (0.12% profit - above default 0.1% threshold)
        0.02,    // Volume above default threshold (0.02 > 0.01)
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;

    // Should find opportunities with default thresholds
    assert_eq!(opportunities.len(), 1);
    let opportunity = &opportunities[0];
    assert!(opportunity.profit_percentage >= 0.1); // Default profit threshold
    assert!(opportunity.volume >= 0.01); // Default volume threshold
}

#[tokio::test]
async fn test_arbitrage_detector_default_profit_threshold() {
    // Test that default profit threshold (0.1%) is enforced
    let detector = ArbitrageDetector::default();

    // Create a scenario with profit just below default threshold
    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,   // Buy here
        aggregator_core::Exchange::Binance, // Sell here
        50000.0,                            // Buy price
        50040.0, // Sell price (0.08% profit - below default 0.1% threshold)
        1.0,     // Volume above threshold
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;

    // Should find no opportunities because profit is below default threshold
    assert_no_arbitrage_opportunities(&opportunities);
}

#[tokio::test]
async fn test_arbitrage_detector_default_volume_threshold() {
    // Test that default volume threshold (0.01) is enforced
    let detector = ArbitrageDetector::default();

    // Create a scenario with high profit but volume below default threshold
    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,   // Buy here
        aggregator_core::Exchange::Binance, // Sell here
        50000.0,                            // Buy price
        50500.0,                            // Sell price (1% profit - well above threshold)
        0.005,                              // Volume below default threshold (0.005 < 0.01)
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;

    // Should find no opportunities because volume is below default threshold
    assert_no_arbitrage_opportunities(&opportunities);
}

#[tokio::test]
async fn test_threshold_storage_and_usage_consistency() {
    // Test that thresholds are stored and used consistently
    let custom_profit = 0.25; // 0.25%
    let custom_volume = 0.5; // 0.5 units

    let detector = ArbitrageDetector::new(custom_profit, custom_volume);

    // Test multiple scenarios to ensure thresholds are consistently applied

    // Scenario 1: Both thresholds met
    let scenarios1 = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50150.0, // Sell price (0.3% profit - above 0.25% threshold)
        0.6,     // Volume above 0.5 threshold
    );

    let opportunities1 = detector.detect_opportunities(&scenarios1).await;
    assert_eq!(
        opportunities1.len(),
        1,
        "Should find opportunity when both thresholds are met"
    );

    // Scenario 2: Profit threshold not met
    let scenarios2 = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50100.0, // Sell price (0.2% profit - below 0.25% threshold)
        0.6,     // Volume above threshold
    );

    let opportunities2 = detector.detect_opportunities(&scenarios2).await;
    assert_no_arbitrage_opportunities(&opportunities2);

    // Scenario 3: Volume threshold not met
    let scenarios3 = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50200.0, // Sell price (0.4% profit - above threshold)
        0.3,     // Volume below 0.5 threshold
    );

    let opportunities3 = detector.detect_opportunities(&scenarios3).await;
    assert_no_arbitrage_opportunities(&opportunities3);
}

#[tokio::test]
async fn test_extreme_threshold_values() {
    // Test with very high thresholds
    let detector_high = ArbitrageDetector::new(10.0, 100.0); // 10% profit, 100 volume

    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        52000.0, // Sell price (4% profit - below 10% threshold)
        50.0,    // Volume below 100 threshold
    );

    let opportunities = detector_high.detect_opportunities(&scenarios).await;
    assert_no_arbitrage_opportunities(&opportunities);

    // Test with very low thresholds
    let detector_low = ArbitrageDetector::new(0.001, 0.0001); // 0.001% profit, 0.0001 volume

    let scenarios_low = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50001.0, // Sell price (0.002% profit - above 0.001% threshold)
        0.001,   // Volume above 0.0001 threshold
    );

    let opportunities_low = detector_low.detect_opportunities(&scenarios_low).await;
    assert_eq!(
        opportunities_low.len(),
        1,
        "Should find opportunity with very low thresholds"
    );
}

#[tokio::test]
async fn test_zero_thresholds() {
    // Test with zero thresholds (should accept any positive profit and volume)
    let detector = ArbitrageDetector::new(0.0, 0.0);

    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50000.1, // Sell price (tiny profit)
        0.0001,  // Tiny volume
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;
    assert_eq!(
        opportunities.len(),
        1,
        "Should find opportunity with zero thresholds"
    );

    let opportunity = &opportunities[0];
    assert!(opportunity.profit_percentage > 0.0);
    assert!(opportunity.volume > 0.0);
}

// Core functionality tests for ArbitrageDetector

#[tokio::test]
async fn test_detect_opportunities_with_valid_arbitrage_scenarios() {
    // Test basic arbitrage detection with clear profit opportunity
    let detector = ArbitrageDetector::new(0.1, 0.01); // 0.1% profit, 0.01 volume

    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,   // Buy here (lower ask)
        aggregator_core::Exchange::Binance, // Sell here (higher bid)
        49950.0,                            // Buy price (ask on Bybit)
        50100.0,                            // Sell price (bid on Binance)
        1.0,                                // Volume
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;

    assert_eq!(
        opportunities.len(),
        1,
        "Should find exactly one arbitrage opportunity"
    );

    let opportunity = &opportunities[0];
    assert_eq!(opportunity.buy_exchange, aggregator_core::Exchange::Bybit);
    assert_eq!(
        opportunity.sell_exchange,
        aggregator_core::Exchange::Binance
    );
    assert_eq!(opportunity.symbol, "BTC/USDT"); // TradingPair format
    assert_eq!(opportunity.buy_price, 49950.0);
    assert_eq!(opportunity.sell_price, 50100.0);
    assert!(opportunity.profit_percentage > 0.1); // Above threshold
    assert_eq!(opportunity.volume, 1.0);
}

#[tokio::test]
async fn test_detect_opportunities_multiple_valid_scenarios() {
    // Test detection with multiple trading pairs having arbitrage opportunities
    let detector = ArbitrageDetector::new(0.1, 0.01);
    let mut all_scenarios = std::collections::HashMap::new();

    // BTC/USDT arbitrage
    let btc_scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50100.0, // Sell price (0.2% profit)
        1.0,
    );

    // ETH/USDT arbitrage
    let eth_scenarios = TestDataFactory::create_arbitrage_scenario(
        "ETHUSDT",
        aggregator_core::Exchange::Coinbase,
        aggregator_core::Exchange::Bybit,
        3000.0, // Buy price
        3015.0, // Sell price (0.5% profit)
        2.0,
    );

    // Merge scenarios
    all_scenarios.extend(btc_scenarios);
    all_scenarios.extend(eth_scenarios);

    let opportunities = detector.detect_opportunities(&all_scenarios).await;

    assert_eq!(
        opportunities.len(),
        2,
        "Should find opportunities for both trading pairs"
    );

    // Verify both opportunities are present
    let btc_opportunity = opportunities.iter().find(|op| op.symbol == "BTC/USDT");
    let eth_opportunity = opportunities.iter().find(|op| op.symbol == "ETH/USDT");

    assert!(
        btc_opportunity.is_some(),
        "Should find BTC arbitrage opportunity"
    );
    assert!(
        eth_opportunity.is_some(),
        "Should find ETH arbitrage opportunity"
    );

    // Verify profit percentages are above threshold
    assert!(btc_opportunity.unwrap().profit_percentage >= 0.1);
    assert!(eth_opportunity.unwrap().profit_percentage >= 0.1);
}

#[tokio::test]
async fn test_profit_threshold_filtering_behavior() {
    // Test that opportunities below profit threshold are filtered out
    let detector = ArbitrageDetector::new(0.5, 0.01); // High profit threshold: 0.5%

    // Create scenario with profit below threshold
    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50100.0, // Sell price (0.2% profit - below 0.5% threshold)
        1.0,
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;
    assert_no_arbitrage_opportunities(&opportunities);

    // Test with profit above threshold
    let scenarios_above = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50300.0, // Sell price (0.6% profit - above 0.5% threshold)
        1.0,
    );

    let opportunities_above = detector.detect_opportunities(&scenarios_above).await;
    assert_eq!(
        opportunities_above.len(),
        1,
        "Should find opportunity above profit threshold"
    );
    assert!(opportunities_above[0].profit_percentage >= 0.5);
}

#[tokio::test]
async fn test_profit_threshold_edge_cases() {
    // Test profit threshold filtering with edge cases
    let detector = ArbitrageDetector::new(0.1, 0.01);

    // Scenario exactly at threshold
    let scenarios_exact = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50050.0, // Sell price (exactly 0.1% profit)
        1.0,
    );

    let opportunities_exact = detector.detect_opportunities(&scenarios_exact).await;
    assert_eq!(
        opportunities_exact.len(),
        1,
        "Should find opportunity exactly at threshold"
    );
    assert!(opportunities_exact[0].profit_percentage >= 0.1);

    // Scenario just below threshold
    let scenarios_below = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50049.0, // Sell price (0.098% profit - just below threshold)
        1.0,
    );

    let opportunities_below = detector.detect_opportunities(&scenarios_below).await;
    assert_no_arbitrage_opportunities(&opportunities_below);
}

#[tokio::test]
async fn test_volume_threshold_filtering_behavior() {
    // Test that opportunities below volume threshold are filtered out
    let detector = ArbitrageDetector::new(0.1, 2.0); // High volume threshold: 2.0

    // Create scenario with volume below threshold
    let scenarios = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50500.0, // Sell price (1% profit - well above profit threshold)
        1.5,     // Volume below 2.0 threshold
    );

    let opportunities = detector.detect_opportunities(&scenarios).await;
    assert_no_arbitrage_opportunities(&opportunities);

    // Test with volume above threshold
    let scenarios_above = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50500.0, // Sell price (1% profit)
        2.5,     // Volume above 2.0 threshold
    );

    let opportunities_above = detector.detect_opportunities(&scenarios_above).await;
    assert_eq!(
        opportunities_above.len(),
        1,
        "Should find opportunity above volume threshold"
    );
    assert!(opportunities_above[0].volume >= 2.0);
}

#[tokio::test]
async fn test_volume_threshold_edge_cases() {
    // Test volume threshold filtering with edge cases
    let detector = ArbitrageDetector::new(0.1, 0.5);

    // Scenario exactly at volume threshold
    let scenarios_exact = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50500.0, // Sell price (1% profit)
        0.5,     // Volume exactly at threshold
    );

    let opportunities_exact = detector.detect_opportunities(&scenarios_exact).await;
    assert_eq!(
        opportunities_exact.len(),
        1,
        "Should find opportunity exactly at volume threshold"
    );
    assert!(opportunities_exact[0].volume >= 0.5);

    // Scenario just below volume threshold
    let scenarios_below = TestDataFactory::create_arbitrage_scenario(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        aggregator_core::Exchange::Binance,
        50000.0, // Buy price
        50500.0, // Sell price (1% profit)
        0.49,    // Volume just below threshold
    );

    let opportunities_below = detector.detect_opportunities(&scenarios_below).await;
    assert_no_arbitrage_opportunities(&opportunities_below);
}

#[tokio::test]
async fn test_multiple_exchange_arbitrage_detection() {
    // Test arbitrage detection across multiple exchanges (more than 2)
    let detector = ArbitrageDetector::new(0.1, 0.01);
    let mut summaries = std::collections::HashMap::new();
    let pair = aggregator_core::TradingPair::new("BTC", "USDT");

    // Create summaries for multiple exchanges with different price levels
    let binance_summary = TestDataFactory::create_summary(
        "BTCUSDT",
        aggregator_core::Exchange::Binance,
        50100.0, // bid (highest - best for selling)
        50101.0, // ask
        1.0,
        1.0,
    );

    let bybit_summary = TestDataFactory::create_summary(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        49950.0, // bid
        49951.0, // ask (lowest - best for buying)
        1.5,
        1.5,
    );

    let coinbase_summary = TestDataFactory::create_summary(
        "BTCUSDT",
        aggregator_core::Exchange::Coinbase,
        50000.0, // bid (middle)
        50001.0, // ask (middle)
        2.0,
        2.0,
    );

    summaries.insert(pair, vec![binance_summary, bybit_summary, coinbase_summary]);

    let opportunities = detector.detect_opportunities(&summaries).await;

    assert_eq!(
        opportunities.len(),
        1,
        "Should find one arbitrage opportunity across multiple exchanges"
    );

    let opportunity = &opportunities[0];
    assert_eq!(opportunity.buy_exchange, aggregator_core::Exchange::Bybit); // Lowest ask
    assert_eq!(
        opportunity.sell_exchange,
        aggregator_core::Exchange::Binance
    ); // Highest bid
    assert_eq!(opportunity.buy_price, 49951.0); // Bybit ask
    assert_eq!(opportunity.sell_price, 50100.0); // Binance bid

    // Verify profit calculation
    let expected_profit_percentage = ((50100.0 - 49951.0) / 49951.0) * 100.0;
    assert!((opportunity.profit_percentage - expected_profit_percentage).abs() < 0.001);

    // Volume should be minimum of bid and ask quantities
    assert_eq!(opportunity.volume, 1.0); // min(1.0 from Binance bid, 1.5 from Bybit ask)
}

#[tokio::test]
async fn test_multiple_exchange_no_arbitrage() {
    // Test multiple exchanges where no arbitrage opportunity exists
    let detector = ArbitrageDetector::new(0.1, 0.01);
    let mut summaries = std::collections::HashMap::new();
    let pair = aggregator_core::TradingPair::new("BTC", "USDT");

    // Create summaries where all exchanges have similar prices (no arbitrage)
    let binance_summary = TestDataFactory::create_summary(
        "BTCUSDT",
        aggregator_core::Exchange::Binance,
        50000.0, // bid
        50100.0, // ask
        1.0,
        1.0,
    );

    let bybit_summary = TestDataFactory::create_summary(
        "BTCUSDT",
        aggregator_core::Exchange::Bybit,
        50010.0, // bid (higher than Binance ask - no arbitrage)
        50110.0, // ask
        1.0,
        1.0,
    );

    let coinbase_summary = TestDataFactory::create_summary(
        "BTCUSDT",
        aggregator_core::Exchange::Coinbase,
        50005.0, // bid
        50105.0, // ask
        1.0,
        1.0,
    );

    summaries.insert(pair, vec![binance_summary, bybit_summary, coinbase_summary]);

    let opportunities = detector.detect_opportunities(&summaries).await;
    assert_no_arbitrage_opportunities(&opportunities);
}

#[tokio::test]
async fn test_multiple_exchange_complex_scenario() {
    // Test complex scenario with multiple exchanges and varying volumes
    let detector = ArbitrageDetector::new(0.1, 0.1); // Higher volume threshold
    let mut summaries = std::collections::HashMap::new();
    let pair = aggregator_core::TradingPair::new("ETH", "USDT");

    // Exchange 1: High bid, low volume
    let exchange1_summary = TestDataFactory::create_summary(
        "ETHUSDT",
        aggregator_core::Exchange::Binance,
        3100.0, // High bid (good for selling)
        3101.0, // ask
        0.05,   // Low volume (below threshold)
        1.0,
    );

    // Exchange 2: Low ask, high volume
    let exchange2_summary = TestDataFactory::create_summary(
        "ETHUSDT",
        aggregator_core::Exchange::Bybit,
        2990.0, // bid
        2995.0, // Low ask (good for buying)
        1.0,
        0.5, // High volume for ask
    );

    // Exchange 3: Medium prices, high volume
    let exchange3_summary = TestDataFactory::create_summary(
        "ETHUSDT",
        aggregator_core::Exchange::Coinbase,
        3050.0, // Medium bid
        3055.0, // Medium ask
        0.3,    // High volume for bid
        0.3,
    );

    summaries.insert(
        pair,
        vec![exchange1_summary, exchange2_summary, exchange3_summary],
    );

    let opportunities = detector.detect_opportunities(&summaries).await;

    // Should find no opportunities because the best bid (Binance) has insufficient volume
    // The algorithm finds globally best bid/ask, not the best combination that meets volume requirements
    assert_eq!(
        opportunities.len(),
        0,
        "Should find no opportunities because best bid has insufficient volume"
    );
}
