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
