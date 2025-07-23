//! Custom assertion helpers for domain-specific validations

use aggregator_core::{ArbitrageOpportunity, Exchange};

/// Assert that an arbitrage opportunity matches expected values
pub fn assert_arbitrage_opportunity(
    opportunity: &ArbitrageOpportunity,
    expected_buy_exchange: Exchange,
    expected_sell_exchange: Exchange,
    expected_symbol: &str,
    min_profit_percentage: f64,
) {
    assert_eq!(
        opportunity.buy_exchange, expected_buy_exchange,
        "Buy exchange mismatch: expected {:?}, got {:?}",
        expected_buy_exchange, opportunity.buy_exchange
    );

    assert_eq!(
        opportunity.sell_exchange, expected_sell_exchange,
        "Sell exchange mismatch: expected {:?}, got {:?}",
        expected_sell_exchange, opportunity.sell_exchange
    );

    assert_eq!(
        opportunity.symbol, expected_symbol,
        "Symbol mismatch: expected {}, got {}",
        expected_symbol, opportunity.symbol
    );

    assert!(
        opportunity.profit_percentage >= min_profit_percentage,
        "Profit percentage too low: expected >= {}, got {}",
        min_profit_percentage,
        opportunity.profit_percentage
    );

    assert!(
        opportunity.buy_price > 0.0,
        "Buy price should be positive, got {}",
        opportunity.buy_price
    );

    assert!(
        opportunity.sell_price > 0.0,
        "Sell price should be positive, got {}",
        opportunity.sell_price
    );

    assert!(
        opportunity.sell_price > opportunity.buy_price,
        "Sell price ({}) should be higher than buy price ({})",
        opportunity.sell_price,
        opportunity.buy_price
    );

    assert!(
        opportunity.volume > 0.0,
        "Volume should be positive, got {}",
        opportunity.volume
    );
}

/// Assert that no arbitrage opportunities exist
pub fn assert_no_arbitrage_opportunities(opportunities: &[ArbitrageOpportunity]) {
    assert!(
        opportunities.is_empty(),
        "Expected no arbitrage opportunities, but found {}: {:#?}",
        opportunities.len(),
        opportunities
    );
}

/// Assert that the number of opportunities matches expected count
pub fn assert_opportunity_count(opportunities: &[ArbitrageOpportunity], expected_count: usize) {
    assert_eq!(
        opportunities.len(),
        expected_count,
        "Expected {} opportunities, but found {}: {:#?}",
        expected_count,
        opportunities.len(),
        opportunities
    );
}

/// Assert spread calculation with tolerance
pub fn assert_spread_calculation(actual: Option<f64>, expected: f64, tolerance: f64) {
    match actual {
        Some(actual_value) => {
            assert!(
                (actual_value - expected).abs() <= tolerance,
                "Spread calculation mismatch: expected {} ± {}, got {}",
                expected,
                tolerance,
                actual_value
            );
        }
        None => {
            panic!("Expected spread value of {}, but got None", expected);
        }
    }
}

/// Assert spread calculation returns None
pub fn assert_spread_none(actual: Option<f64>, reason: &str) {
    assert!(
        actual.is_none(),
        "Expected spread to be None ({}), but got {:?}",
        reason,
        actual
    );
}

/// Assert VWAP calculation with tolerance
pub fn assert_vwap_calculation(actual: Option<f64>, expected: f64, tolerance: f64) {
    match actual {
        Some(actual_value) => {
            assert!(
                (actual_value - expected).abs() <= tolerance,
                "VWAP calculation mismatch: expected {} ± {}, got {}",
                expected,
                tolerance,
                actual_value
            );
        }
        None => {
            panic!("Expected VWAP value of {}, but got None", expected);
        }
    }
}

/// Assert VWAP calculation returns None
pub fn assert_vwap_none(actual: Option<f64>, reason: &str) {
    assert!(
        actual.is_none(),
        "Expected VWAP to be None ({}), but got {:?}",
        reason,
        actual
    );
}

/// Assert that a value is within a specified tolerance of an expected value
pub fn assert_within_tolerance(actual: f64, expected: f64, tolerance: f64, context: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{}: expected {} ± {}, got {}",
        context,
        expected,
        tolerance,
        actual
    );
}

