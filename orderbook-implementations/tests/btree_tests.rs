//! Unit tests specific to BTreeSet implementation
//!
//! These tests focus on BTreeSet-specific behavior and edge cases

use aggregator_core::{Ask, Bid, Exchange};
use chrono::Utc;
use orderbook_implementations::{
    btree_set::{BTreeAskSide, BTreeBidSide, BTreeOrderBook},
    BuySide, OrderBook, SellSide,
};

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

#[tokio::test]
async fn test_btree_orderbook_creation() {
    let orderbook = BTreeOrderBook::new();
    assert_eq!(orderbook.bid_depth().await, 0);
    assert_eq!(orderbook.ask_depth().await, 0);

    // Test default implementation
    let default_orderbook = BTreeOrderBook::default();
    assert_eq!(default_orderbook.bid_depth().await, 0);
    assert_eq!(default_orderbook.ask_depth().await, 0);
}

#[tokio::test]
async fn test_btree_bid_side_operations() {
    let mut bid_side = BTreeBidSide::new();

    // Test empty state
    assert_eq!(bid_side.bid_depth().await, 0);
    assert!(bid_side.get_best_bid().await.is_none());

    // Add some bids
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(99.0, 5.0, Exchange::Coinbase),
        create_bid(101.0, 8.0, Exchange::Kraken),
    ];

    bid_side.update_bids(bids, 10).await;

    // Test operations
    assert_eq!(bid_side.bid_depth().await, 3);

    let best_bid = bid_side.get_best_bid().await.unwrap();
    assert_eq!(best_bid.price, 101.0);
    assert_eq!(best_bid.exchange, Exchange::Kraken);

    let top_2_bids = bid_side.get_best_n_bids(2).await;
    assert_eq!(top_2_bids.len(), 2);
    assert_eq!(top_2_bids[0].price, 101.0);
    assert_eq!(top_2_bids[1].price, 100.0);

    // Test clear
    bid_side.clear_bids().await;
    assert_eq!(bid_side.bid_depth().await, 0);
    assert!(bid_side.get_best_bid().await.is_none());
}

#[tokio::test]
async fn test_btree_ask_side_operations() {
    let mut ask_side = BTreeAskSide::new();

    // Test empty state
    assert_eq!(ask_side.ask_depth().await, 0);
    assert!(ask_side.get_best_ask().await.is_none());

    // Add some asks
    let asks = vec![
        create_ask(102.0, 12.0, Exchange::Binance),
        create_ask(101.0, 8.0, Exchange::Coinbase),
        create_ask(103.0, 6.0, Exchange::Kraken),
    ];

    ask_side.update_asks(asks, 10).await;

    // Test operations
    assert_eq!(ask_side.ask_depth().await, 3);

    let best_ask = ask_side.get_best_ask().await.unwrap();
    assert_eq!(best_ask.price, 101.0);
    assert_eq!(best_ask.exchange, Exchange::Coinbase);

    let top_2_asks = ask_side.get_best_n_asks(2).await;
    assert_eq!(top_2_asks.len(), 2);
    assert_eq!(top_2_asks[0].price, 101.0);
    assert_eq!(top_2_asks[1].price, 102.0);

    // Test clear
    ask_side.clear_asks().await;
    assert_eq!(ask_side.ask_depth().await, 0);
    assert!(ask_side.get_best_ask().await.is_none());
}

#[tokio::test]
async fn test_btree_side_views() {
    let mut orderbook = BTreeOrderBook::new();

    // Add some orders to the main orderbook
    let bids = vec![create_bid(100.0, 10.0, Exchange::Binance)];
    let asks = vec![create_ask(101.0, 8.0, Exchange::Binance)];

    orderbook.update_bids(bids, 10).await;
    orderbook.update_asks(asks, 10).await;

    // Get side views
    let bid_side = orderbook.bid_side();
    let ask_side = orderbook.ask_side();

    // Test that side views share the same data
    assert_eq!(bid_side.bid_depth().await, 1);
    assert_eq!(ask_side.ask_depth().await, 1);

    let best_bid = bid_side.get_best_bid().await.unwrap();
    let best_ask = ask_side.get_best_ask().await.unwrap();

    assert_eq!(best_bid.price, 100.0);
    assert_eq!(best_ask.price, 101.0);
}

