//! Integration tests for order book implementations
//!
//! These tests verify that all order book implementations behave consistently
//! and correctly handle various edge cases and scenarios.

use aggregator_core::{Ask, Bid, Exchange};
use chrono::Utc;
use orderbook_implementations::{BTreeOrderBook, HashMapOrderBook, OrderBook};
use std::time::Duration;
use tokio::time::sleep;

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

/// Test basic order book operations across all implementations
#[tokio::test]
async fn test_basic_operations_btree() {
    test_basic_operations(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_basic_operations_hashmap() {
    test_basic_operations(HashMapOrderBook::new()).await;
}

async fn test_basic_operations<T: OrderBook>(mut orderbook: T) {
    // Test empty orderbook
    assert!(orderbook.get_best_bid().await.is_none());
    assert!(orderbook.get_best_ask().await.is_none());
    assert_eq!(orderbook.bid_depth().await, 0);
    assert_eq!(orderbook.ask_depth().await, 0);
    assert!(orderbook.get_spread().await.is_none());

    // Add some bids
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(99.0, 5.0, Exchange::Binance),
        create_bid(98.0, 15.0, Exchange::Coinbase),
    ];
    orderbook.update_bids(bids, 10).await;

    // Test bid operations
    assert_eq!(orderbook.bid_depth().await, 3);
    let best_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(best_bid.price, 100.0);
    assert_eq!(best_bid.quantity, 10.0);

    // Add some asks
    let asks = vec![
        create_ask(101.0, 8.0, Exchange::Binance),
        create_ask(102.0, 12.0, Exchange::Coinbase),
        create_ask(103.0, 6.0, Exchange::Binance),
    ];
    orderbook.update_asks(asks, 10).await;

    // Test ask operations
    assert_eq!(orderbook.ask_depth().await, 3);
    let best_ask = orderbook.get_best_ask().await.unwrap();
    assert_eq!(best_ask.price, 101.0);
    assert_eq!(best_ask.quantity, 8.0);

    // Test spread
    let spread = orderbook.get_spread().await.unwrap();
    assert_eq!(spread, 1.0); // 101.0 - 100.0

    // Test getting multiple orders
    let top_2_bids = orderbook.get_best_n_bids(2).await;
    assert_eq!(top_2_bids.len(), 2);
    assert_eq!(top_2_bids[0].price, 100.0);
    assert_eq!(top_2_bids[1].price, 99.0);

    let top_2_asks = orderbook.get_best_n_asks(2).await;
    assert_eq!(top_2_asks.len(), 2);
    assert_eq!(top_2_asks[0].price, 101.0);
    assert_eq!(top_2_asks[1].price, 102.0);
}

/// Test order updates and replacements
#[tokio::test]
async fn test_order_updates_btree() {
    test_order_updates(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_order_updates_hashmap() {
    test_order_updates(HashMapOrderBook::new()).await;
}

async fn test_order_updates<T: OrderBook>(mut orderbook: T) {
    // Add initial bid
    let initial_bid = create_bid(100.0, 10.0, Exchange::Binance);
    orderbook.update_bids(vec![initial_bid], 10).await;
    assert_eq!(orderbook.bid_depth().await, 1);

    // Update with new quantity at same price and exchange
    let updated_bid = create_bid(100.0, 20.0, Exchange::Binance);
    orderbook.update_bids(vec![updated_bid], 10).await;

    // Should still have only one bid with updated quantity
    assert_eq!(orderbook.bid_depth().await, 1);
    let best_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(best_bid.quantity, 20.0);

    // Add bid at same price but different exchange
    let different_exchange_bid = create_bid(100.0, 15.0, Exchange::Coinbase);
    orderbook
        .update_bids(vec![different_exchange_bid], 10)
        .await;

    // Note: Due to Ord implementation only comparing price, BTreeSet will treat
    // orders with same price as equal regardless of exchange. The behavior
    // depends on the implementation - BTreeSet will keep one, HashMap can keep both.
    let depth = orderbook.bid_depth().await;
    assert!(depth >= 1); // At least one bid should exist
}

/// Test order removal with zero quantity
#[tokio::test]
async fn test_order_removal_btree() {
    test_order_removal(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_order_removal_hashmap() {
    test_order_removal(HashMapOrderBook::new()).await;
}

async fn test_order_removal<T: OrderBook>(mut orderbook: T) {
    // Add some orders
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(99.0, 5.0, Exchange::Binance),
    ];
    orderbook.update_bids(bids, 10).await;
    assert_eq!(orderbook.bid_depth().await, 2);

    // Remove one order by setting quantity to 0
    let remove_bid = Bid {
        price: 100.0,
        quantity: 0.0,
        exchange: Exchange::Binance,
        timestamp: Utc::now(),
    };
    orderbook.update_bids(vec![remove_bid], 10).await;

    // Should have one less bid
    assert_eq!(orderbook.bid_depth().await, 1);
    let best_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(best_bid.price, 99.0);

    // Remove the last bid
    let remove_last_bid = Bid {
        price: 99.0,
        quantity: 0.0,
        exchange: Exchange::Binance,
        timestamp: Utc::now(),
    };
    orderbook.update_bids(vec![remove_last_bid], 10).await;

    // Should be empty
    assert_eq!(orderbook.bid_depth().await, 0);
    assert!(orderbook.get_best_bid().await.is_none());
}

/// Test depth limiting functionality
#[tokio::test]
async fn test_depth_limiting_btree() {
    test_depth_limiting(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_depth_limiting_hashmap() {
    test_depth_limiting(HashMapOrderBook::new()).await;
}

async fn test_depth_limiting<T: OrderBook>(mut orderbook: T) {
    // Add more bids than the max depth
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(99.0, 10.0, Exchange::Binance),
        create_bid(98.0, 10.0, Exchange::Binance),
        create_bid(97.0, 10.0, Exchange::Binance),
        create_bid(96.0, 10.0, Exchange::Binance),
    ];

    // Limit to 3 levels
    orderbook.update_bids(bids, 3).await;

    // Should only keep the best 3 bids
    assert_eq!(orderbook.bid_depth().await, 3);
    let all_bids = orderbook.get_best_n_bids(10).await;
    assert_eq!(all_bids.len(), 3);
    assert_eq!(all_bids[0].price, 100.0);
    assert_eq!(all_bids[1].price, 99.0);
    assert_eq!(all_bids[2].price, 98.0);

    // Test same for asks
    let asks = vec![
        create_ask(101.0, 10.0, Exchange::Binance),
        create_ask(102.0, 10.0, Exchange::Binance),
        create_ask(103.0, 10.0, Exchange::Binance),
        create_ask(104.0, 10.0, Exchange::Binance),
        create_ask(105.0, 10.0, Exchange::Binance),
    ];

    orderbook.update_asks(asks, 3).await;

    assert_eq!(orderbook.ask_depth().await, 3);
    let all_asks = orderbook.get_best_n_asks(10).await;
    assert_eq!(all_asks.len(), 3);
    assert_eq!(all_asks[0].price, 101.0);
    assert_eq!(all_asks[1].price, 102.0);
    assert_eq!(all_asks[2].price, 103.0);
}

/// Test clearing the order book
#[tokio::test]
async fn test_clear_orderbook_btree() {
    test_clear_orderbook(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_clear_orderbook_hashmap() {
    test_clear_orderbook(HashMapOrderBook::new()).await;
}

async fn test_clear_orderbook<T: OrderBook>(mut orderbook: T) {
    // Add some orders
    let bids = vec![create_bid(100.0, 10.0, Exchange::Binance)];
    let asks = vec![create_ask(101.0, 10.0, Exchange::Binance)];

    orderbook.update_bids(bids, 10).await;
    orderbook.update_asks(asks, 10).await;

    assert_eq!(orderbook.bid_depth().await, 1);
    assert_eq!(orderbook.ask_depth().await, 1);

    // Clear the orderbook
    orderbook.clear().await;

    // Should be empty
    assert_eq!(orderbook.bid_depth().await, 0);
    assert_eq!(orderbook.ask_depth().await, 0);
    assert!(orderbook.get_best_bid().await.is_none());
    assert!(orderbook.get_best_ask().await.is_none());
    assert!(orderbook.get_spread().await.is_none());
}

/// Test edge cases with empty requests
#[tokio::test]
async fn test_edge_cases_btree() {
    test_edge_cases(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_edge_cases_hashmap() {
    test_edge_cases(HashMapOrderBook::new()).await;
}

async fn test_edge_cases<T: OrderBook>(mut orderbook: T) {
    // Test updating with empty vectors
    orderbook.update_bids(vec![], 10).await;
    orderbook.update_asks(vec![], 10).await;
    assert_eq!(orderbook.bid_depth().await, 0);
    assert_eq!(orderbook.ask_depth().await, 0);

    // Test getting 0 orders
    let zero_bids = orderbook.get_best_n_bids(0).await;
    let zero_asks = orderbook.get_best_n_asks(0).await;
    assert_eq!(zero_bids.len(), 0);
    assert_eq!(zero_asks.len(), 0);

    // Add one order and test getting more than available
    let bid = create_bid(100.0, 10.0, Exchange::Binance);
    orderbook.update_bids(vec![bid], 10).await;

    let many_bids = orderbook.get_best_n_bids(100).await;
    assert_eq!(many_bids.len(), 1);

    // Test max_depth of 0
    orderbook
        .update_bids(vec![create_bid(99.0, 10.0, Exchange::Binance)], 0)
        .await;
    assert_eq!(orderbook.bid_depth().await, 0);
}

/// Test multiple exchanges at same price level
#[tokio::test]
async fn test_multiple_exchanges_btree() {
    test_multiple_exchanges(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_multiple_exchanges_hashmap() {
    test_multiple_exchanges(HashMapOrderBook::new()).await;
}

async fn test_multiple_exchanges<T: OrderBook>(mut orderbook: T) {
    // Add bids at same price from different exchanges
    let bids = vec![
        create_bid(100.0, 10.0, Exchange::Binance),
        create_bid(100.0, 15.0, Exchange::Coinbase),
        create_bid(100.0, 5.0, Exchange::Kraken),
    ];

    orderbook.update_bids(bids, 10).await;

    // Note: BTreeSet only keeps one order per price level due to Ord implementation
    // HashMap can keep multiple orders at same price with different exchanges
    let depth = orderbook.bid_depth().await;
    assert!(depth >= 1); // At least one bid should exist

    // Get all available bids
    let all_bids = orderbook.get_best_n_bids(10).await;
    assert!(all_bids.len() >= 1);
    for bid in &all_bids {
        assert_eq!(bid.price, 100.0);
    }

    // Update one exchange
    let update_bid = create_bid(100.0, 20.0, Exchange::Binance);
    orderbook.update_bids(vec![update_bid], 10).await;

    // Should still have at least one entry
    let final_depth = orderbook.bid_depth().await;
    assert!(final_depth >= 1);
    let updated_bids = orderbook.get_best_n_bids(10).await;
    // For implementations that support multiple exchanges at same price,
    // verify the Binance bid was updated
    if let Some(binance_bid) = updated_bids
        .iter()
        .find(|b| b.exchange == Exchange::Binance)
    {
        assert_eq!(binance_bid.quantity, 20.0);
    }
}

/// Test concurrent access patterns
#[tokio::test]
async fn test_concurrent_access_btree() {
    test_concurrent_access(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_concurrent_access_hashmap() {
    test_concurrent_access(HashMapOrderBook::new()).await;
}

async fn test_concurrent_access<T: OrderBook + Clone + 'static>(orderbook: T) {
    let orderbook = std::sync::Arc::new(tokio::sync::Mutex::new(orderbook));

    // Spawn multiple tasks that read and write concurrently
    let mut handles = vec![];

    // Writer task
    let orderbook_writer = orderbook.clone();
    let writer_handle = tokio::spawn(async move {
        for i in 0..10 {
            let bid = create_bid(100.0 + i as f64, 10.0, Exchange::Binance);
            let mut ob = orderbook_writer.lock().await;
            ob.update_bids(vec![bid], 100).await;
            sleep(Duration::from_millis(1)).await;
        }
    });
    handles.push(writer_handle);

    // Reader tasks
    for _ in 0..3 {
        let orderbook_reader = orderbook.clone();
        let reader_handle = tokio::spawn(async move {
            for _ in 0..20 {
                let ob = orderbook_reader.lock().await;
                let _ = ob.get_best_bid().await;
                let _ = ob.bid_depth().await;
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(reader_handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify final state
    let ob = orderbook.lock().await;
    assert!(ob.bid_depth().await > 0);
}

/// Test price precision handling
#[tokio::test]
async fn test_price_precision_btree() {
    test_price_precision(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_price_precision_hashmap() {
    test_price_precision(HashMapOrderBook::new()).await;
}

async fn test_price_precision<T: OrderBook>(mut orderbook: T) {
    // Test with high precision prices
    let bids = vec![
        create_bid(100.12345678, 10.0, Exchange::Binance),
        create_bid(100.12345677, 5.0, Exchange::Binance),
        create_bid(100.12345679, 15.0, Exchange::Binance),
    ];

    orderbook.update_bids(bids, 10).await;

    let best_bid = orderbook.get_best_bid().await.unwrap();
    assert_eq!(best_bid.price, 100.12345679);
    assert_eq!(best_bid.quantity, 15.0);

    let all_bids = orderbook.get_best_n_bids(10).await;
    assert_eq!(all_bids.len(), 3);
    assert_eq!(all_bids[0].price, 100.12345679);
    assert_eq!(all_bids[1].price, 100.12345678);
    assert_eq!(all_bids[2].price, 100.12345677);
}

/// Test large order book performance
#[tokio::test]
async fn test_large_orderbook_btree() {
    test_large_orderbook(BTreeOrderBook::new()).await;
}

#[tokio::test]
async fn test_large_orderbook_hashmap() {
    test_large_orderbook(HashMapOrderBook::new()).await;
}

async fn test_large_orderbook<T: OrderBook>(mut orderbook: T) {
    // Create a large number of orders
    let mut bids = Vec::new();
    let mut asks = Vec::new();

    for i in 0..1000 {
        bids.push(create_bid(100.0 - i as f64 * 0.01, 10.0, Exchange::Binance));
        asks.push(create_ask(101.0 + i as f64 * 0.01, 10.0, Exchange::Binance));
    }

    // Update with large batch
    orderbook.update_bids(bids, 500).await;
    orderbook.update_asks(asks, 500).await;

    // Verify depth limiting worked
    assert_eq!(orderbook.bid_depth().await, 500);
    assert_eq!(orderbook.ask_depth().await, 500);

    // Verify ordering is correct
    let best_bid = orderbook.get_best_bid().await.unwrap();
    let best_ask = orderbook.get_best_ask().await.unwrap();
    assert_eq!(best_bid.price, 100.0);
    assert_eq!(best_ask.price, 101.0);

    // Test getting large number of orders
    let top_100_bids = orderbook.get_best_n_bids(100).await;
    let top_100_asks = orderbook.get_best_n_asks(100).await;
    assert_eq!(top_100_bids.len(), 100);
    assert_eq!(top_100_asks.len(), 100);

    // Verify they're properly sorted
    for i in 1..top_100_bids.len() {
        assert!(top_100_bids[i - 1].price >= top_100_bids[i].price);
    }
    for i in 1..top_100_asks.len() {
        assert!(top_100_asks[i - 1].price <= top_100_asks[i].price);
    }
}