/// Assert that a profit percentage meets minimum threshold
pub fn assert_profit_threshold(profit_percentage: f64, min_threshold: f64) {
    assert!(
        profit_percentage >= min_threshold,
        "Profit percentage {} does not meet minimum threshold {}",
        profit_percentage,
        min_threshold
    );
}

/// Assert that a volume meets minimum threshold
pub fn assert_volume_threshold(volume: f64, min_threshold: f64) {
    assert!(
        volume >= min_threshold,
        "Volume {} does not meet minimum threshold {}",
        volume,
        min_threshold
    );
}

/// Assert that prices are valid (positive and finite)
pub fn assert_valid_price(price: f64, context: &str) {
    assert!(
        price > 0.0 && price.is_finite(),
        "{}: price should be positive and finite, got {}",
        context,
        price
    );
}

/// Assert that quantities are valid (non-negative and finite)
pub fn assert_valid_quantity(quantity: f64, context: &str) {
    assert!(
        quantity >= 0.0 && quantity.is_finite(),
        "{}: quantity should be non-negative and finite, got {}",
        context,
        quantity
    );
}

/// Assert that an opportunity has valid profit calculation
pub fn assert_valid_profit_calculation(opportunity: &ArbitrageOpportunity) {
    let calculated_profit = opportunity.sell_price - opportunity.buy_price;
    let calculated_percentage = (calculated_profit / opportunity.buy_price) * 100.0;

    assert_within_tolerance(
        opportunity.profit_percentage,
        calculated_percentage,
        0.001, // 0.001% tolerance
        "Profit percentage calculation",
    );
}

/// Assert that opportunities are sorted by profit percentage (descending)
pub fn assert_opportunities_sorted_by_profit(opportunities: &[ArbitrageOpportunity]) {
    for i in 1..opportunities.len() {
        assert!(
            opportunities[i-1].profit_percentage >= opportunities[i].profit_percentage,
            "Opportunities not sorted by profit percentage: position {} has {}%, position {} has {}%",
            i-1, opportunities[i-1].profit_percentage,
            i, opportunities[i].profit_percentage
        );
    }
}

/// Assert that all opportunities in a list meet minimum thresholds
pub fn assert_all_opportunities_meet_thresholds(
    opportunities: &[ArbitrageOpportunity],
    min_profit_threshold: f64,
    min_volume_threshold: f64,
) {
    for (i, opportunity) in opportunities.iter().enumerate() {
        assert_profit_threshold(opportunity.profit_percentage, min_profit_threshold);
        assert_volume_threshold(opportunity.volume, min_volume_threshold);
        assert_valid_profit_calculation(opportunity);

        // Additional context for debugging
        if opportunity.profit_percentage < min_profit_threshold {
            panic!(
                "Opportunity {} does not meet profit threshold: {}% < {}%",
                i, opportunity.profit_percentage, min_profit_threshold
            );
        }

        if opportunity.volume < min_volume_threshold {
            panic!(
                "Opportunity {} does not meet volume threshold: {} < {}",
                i, opportunity.volume, min_volume_threshold
            );
        }
    }
}

/// Assert that opportunities contain unique exchange pairs
pub fn assert_unique_exchange_pairs(opportunities: &[ArbitrageOpportunity]) {
    let mut seen_pairs = std::collections::HashSet::new();

    for (i, opportunity) in opportunities.iter().enumerate() {
        let pair = (
            opportunity.buy_exchange.clone(),
            opportunity.sell_exchange.clone(),
            opportunity.symbol.clone(),
        );

        assert!(
            seen_pairs.insert(pair.clone()),
            "Duplicate exchange pair found at position {}: {:?}",
            i,
            pair
        );
    }
}

/// Assert that timestamps are recent (within specified duration)
pub fn assert_recent_timestamp(opportunity: &ArbitrageOpportunity, max_age: std::time::Duration) {
    let now = chrono::Utc::now();
    let age = now.signed_duration_since(opportunity.timestamp);

    assert!(
        age.to_std().unwrap_or(std::time::Duration::MAX) <= max_age,
        "Opportunity timestamp is too old: {} seconds ago (max: {} seconds)",
        age.num_seconds(),
        max_age.as_secs()
    );
}