#[tokio::test]
async fn test_btree_sorting_behavior() {
    let mut orderbook = BTreeOrderBook::new();

    // Add bids in random order
    let bids = vec![
        create_bid(95.0, 10.0, Exchange::Binance),
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(97.5, 10.0, Exchange::Binance),
        create_bid(102.0, 10.0, Exchange::Binance),
        create_bid(99.0, 10.0, Exchange::Binance),
    ];

    orderbook.update_bids(bids, 10).await;

    // Should be sorted in descending order (highest first)
    let all_bids = orderbook.get_best_n_bids(10).await;
    let expected_prices = vec![102.0, 100.0, 99.0, 97.5, 95.0];

    for (i, expected_price) in expected_prices.iter().enumerate() {
        assert_eq!(all_bids[i].price, *expected_price);
    }

    // Add asks in random order
    let asks = vec![
        create_ask(105.0, 10.0, Exchange::Binance),
        create_ask(101.0, 10.0, Exchange::Binance),
        create_ask(103.5, 10.0, Exchange::Binance),
        create_ask(100.5, 10.0, Exchange::Binance),
        create_ask(102.0, 10.0, Exchange::Binance),
    ];

    orderbook.update_asks(asks, 10).await;

    // Should be sorted in ascending order (lowest first)
    let all_asks = orderbook.get_best_n_asks(10).await;
    let expected_ask_prices = vec![100.5, 101.0, 102.0, 103.5, 105.0];

    for (i, expected_price) in expected_ask_prices.iter().enumerate() {
        assert_eq!(all_asks[i].price, *expected_price);
    }
}

#[tokio::test]
async fn test_btree_depth_trimming() {
    let mut orderbook = BTreeOrderBook::new();

    // Add more bids than max_depth
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(99.0, 10.0, Exchange::Binance),
        create_bid(98.0, 10.0, Exchange::Binance),
        create_bid(97.0, 10.0, Exchange::Binance),
        create_bid(96.0, 10.0, Exchange::Binance),
        create_bid(95.0, 10.0, Exchange::Binance),
    ];

    // Limit to 3 levels
    orderbook.update_bids(bids, 3).await;

    // Should keep only the best 3 (highest prices)
    assert_eq!(orderbook.bid_depth().await, 3);
    let remaining_bids = orderbook.get_best_n_bids(10).await;
    assert_eq!(remaining_bids.len(), 3);
    assert_eq!(remaining_bids[0].price, 100.0);
    assert_eq!(remaining_bids[1].price, 99.0);
    assert_eq!(remaining_bids[2].price, 98.0);

    // Test same for asks
    let asks = vec![
        create_ask(101.0, 10.0, Exchange::Binance),
        create_ask(102.0, 10.0, Exchange::Binance),
        create_ask(103.0, 10.0, Exchange::Binance),
        create_ask(104.0, 10.0, Exchange::Binance),
        create_ask(105.0, 10.0, Exchange::Binance),
        create_ask(106.0, 10.0, Exchange::Binance),
    ];

    orderbook.update_asks(asks, 3).await;

    // Should keep only the best 3 (lowest prices)
    assert_eq!(orderbook.ask_depth().await, 3);
    let remaining_asks = orderbook.get_best_n_asks(10).await;
    assert_eq!(remaining_asks.len(), 3);
    assert_eq!(remaining_asks[0].price, 101.0);
    assert_eq!(remaining_asks[1].price, 102.0);
    assert_eq!(remaining_asks[2].price, 103.0);
}

#[tokio::test]
async fn test_btree_update_same_price_different_exchange() {
    let mut orderbook = BTreeOrderBook::new();

    // Add bids at same price from different exchanges
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(100.0, 15.0, Exchange::Coinbase),
        create_bid(100.0, 5.0, Exchange::Kraken),
    ];

    orderbook.update_bids(bids, 10).await;
    // BTreeSet only keeps one order per price level due to Ord implementation
    assert_eq!(orderbook.bid_depth().await, 1);

    // Update one exchange
    let update_bid = create_bid(100.0, 20.0, Exchange::Binance);
    orderbook.update_bids(vec![update_bid], 10).await;

    // Should still have 1 entry
    assert_eq!(orderbook.bid_depth().await, 1);

    // Verify the remaining bid has the updated quantity
    let all_bids = orderbook.get_best_n_bids(10).await;
    assert_eq!(all_bids.len(), 1);
    assert_eq!(all_bids[0].price, 100.0);
    assert_eq!(all_bids[0].quantity, 20.0);
}

