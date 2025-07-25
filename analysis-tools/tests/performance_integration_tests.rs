//! Performance and integration tests for analysis-tools module
//!
//! This module contains comprehensive tests for:
//! - Processing time with realistic data volumes
//! - Large dataset handling within acceptable time limits
//! - Integration with aggregator_core types and data consistency
//! - Real-world-like market data scenarios

mod common;

use aggregator_core::{Exchange, PriceLevel, Summary, TradingPair};
use analysis_tools::{AnalysisEngine, ArbitrageDetector, DefaultAnalysisEngine};
use chrono::Utc;
use common::{
    assert_all_opportunities_meet_thresholds, assert_no_arbitrage_opportunities,
    assert_opportunities_sorted_by_profit, assert_recent_timestamp, assert_unique_exchange_pairs,
    TestDataFactory,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::timeout;

/// Performance test configuration
const PERFORMANCE_TIMEOUT: Duration = Duration::from_secs(5);
const LARGE_DATASET_SIZE: usize = 1000;
const REALISTIC_DATASET_SIZE: usize = 100;
const MAX_PROCESSING_TIME_MS: u128 = 1000; // 1 second max for realistic datasets

#[tokio::test]
async fn test_processing_time_with_realistic_data_volumes() {
    // Test ArbitrageDetector processing time with realistic market data volumes
    let detector = ArbitrageDetector::new(0.1, 0.01);
    let mut summaries = HashMap::new();

    // Create realistic dataset with multiple trading pairs and exchanges
    let trading_pairs = vec![
        ("BTC", "USDT"),
        ("ETH", "USDT"),
        ("BNB", "USDT"),
        ("ADA", "USDT"),
        ("SOL", "USDT"),
        ("DOT", "USDT"),
        ("AVAX", "USDT"),
        ("MATIC", "USDT"),
        ("LINK", "USDT"),
        ("UNI", "USDT"),
    ];

    let exchanges = vec![
        Exchange::Binance,
        Exchange::Bybit,
        Exchange::Coinbase,
        Exchange::Kraken,
        Exchange::OKX,
    ];

    // Generate realistic market data for each pair across all exchanges
    for (base, quote) in &trading_pairs {
        let pair = TradingPair::new(base, quote);
        let symbol = format!("{}{}", base, quote);
        let mut pair_summaries = Vec::new();

        let base_price = match *base {
            "BTC" => 50000.0,
            "ETH" => 3000.0,
            "BNB" => 300.0,
            "ADA" => 0.5,
            "SOL" => 100.0,
            "DOT" => 20.0,
            "AVAX" => 40.0,
            "MATIC" => 1.0,
            "LINK" => 15.0,
            "UNI" => 8.0,
            _ => 1.0,
        };

        for (i, exchange) in exchanges.iter().enumerate() {
            // Add slight price variations between exchanges
            let price_variation = (i as f64 - 2.0) * base_price * 0.001; // ±0.1% variation
            let bid_price = base_price + price_variation;
            let ask_price = bid_price + (base_price * 0.001); // 0.1% spread

            let summary = TestDataFactory::create_summary_with_depth(
                &symbol,
                exchange.clone(),
                vec![
                    (bid_price, 1.0),
                    (bid_price - base_price * 0.0001, 1.5),
                    (bid_price - base_price * 0.0002, 2.0),
                ],
                vec![
                    (ask_price, 1.2),
                    (ask_price + base_price * 0.0001, 1.8),
                    (ask_price + base_price * 0.0002, 2.5),
                ],
            );

            pair_summaries.push(summary);
        }

        summaries.insert(pair, pair_summaries);
    }

    // Measure processing time
    let start_time = Instant::now();
    let opportunities = detector.detect_opportunities(&summaries).await;
    let processing_time = start_time.elapsed();

    // Verify processing completed within acceptable time
    assert!(
        processing_time.as_millis() <= MAX_PROCESSING_TIME_MS,
        "Processing took {}ms, expected <= {}ms",
        processing_time.as_millis(),
        MAX_PROCESSING_TIME_MS
    );

    // Verify results are valid
    assert!(
        !opportunities.is_empty(),
        "Should find some arbitrage opportunities in realistic data"
    );

    // Verify all opportunities meet thresholds
    assert_all_opportunities_meet_thresholds(&opportunities, 0.1, 0.01);

    println!(
        "Processed {} trading pairs across {} exchanges in {}ms, found {} opportunities",
        trading_pairs.len(),
        exchanges.len(),
        processing_time.as_millis(),
        opportunities.len()
    );
}

#[tokio::test]
async fn test_analysis_engine_processing_time() {
    // Test DefaultAnalysisEngine processing time with realistic data
    let engine = DefaultAnalysisEngine::new();
    let mut summaries = HashMap::new();

    // Create realistic dataset for analysis engine (uses different input format)
    for i in 0..REALISTIC_DATASET_SIZE {
        let symbol = format!("PAIR{:03}USDT", i);
        let exchange_key = format!(
            "{}_{}",
            match i % 5 {
                0 => "binance",
                1 => "bybit",
                2 => "coinbase",
                3 => "kraken",
                _ => "okx",
            },
            symbol.to_lowercase()
        );

        let base_price = 1000.0 + (i as f64 * 10.0);
        let summary = TestDataFactory::create_summary_with_depth(
            &symbol,
            match i % 5 {
                0 => Exchange::Binance,
                1 => Exchange::Bybit,
                2 => Exchange::Coinbase,
                3 => Exchange::Kraken,
                _ => Exchange::OKX,
            },
            vec![(base_price, 1.0), (base_price - 1.0, 1.5)],
            vec![(base_price + 1.0, 1.2), (base_price + 2.0, 1.8)],
        );

        summaries.insert(exchange_key, summary);
    }

    // Test analyze_summaries performance
    let start_time = Instant::now();
    let opportunities = engine.analyze_summaries(&summaries).await.unwrap();
    let analysis_time = start_time.elapsed();

    assert!(
        analysis_time.as_millis() <= MAX_PROCESSING_TIME_MS,
        "Analysis took {}ms, expected <= {}ms",
        analysis_time.as_millis(),
        MAX_PROCESSING_TIME_MS
    );

    // Test individual method performance
    let test_summary = summaries.values().next().unwrap();

    let start_time = Instant::now();
    let _spread = engine.calculate_spread(test_summary).await;
    let spread_time = start_time.elapsed();

    let start_time = Instant::now();
    let _vwap = engine.calculate_volume_weighted_price(test_summary).await;
    let vwap_time = start_time.elapsed();

    assert!(
        spread_time.as_millis() <= 10,
        "Spread calculation took {}ms, expected <= 10ms",
        spread_time.as_millis()
    );

    assert!(
        vwap_time.as_millis() <= 10,
        "VWAP calculation took {}ms, expected <= 10ms",
        vwap_time.as_millis()
    );

    println!(
        "Analyzed {} summaries in {}ms, found {} opportunities",
        summaries.len(),
        analysis_time.as_millis(),
        opportunities.len()
    );
}

#[tokio::test]
async fn test_large_dataset_handling_within_time_limits() {
    // Test processing of large datasets within acceptable time limits
    let detector = ArbitrageDetector::new(0.1, 0.01);
    let mut summaries = HashMap::new();

    // Create large dataset with many trading pairs
    for i in 0..LARGE_DATASET_SIZE {
        let pair = TradingPair::new(&format!("TOKEN{:04}", i), "USDT");
        let symbol = format!("TOKEN{:04}USDT", i);
        let mut pair_summaries = Vec::new();

        let base_price = 1.0 + (i as f64 * 0.01);

        // Create summaries for 3 exchanges per pair
        for (j, exchange) in [Exchange::Binance, Exchange::Bybit, Exchange::Coinbase]
            .iter()
            .enumerate()
        {
            let price_offset = (j as f64 - 1.0) * base_price * 0.002; // Small price differences
            let summary = TestDataFactory::create_summary(
                &symbol,
                exchange.clone(),
                base_price + price_offset,
                base_price + price_offset + (base_price * 0.001),
                1.0,
                1.0,
            );
            pair_summaries.push(summary);
        }

        summaries.insert(pair, pair_summaries);
    }

    // Test with timeout to ensure it completes within reasonable time
    let result = timeout(PERFORMANCE_TIMEOUT, async {
        let start_time = Instant::now();
        let opportunities = detector.detect_opportunities(&summaries).await;
        let processing_time = start_time.elapsed();
        (opportunities, processing_time)
    })
    .await;

    assert!(
        result.is_ok(),
        "Large dataset processing timed out after {:?}",
        PERFORMANCE_TIMEOUT
    );

    let (opportunities, processing_time) = result.unwrap();

    // Verify processing time is reasonable for large dataset
    assert!(
        processing_time.as_secs() <= 3,
        "Large dataset processing took {}s, expected <= 3s",
        processing_time.as_secs()
    );

    // Verify results are valid
    assert_all_opportunities_meet_thresholds(&opportunities, 0.1, 0.01);
    assert_unique_exchange_pairs(&opportunities);

    println!(
        "Processed {} trading pairs ({} total summaries) in {}ms, found {} opportunities",
        LARGE_DATASET_SIZE,
        LARGE_DATASET_SIZE * 3,
        processing_time.as_millis(),
        opportunities.len()
    );
}

#[tokio::test]
async fn test_concurrent_processing_safety() {
    // Test that the system handles concurrent processing safely
    let detector = ArbitrageDetector::new(0.1, 0.01);

    // Create multiple datasets for concurrent processing
    let mut datasets = Vec::new();
    for dataset_id in 0..5 {
        let mut summaries = HashMap::new();

        for i in 0..20 {
            let pair = TradingPair::new(&format!("PAIR{:02}", i), "USDT");
            let symbol = format!("PAIR{:02}USDT", i);
            let base_price = 100.0 + (dataset_id as f64 * 10.0) + (i as f64);

            let summary1 = TestDataFactory::create_summary(
                &symbol,
                Exchange::Binance,
                base_price,
                base_price + 1.0,
                1.0,
                1.0,
            );

            let summary2 = TestDataFactory::create_summary(
                &symbol,
                Exchange::Bybit,
                base_price + 0.5,
                base_price + 1.5,
                1.0,
                1.0,
            );

            summaries.insert(pair, vec![summary1, summary2]);
        }

        datasets.push(summaries);
    }

    // Process all datasets concurrently
    let start_time = Instant::now();
    let tasks: Vec<_> = datasets
        .into_iter()
        .map(|dataset| {
            let detector_ref = &detector;
            async move { detector_ref.detect_opportunities(&dataset).await }
        })
        .collect();

    let results = futures::future::join_all(tasks).await;
    let concurrent_time = start_time.elapsed();

    // Verify all tasks completed successfully
    assert_eq!(results.len(), 5, "All concurrent tasks should complete");

    // Verify results are valid
    for opportunities in &results {
        assert_all_opportunities_meet_thresholds(opportunities, 0.1, 0.01);
    }

    // Verify concurrent processing is reasonably fast
    assert!(
        concurrent_time.as_millis() <= 2000,
        "Concurrent processing took {}ms, expected <= 2000ms",
        concurrent_time.as_millis()
    );

    let total_opportunities: usize = results.iter().map(|r| r.len()).sum();
    println!(
        "Processed 5 datasets concurrently in {}ms, found {} total opportunities",
        concurrent_time.as_millis(),
        total_opportunities
    );
}

#[tokio::test]
async fn test_integration_with_aggregator_core_types() {
    // Test integration with all aggregator_core types and data consistency
    let detector = ArbitrageDetector::new(0.1, 0.01);
    let engine = DefaultAnalysisEngine::new();

    // Test with all Exchange variants
    let all_exchanges = Exchange::all();
    let mut summaries_detector = HashMap::new();
    let mut summaries_engine = HashMap::new();

    for (i, exchange) in all_exchanges.iter().enumerate() {
        let pair = TradingPair::new("BTC", "USDT");
        let symbol = "BTCUSDT";
        let base_price = 50000.0 + (i as f64 * 10.0);

        // Create PriceLevel instances with proper timestamps
        let timestamp = Utc::now();
        let bid_level = PriceLevel {
            price: base_price,
            quantity: 1.0 + (i as f64 * 0.1),
            exchange: exchange.clone(),
            timestamp,
        };

        let ask_level = PriceLevel {
            price: base_price + 50.0,
            quantity: 1.2 + (i as f64 * 0.1),
            exchange: exchange.clone(),
            timestamp,
        };

        // Create Summary with proper structure
        let summary = Summary {
            symbol: symbol.to_string(),
            spread: ask_level.price - bid_level.price,
            bids: vec![bid_level],
            asks: vec![ask_level],
            timestamp,
        };

        // Add to detector summaries (grouped by TradingPair)
        summaries_detector
            .entry(pair.clone())
            .or_insert_with(Vec::new)
            .push(summary.clone());

        // Add to engine summaries (keyed by string)
        let key = format!("{}_{}", exchange.to_string(), symbol.to_lowercase());
        summaries_engine.insert(key, summary);
    }

    // Test ArbitrageDetector with aggregator_core types
    let opportunities = detector.detect_opportunities(&summaries_detector).await;

    // Verify type consistency
    for opportunity in &opportunities {
        // Verify Exchange enum variants are preserved
        assert!(all_exchanges.contains(&opportunity.buy_exchange));
        assert!(all_exchanges.contains(&opportunity.sell_exchange));

        // Verify symbol format matches TradingPair::Display
        assert_eq!(opportunity.symbol, "BTC/USDT");

        // Verify timestamp is recent and valid
        assert_recent_timestamp(opportunity, Duration::from_secs(10));

        // Verify price and volume are valid numbers
        assert!(opportunity.buy_price > 0.0 && opportunity.buy_price.is_finite());
        assert!(opportunity.sell_price > 0.0 && opportunity.sell_price.is_finite());
        assert!(opportunity.volume > 0.0 && opportunity.volume.is_finite());
        assert!(opportunity.profit_percentage.is_finite());
    }

    // Test DefaultAnalysisEngine with aggregator_core types
    let engine_opportunities = engine.analyze_summaries(&summaries_engine).await.unwrap();

    // Verify engine results maintain type consistency
    for opportunity in &engine_opportunities {
        assert!(all_exchanges.contains(&opportunity.buy_exchange));
        assert!(all_exchanges.contains(&opportunity.sell_exchange));
        assert_recent_timestamp(opportunity, Duration::from_secs(10));
    }

    // Test individual engine methods
    let test_summary = summaries_engine.values().next().unwrap();

    let spread = engine.calculate_spread(test_summary).await;
    assert!(spread.is_some());
    assert!(spread.unwrap() > 0.0 && spread.unwrap().is_finite());

    let vwap = engine.calculate_volume_weighted_price(test_summary).await;
    assert!(vwap.is_some());
    assert!(vwap.unwrap() > 0.0 && vwap.unwrap().is_finite());

    println!(
        "Integration test: ArbitrageDetector found {} opportunities, AnalysisEngine found {} opportunities",
        opportunities.len(),
        engine_opportunities.len()
    );
}

#[tokio::test]
async fn test_data_consistency_across_components() {
    // Test that data remains consistent when passed between different components
    let detector = ArbitrageDetector::new(0.1, 0.01);
    let engine = DefaultAnalysisEngine::new();

    // Create identical market data for both components
    let market_data = TestDataFactory::create_realistic_market_scenario("BTCUSDT");

    // Convert for engine format
    let mut engine_summaries = HashMap::new();
    for (pair, summaries) in &market_data {
        for (i, summary) in summaries.iter().enumerate() {
            let key = format!("exchange{}_{}", i, pair.to_string().to_lowercase());
            engine_summaries.insert(key, summary.clone());
        }
    }

    // Process with both components
    let detector_opportunities = detector.detect_opportunities(&market_data).await;
    let engine_opportunities = engine.analyze_summaries(&engine_summaries).await.unwrap();

    // Verify data consistency between components
    if !detector_opportunities.is_empty() && !engine_opportunities.is_empty() {
        // Both should find opportunities in the same market data
        let detector_symbols: std::collections::HashSet<_> =
            detector_opportunities.iter().map(|o| &o.symbol).collect();
        let engine_symbols: std::collections::HashSet<_> =
            engine_opportunities.iter().map(|o| &o.symbol).collect();

        // Should have some overlap in detected symbols
        let intersection: Vec<_> = detector_symbols.intersection(&engine_symbols).collect();
        assert!(
            !intersection.is_empty(),
            "Both components should detect opportunities in the same symbols"
        );
    }

    // Test spread calculation consistency
    let test_summary = market_data.values().next().unwrap().first().unwrap();
    let engine_spread = engine.calculate_spread(test_summary).await;

    // Manual spread calculation for verification
    if let (Some(best_bid), Some(best_ask)) = (test_summary.bids.first(), test_summary.asks.first())
    {
        let manual_spread = best_ask.price - best_bid.price;
        if let Some(calculated_spread) = engine_spread {
            assert!(
                (calculated_spread - manual_spread).abs() < 0.001,
                "Spread calculation inconsistency: engine={}, manual={}",
                calculated_spread,
                manual_spread
            );
        }
    }

    println!(
        "Data consistency test: detector found {} opportunities, engine found {} opportunities",
        detector_opportunities.len(),
        engine_opportunities.len()
    );
}

#[tokio::test]
async fn test_real_world_market_data_scenarios() {
    // Test with realistic market conditions and scenarios
    let detector = ArbitrageDetector::new(0.05, 0.01); // Lower threshold for realistic markets
    let engine = DefaultAnalysisEngine::new();

    // Scenario 1: High volatility market (crypto bull run)
    let mut high_volatility_summaries = HashMap::new();
    let btc_pair = TradingPair::new("BTC", "USDT");

    let btc_summaries = vec![
        // Binance: Higher prices due to high demand
        TestDataFactory::create_summary_with_depth(
            "BTCUSDT",
            Exchange::Binance,
            vec![(52000.0, 0.5), (51950.0, 1.0), (51900.0, 1.5)],
            vec![(52050.0, 0.8), (52100.0, 1.2), (52150.0, 2.0)],
        ),
        // Bybit: Lower prices, potential arbitrage
        TestDataFactory::create_summary_with_depth(
            "BTCUSDT",
            Exchange::Bybit,
            vec![(51800.0, 1.0), (51750.0, 1.5), (51700.0, 2.0)],
            vec![(51850.0, 1.2), (51900.0, 1.8), (51950.0, 2.5)],
        ),
        // Coinbase: Premium pricing (typical for US market)
        TestDataFactory::create_summary_with_depth(
            "BTCUSDT",
            Exchange::Coinbase,
            vec![(52100.0, 0.3), (52050.0, 0.8), (52000.0, 1.2)],
            vec![(52150.0, 0.5), (52200.0, 1.0), (52250.0, 1.5)],
        ),
    ];

    high_volatility_summaries.insert(btc_pair, btc_summaries);

    // Scenario 2: Stable market with tight spreads
    let eth_pair = TradingPair::new("ETH", "USDT");
    let eth_summaries = vec![
        TestDataFactory::create_summary_with_depth(
            "ETHUSDT",
            Exchange::Binance,
            vec![(3000.0, 2.0), (2999.5, 3.0), (2999.0, 4.0)],
            vec![(3000.5, 2.5), (3001.0, 3.5), (3001.5, 5.0)],
        ),
        TestDataFactory::create_summary_with_depth(
            "ETHUSDT",
            Exchange::Bybit,
            vec![(2999.8, 2.2), (2999.3, 3.2), (2998.8, 4.2)],
            vec![(3000.3, 2.8), (3000.8, 3.8), (3001.3, 5.2)],
        ),
    ];

    high_volatility_summaries.insert(eth_pair, eth_summaries);

    // Test processing of realistic scenarios
    let start_time = Instant::now();
    let opportunities = detector
        .detect_opportunities(&high_volatility_summaries)
        .await;
    let processing_time = start_time.elapsed();

    // Verify realistic processing performance
    assert!(
        processing_time.as_millis() <= 100,
        "Real-world scenario processing took {}ms, expected <= 100ms",
        processing_time.as_millis()
    );

    // Verify opportunities are realistic
    for opportunity in &opportunities {
        // Profit should be reasonable for crypto markets (not too high)
        assert!(
            opportunity.profit_percentage <= 5.0,
            "Profit percentage {}% seems unrealistic for modern crypto markets",
            opportunity.profit_percentage
        );

        // Volume should be meaningful
        assert!(
            opportunity.volume >= 0.01,
            "Volume {} is too small for realistic trading",
            opportunity.volume
        );

        // Price differences should be reasonable
        let price_diff = opportunity.sell_price - opportunity.buy_price;
        let relative_diff = price_diff / opportunity.buy_price;
        assert!(
            relative_diff <= 0.05,
            "Price difference {}% is too large for realistic arbitrage",
            relative_diff * 100.0
        );
    }

    // Test engine with realistic data
    let mut engine_summaries = HashMap::new();
    for (pair, summaries) in &high_volatility_summaries {
        for (i, summary) in summaries.iter().enumerate() {
            let key = format!(
                "{}_{}",
                match i {
                    0 => "binance",
                    1 => "bybit",
                    _ => "coinbase",
                },
                pair.to_string().replace("/", "").to_lowercase()
            );
            engine_summaries.insert(key, summary.clone());
        }
    }

    let engine_opportunities = engine.analyze_summaries(&engine_summaries).await.unwrap();

    // Test spread calculations on realistic data
    for summary in engine_summaries.values() {
        let spread = engine.calculate_spread(summary).await;
        if let Some(spread_value) = spread {
            // Spreads should be reasonable for crypto markets
            let relative_spread = spread_value / summary.bids.first().unwrap().price;
            assert!(
                relative_spread <= 0.01,
                "Spread {}% is too wide for realistic crypto market",
                relative_spread * 100.0
            );
        }

        let vwap = engine.calculate_volume_weighted_price(summary).await;
        if let Some(vwap_value) = vwap {
            // VWAP should be within reasonable range of mid-price
            let mid_price =
                (summary.bids.first().unwrap().price + summary.asks.first().unwrap().price) / 2.0;
            let vwap_diff = (vwap_value - mid_price).abs() / mid_price;
            assert!(
                vwap_diff <= 0.005,
                "VWAP {} differs too much from mid-price {} ({}%)",
                vwap_value,
                mid_price,
                vwap_diff * 100.0
            );
        }
    }

    println!(
        "Real-world scenario test: processed in {}ms, detector found {} opportunities, engine found {} opportunities",
        processing_time.as_millis(),
        opportunities.len(),
        engine_opportunities.len()
    );
}

#[tokio::test]
async fn test_memory_usage_with_large_datasets() {
    // Test memory efficiency with large datasets
    let detector = ArbitrageDetector::new(0.1, 0.01);

    // Create progressively larger datasets and monitor processing
    let dataset_sizes = vec![100, 500, 1000];

    for size in dataset_sizes {
        let mut summaries = HashMap::new();

        // Create dataset of specified size
        for i in 0..size {
            let pair = TradingPair::new(&format!("PAIR{:04}", i), "USDT");
            let symbol = format!("PAIR{:04}USDT", i);

            let summary1 = TestDataFactory::create_summary(
                &symbol,
                Exchange::Binance,
                100.0 + (i as f64),
                101.0 + (i as f64),
                1.0,
                1.0,
            );

            let summary2 = TestDataFactory::create_summary(
                &symbol,
                Exchange::Bybit,
                100.5 + (i as f64),
                101.5 + (i as f64),
                1.0,
                1.0,
            );

            summaries.insert(pair, vec![summary1, summary2]);
        }

        // Process and measure time (memory usage would require external tools)
        let start_time = Instant::now();
        let opportunities = detector.detect_opportunities(&summaries).await;
        let processing_time = start_time.elapsed();

        // Verify processing time scales reasonably with dataset size
        let time_per_pair = processing_time.as_millis() as f64 / size as f64;
        assert!(
            time_per_pair <= 2.0,
            "Processing time per pair ({}ms) is too high for dataset size {}",
            time_per_pair,
            size
        );

        // Verify results are still valid
        assert_all_opportunities_meet_thresholds(&opportunities, 0.1, 0.01);

        println!(
            "Dataset size {}: processed in {}ms ({:.2}ms per pair), found {} opportunities",
            size,
            processing_time.as_millis(),
            time_per_pair,
            opportunities.len()
        );
    }
}
