//! Edge case and error handling tests
//!
//! These tests verify that order book implementations handle unusual
//! inputs and edge cases gracefully.

use aggregator_core::{Ask, Bid, Exchange};
use chrono::Utc;
use orderbook_implementations::{BTreeOrderBook, HashMapOrderBook, OrderBook};

/// Helper function to create a test bid
fn create_bid(price: f64, quantity: f64, exchange: Exchange) -> Bid {
    Bid {
        price,
        quantity,
        exchange,
        timestamp: Utc::now(),
    }
}

/// Helper function to create a test ask
fn create_ask(price: f64, quantity: f64, exchange: Exchange) -> Ask {
    Ask {
        price,
        quantity,
        exchange,
        timestamp: Utc::now(),
    }
}

/// Test handling of extreme price values
#[tokio::test]
async fn test_extreme_prices_btree() {
    test_extreme_prices(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_extreme_prices_hashmap() {
    test_extreme_prices(HashMapOrderBook::new()).await;
}

async fn test_extreme_prices<T: OrderBook>(mut orderbook: T) {
    // Test very large prices
    let large_price_bid = create_bid(f64::MAX / 2.0, 10.0, Exchange::Binance);
    orderbook
        .update_bids(vec![large_price_bid.clone()], 10)
        .await;

    let best_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(best_bid.price, f64::MAX / 2.0);

    // Test very small prices (but positive)
    let small_price_bid = create_bid(f64::MIN_POSITIVE, 10.0, Exchange::Binance);
    orderbook
        .update_bids(vec![small_price_bid.clone()], 10)
        .await;

    // The large price should still be the best
    let best_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(best_bid.price, f64::MAX / 2.0);

    // Test with asks
    let large_price_ask = create_ask(f64::MAX / 2.0, 10.0, Exchange::Binance);
    let small_price_ask = create_ask(f64::MIN_POSITIVE, 10.0, Exchange::Binance);

    orderbook
        .update_asks(vec![large_price_ask, small_price_ask], 10)
        .await;

    // The small price should be the best ask
    let best_ask = orderbook.get_best_ask().await.unwrap();
    assert_eq!(best_ask.price, f64::MIN_POSITIVE);
}

/// Test handling of extreme quantity values
#[tokio::test]
async fn test_extreme_quantities_btree() {
    test_extreme_quantities(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_extreme_quantities_hashmap() {
    test_extreme_quantities(HashMapOrderBook::new()).await;
}

async fn test_extreme_quantities<T: OrderBook>(mut orderbook: T) {
    // Test very large quantity
    let large_qty_bid = create_bid(100.0, f64::MAX / 2.0, Exchange::Binance);
    orderbook.update_bids(vec![large_qty_bid.clone()], 10).await;

    let best_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(best_bid.quantity, f64::MAX / 2.0);

    // Test very small quantity (but positive)
    let small_qty_bid = create_bid(99.0, f64::MIN_POSITIVE, Exchange::Binance);
    orderbook.update_bids(vec![small_qty_bid.clone()], 10).await;

    assert_eq!(orderbook.bid_depth().await, 2);

    // Test updating with zero quantity (should remove)
    let zero_qty_bid = Bid {
        price: 100.0,
        quantity: 0.0,
        exchange: Exchange::Binance,
        timestamp: Utc::now(),
    };
    orderbook.update_bids(vec![zero_qty_bid], 10).await;

    assert_eq!(orderbook.bid_depth().await, 1);
    let remaining_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(remaining_bid.price, 99.0);
}

/// Test handling of special float values (NaN, infinity)
#[tokio::test]
async fn test_special_float_values_btree() {
    test_special_float_values(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_special_float_values_hashmap() {
    test_special_float_values(HashMapOrderBook::new()).await;
}

async fn test_special_float_values<T: OrderBook>(mut orderbook: T) {
    // Note: In a real implementation, you might want to validate inputs
    // and reject NaN/infinity values. This test documents current behavior.

    // Test with normal values first
    let normal_bid = create_bid(100.0, 10.0, Exchange::Binance);
    orderbook.update_bids(vec![normal_bid], 10).await;

    assert_eq!(orderbook.bid_depth().await, 1);

    // Test infinity price - behavior may vary by implementation
    let inf_price_bid = Bid {
        price: f64::INFINITY,
        quantity: 10.0,
        exchange: Exchange::Coinbase,
        timestamp: Utc::now(),
    };

    // This might panic or handle gracefully depending on implementation
    // In production, you'd want to validate inputs before this point
    orderbook.update_bids(vec![inf_price_bid], 10).await;

    // If we get here, the implementation handled infinity
    // The exact behavior depends on how BTreeSet/HashMap handle infinity
    assert!(orderbook.bid_depth().await >= 1);
}

/// Test with maximum depth of 0
#[tokio::test]
async fn test_zero_max_depth_btree() {
    test_zero_max_depth(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_zero_max_depth_hashmap() {
    test_zero_max_depth(HashMapOrderBook::new()).await;
}

async fn test_zero_max_depth<T: OrderBook>(mut orderbook: T) {
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(99.0, 5.0, Exchange::Binance),
    ];

    // Update with max_depth = 0 should keep no orders
    orderbook.update_bids(bids, 0).await;
    assert_eq!(orderbook.bid_depth().await, 0);

    // Same for asks
    let asks = vec![
        create_ask(101.0, 10.0, Exchange::Binance),
        create_ask(102.0, 5.0, Exchange::Binance),
    ];

    orderbook.update_asks(asks, 0).await;
    assert_eq!(orderbook.ask_depth().await, 0);
}

/// Test with maximum depth of 1
#[tokio::test]
async fn test_max_depth_one_btree() {
    test_max_depth_one(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_max_depth_one_hashmap() {
    test_max_depth_one(HashMapOrderBook::new()).await;
}

async fn test_max_depth_one<T: OrderBook>(mut orderbook: T) {
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(99.0, 5.0, Exchange::Binance),
        create_bid(101.0, 8.0, Exchange::Binance),
    ];

    // Should keep only the best bid
    orderbook.update_bids(bids, 1).await;
    assert_eq!(orderbook.bid_depth().await, 1);

    let best_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(best_bid.price, 101.0);

    // Same for asks
    let asks = vec![
        create_ask(102.0, 10.0, Exchange::Binance),
        create_ask(101.0, 5.0, Exchange::Binance),
        create_ask(103.0, 8.0, Exchange::Binance),
    ];

    orderbook.update_asks(asks, 1).await;
    assert_eq!(orderbook.ask_depth().await, 1);

    let best_ask = orderbook.get_best_ask().await.unwrap();
    assert_eq!(best_ask.price, 101.0);
}

/// Test rapid updates to the same price level
#[tokio::test]
async fn test_rapid_updates_btree() {
    test_rapid_updates(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_rapid_updates_hashmap() {
    test_rapid_updates(HashMapOrderBook::new()).await;
}

async fn test_rapid_updates<T: OrderBook>(mut orderbook: T) {
    let price = 100.0;

    // Rapidly update the same price level with different quantities
    for i in 1..=100 {
        let bid = create_bid(price, i as f64, Exchange::Binance);
        orderbook.update_bids(vec![bid], 10).await;

        // Should always have exactly one bid at this price
        assert_eq!(orderbook.bid_depth().await, 1);
        let best_bid = orderbook.get_best_bid().await.unwrap();
        assert_eq!(best_bid.price, price);
        assert_eq!(best_bid.quantity, i as f64);
    }
}

/// Test alternating add/remove operations
#[tokio::test]
async fn test_alternating_add_remove_btree() {
    test_alternating_add_remove(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_alternating_add_remove_hashmap() {
    test_alternating_add_remove(HashMapOrderBook::new()).await;
}

async fn test_alternating_add_remove<T: OrderBook>(mut orderbook: T) {
    let price = 100.0;

    for i in 0..50 {
        if i % 2 == 0 {
            // Add order
            let bid = create_bid(price, 10.0, Exchange::Binance);
            orderbook.update_bids(vec![bid], 10).await;
            assert_eq!(orderbook.bid_depth().await, 1);
        } else {
            // Remove order
            let remove_bid = Bid {
                price,
                quantity: 0.0,
                exchange: Exchange::Binance,
                timestamp: Utc::now(),
            };
            orderbook.update_bids(vec![remove_bid], 10).await;
            assert_eq!(orderbook.bid_depth().await, 0);
        }
    }
}

/// Test with identical timestamps
#[tokio::test]
async fn test_identical_timestamps_btree() {
    test_identical_timestamps(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_identical_timestamps_hashmap() {
    test_identical_timestamps(HashMapOrderBook::new()).await;
}

async fn test_identical_timestamps<T: OrderBook>(mut orderbook: T) {
    let timestamp = Utc::now();

    let bids = vec![
        Bid {
            price: 100.0,
            quantity: 10.0,
            exchange: Exchange::Binance,
            timestamp,
        },
        Bid {
            price: 99.0,
            quantity: 5.0,
            exchange: Exchange::Binance,
            timestamp,
        },
        Bid {
            price: 101.0,
            quantity: 8.0,
            exchange: Exchange::Binance,
            timestamp,
        },
    ];

    orderbook.update_bids(bids, 10).await;
    assert_eq!(orderbook.bid_depth().await, 3);

    // Should still be sorted by price, not timestamp
    let all_bids = orderbook.get_best_n_bids(10).await;
    assert_eq!(all_bids[0].price, 101.0);
    assert_eq!(all_bids[1].price, 100.0);
    assert_eq!(all_bids[2].price, 99.0);
}

/// Test with very large batch updates
#[tokio::test]
async fn test_large_batch_updates_btree() {
    test_large_batch_updates(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_large_batch_updates_hashmap() {
    test_large_batch_updates(HashMapOrderBook::new()).await;
}

async fn test_large_batch_updates<T: OrderBook>(mut orderbook: T) {
    // Create a large batch of orders
    let mut bids = Vec::new();
    for i in 0..10000 {
        bids.push(create_bid(
            100.0 - i as f64 * 0.001,
            10.0,
            Exchange::Binance,
        ));
    }

    // Update with limited depth
    orderbook.update_bids(bids, 100).await;

    // Should be limited to max depth
    assert_eq!(orderbook.bid_depth().await, 100);

    // Best prices should be the highest ones
    let best_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(best_bid.price, 100.0);

    let top_10 = orderbook.get_best_n_bids(10).await;
    assert_eq!(top_10.len(), 10);

    // Verify they're in descending order
    for i in 1..top_10.len() {
        assert!(top_10[i - 1].price >= top_10[i].price);
    }
}

/// Test cross-exchange price conflicts
#[tokio::test]
async fn test_cross_exchange_conflicts_btree() {
    test_cross_exchange_conflicts(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_cross_exchange_conflicts_hashmap() {
    test_cross_exchange_conflicts(HashMapOrderBook::new()).await;
}

async fn test_cross_exchange_conflicts<T: OrderBook>(mut orderbook: T) {
    // Add bids from different exchanges at same price
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(100.0, 15.0, Exchange::Coinbase),
        create_bid(100.0, 5.0, Exchange::Kraken),
    ];

    orderbook.update_bids(bids, 10).await;
    assert_eq!(orderbook.bid_depth().await, 3);

    // Update one exchange
    let update_bid = create_bid(100.0, 20.0, Exchange::Binance);
    orderbook.update_bids(vec![update_bid], 10).await;

    // Should still have 3 orders, with Binance updated
    assert_eq!(orderbook.bid_depth().await, 3);

    let all_bids = orderbook.get_best_n_bids(10).await;
    let binance_bid = all_bids
        .iter()
        .find(|b| b.exchange == Exchange::Binance)
        .unwrap();
    let coinbase_bid = all_bids
        .iter()
        .find(|b| b.exchange == Exchange::Coinbase)
        .unwrap();
    let kraken_bid = all_bids
        .iter()
        .find(|b| b.exchange == Exchange::Kraken)
        .unwrap();

    assert_eq!(binance_bid.quantity, 20.0);
    assert_eq!(coinbase_bid.quantity, 15.0);
    assert_eq!(kraken_bid.quantity, 5.0);

    // Remove one exchange
    let remove_bid = Bid {
        price: 100.0,
        quantity: 0.0,
        exchange: Exchange::Coinbase,
        timestamp: Utc::now(),
    };
    orderbook.update_bids(vec![remove_bid], 10).await;

    // Should have 2 orders left
    assert_eq!(orderbook.bid_depth().await, 2);
    let remaining_bids = orderbook.get_best_n_bids(10).await;
    assert!(!remaining_bids
        .iter()
        .any(|b| b.exchange == Exchange::Coinbase));
}

/// Test spread calculation edge cases
#[tokio::test]
async fn test_spread_edge_cases_btree() {
    test_spread_edge_cases(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_spread_edge_cases_hashmap() {
    test_spread_edge_cases(HashMapOrderBook::new()).await;
}

async fn test_spread_edge_cases<T: OrderBook>(mut orderbook: T) {
    // No spread when empty
    assert!(orderbook.get_spread().await.is_none());

    // No spread with only bids
    let bid = create_bid(100.0, 10.0, Exchange::Binance);
    orderbook.update_bids(vec![bid], 10).await;
    assert!(orderbook.get_spread().await.is_none());

    // No spread with only asks
    orderbook.clear().await;
    let ask = create_ask(101.0, 10.0, Exchange::Binance);
    orderbook.update_asks(vec![ask], 10).await;
    assert!(orderbook.get_spread().await.is_none());

    // Negative spread (crossed market)
    let bid = create_bid(102.0, 10.0, Exchange::Binance);
    orderbook.update_bids(vec![bid], 10).await;

    let spread = orderbook.get_spread().await.unwrap();
    assert_eq!(spread, -1.0); // 101.0 - 102.0

    // Zero spread (same price)
    let same_price_ask = create_ask(102.0, 10.0, Exchange::Binance);
    orderbook.update_asks(vec![same_price_ask], 10).await;

    let spread = orderbook.get_spread().await.unwrap();
    assert_eq!(spread, 0.0); // 102.0 - 102.0

    // Very small spread
    let close_ask = create_ask(102.001, 10.0, Exchange::Binance);
    orderbook.update_asks(vec![close_ask], 10).await;

    let spread = orderbook.get_spread().await.unwrap();
    assert!((spread - 0.001).abs() < f64::EPSILON);
}