#[tokio::test]
async fn test_btree_zero_quantity_removal() {
    let mut orderbook = BTreeOrderBook::new();

    // Add some bids
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(99.0, 5.0, Exchange::Binance),
        create_bid(98.0, 15.0, Exchange::Binance),
    ];

    orderbook.update_bids(bids, 10).await;
    assert_eq!(orderbook.bid_depth().await, 3);

    // Remove middle bid by setting quantity to 0
    let remove_bid = Bid {
        price: 99.0,
        quantity: 0.0,
        exchange: Exchange::Binance,
        timestamp: Utc::now(),
    };

    orderbook.update_bids(vec![remove_bid], 10).await;
    assert_eq!(orderbook.bid_depth().await, 2);

    // Verify the correct bid was removed
    let remaining_bids = orderbook.get_best_n_bids(10).await;
    let prices: Vec<f64> = remaining_bids.iter().map(|b| b.price).collect();
    assert_eq!(prices, vec![100.0, 98.0]);
}

#[tokio::test]
async fn test_btree_empty_update() {
    let mut orderbook = BTreeOrderBook::new();

    // Add some initial data
    let bids = vec![create_bid(100.0, 10.0, Exchange::Binance)];
    orderbook.update_bids(bids, 10).await;
    assert_eq!(orderbook.bid_depth().await, 1);

    // Update with empty vector
    orderbook.update_bids(vec![], 10).await;

    // Should still have the original bid
    assert_eq!(orderbook.bid_depth().await, 1);

    // Same for asks
    let asks = vec![create_ask(101.0, 10.0, Exchange::Binance)];
    orderbook.update_asks(asks, 10).await;
    assert_eq!(orderbook.ask_depth().await, 1);

    orderbook.update_asks(vec![], 10).await;
    assert_eq!(orderbook.ask_depth().await, 1);
}

#[tokio::test]
async fn test_btree_get_more_than_available() {
    let mut orderbook = BTreeOrderBook::new();

    // Add only 2 bids
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(99.0, 5.0, Exchange::Binance),
    ];

    orderbook.update_bids(bids, 10).await;

    // Request more than available
    let many_bids = orderbook.get_best_n_bids(10).await;
    assert_eq!(many_bids.len(), 2);

    // Request 0
    let zero_bids = orderbook.get_best_n_bids(0).await;
    assert_eq!(zero_bids.len(), 0);
}

#[tokio::test]
async fn test_btree_clone_behavior() {
    let mut orderbook1 = BTreeOrderBook::new();

    // Add some data
    let bids = vec![create_bid(100.0, 10.0, Exchange::Binance)];
    orderbook1.update_bids(bids, 10).await;

    // Clone the orderbook
    let orderbook2 = orderbook1.clone();

    // Both should have the same data initially
    assert_eq!(orderbook1.bid_depth().await, 1);
    assert_eq!(orderbook2.bid_depth().await, 1);

    // Modify one
    let new_bids = vec![create_bid(99.0, 5.0, Exchange::Binance)];
    orderbook1.update_bids(new_bids, 10).await;

    // Both should reflect the change (they share the same Arc)
    assert_eq!(orderbook1.bid_depth().await, 2);
    assert_eq!(orderbook2.bid_depth().await, 2);
}

#[tokio::test]
async fn test_btree_spread_calculation() {
    let mut orderbook = BTreeOrderBook::new();

    // No spread when empty
    assert!(orderbook.get_spread().await.is_none());

    // No spread with only bids
    let bids = vec![create_bid(100.0, 10.0, Exchange::Binance)];
    orderbook.update_bids(bids, 10).await;
    assert!(orderbook.get_spread().await.is_none());

    // No spread with only asks
    orderbook.clear().await;
    let asks = vec![create_ask(101.0, 10.0, Exchange::Binance)];
    orderbook.update_asks(asks, 10).await;
    assert!(orderbook.get_spread().await.is_none());

    // Spread with both sides
    let bids = vec![create_bid(100.0, 10.0, Exchange::Binance)];
    orderbook.update_bids(bids, 10).await;

    let spread = orderbook.get_spread().await.unwrap();
    assert_eq!(spread, 1.0); // 101.0 - 100.0

    // Test with different spread
    let new_asks = vec![create_ask(105.0, 10.0, Exchange::Binance)];
    orderbook.update_asks(new_asks, 10).await;

    let new_spread = orderbook.get_spread().await.unwrap();
    assert_eq!(new_spread, 5.0); // 105.0 - 100.0
}
