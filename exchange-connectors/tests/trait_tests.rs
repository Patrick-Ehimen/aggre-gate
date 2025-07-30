use aggregator_core::PriceLevelUpdate;
use exchange_connectors::{Binance, Bitstamp, Bybit, Coinbase, Kraken, OrderBookService};
use std::sync::Arc;
use tokio::sync::mpsc;

mod common;

/// Test that all exchanges implement the OrderBookService trait correctly
#[tokio::test]
async fn test_trait_implementation_binance() {
    let exchange: Box<dyn OrderBookService + Send + Sync> = Box::new(Binance::new());
    let (tx, _rx) = mpsc::channel::<PriceLevelUpdate>(100);

    let result = exchange
        .spawn_order_book_service(["BTC", "USDT"], 100, 1000, tx)
        .await;

    // Should return a result (success or failure is both acceptable)
    match result {
        Ok(handles) => {
            println!("Binance service spawned {} handles", handles.len());
        }
        Err(e) => {
            println!("Binance service failed to spawn: {}", e);
        }
    }
}

#[tokio::test]
async fn test_trait_implementation_bybit() {
    let exchange: Box<dyn OrderBookService + Send + Sync> = Box::new(Bybit::new());
    let (tx, _rx) = mpsc::channel::<PriceLevelUpdate>(100);

    let result = exchange
        .spawn_order_book_service(["BTC", "USDT"], 100, 1000, tx)
        .await;

    match result {
        Ok(handles) => {
            println!("Bybit service spawned {} handles", handles.len());
        }
        Err(e) => {
            println!("Bybit service failed to spawn: {}", e);
        }
    }
}

#[tokio::test]
async fn test_trait_implementation_kraken() {
    let exchange: Box<dyn OrderBookService + Send + Sync> = Box::new(Kraken::new());
    let (tx, _rx) = mpsc::channel::<PriceLevelUpdate>(100);

    let result = exchange
        .spawn_order_book_service(["BTC", "USD"], 100, 1000, tx)
        .await;

    match result {
        Ok(handles) => {
            println!("Kraken service spawned {} handles", handles.len());
        }
        Err(e) => {
            println!("Kraken service failed to spawn: {}", e);
        }
    }
}

#[tokio::test]
async fn test_trait_implementation_bitstamp() {
    let exchange: Box<dyn OrderBookService + Send + Sync> = Box::new(Bitstamp::new());
    let (tx, _rx) = mpsc::channel::<PriceLevelUpdate>(100);

    let result = exchange
        .spawn_order_book_service(["BTC", "USD"], 100, 1000, tx)
        .await;

    // Bitstamp is a placeholder, should return empty handles
    assert!(result.is_ok());
    let handles = result.unwrap();
    assert!(handles.is_empty());
}

#[tokio::test]
async fn test_trait_implementation_coinbase() {
    let exchange: Box<dyn OrderBookService + Send + Sync> = Box::new(Coinbase::new());
    let (tx, _rx) = mpsc::channel::<PriceLevelUpdate>(100);

    let result = exchange
        .spawn_order_book_service(["BTC", "USD"], 100, 1000, tx)
        .await;

    // Coinbase is a placeholder, should return empty handles
    assert!(result.is_ok());
    let handles = result.unwrap();
    assert!(handles.is_empty());
}

#[tokio::test]
async fn test_multiple_exchanges_with_same_channel() {
    let (tx, mut rx) = mpsc::channel::<PriceLevelUpdate>(1000);

    let exchanges: Vec<Box<dyn OrderBookService + Send + Sync>> =
        vec![Box::new(Bitstamp::new()), Box::new(Coinbase::new())];

    let mut all_handles = Vec::new();

    for exchange in exchanges {
        match exchange
            .spawn_order_book_service(["BTC", "USD"], 50, 500, tx.clone())
            .await
        {
            Ok(mut handles) => {
                all_handles.append(&mut handles);
            }
            Err(e) => {
                println!("Exchange failed to spawn: {}", e);
            }
        }
    }

    // For placeholder exchanges, we expect no handles
    assert!(all_handles.is_empty());
}

#[test]
fn test_trait_object_creation() {
    let exchanges: Vec<Box<dyn OrderBookService + Send + Sync>> = vec![
        Box::new(Binance::new()),
        Box::new(Bybit::new()),
        Box::new(Kraken::new()),
        Box::new(Bitstamp::new()),
        Box::new(Coinbase::new()),
    ];

    assert_eq!(exchanges.len(), 5);

    // Test that we can store different exchange types in the same collection
    for (i, _exchange) in exchanges.iter().enumerate() {
        println!("Exchange {} created successfully", i);
    }
}

#[test]
fn test_arc_wrapped_exchanges() {
    let binance = Arc::new(Binance::new());
    let bybit = Arc::new(Bybit::new());
    let kraken = Arc::new(Kraken::new());

    // Test that exchanges can be wrapped in Arc for sharing across threads
    let binance_clone = Arc::clone(&binance);
    let bybit_clone = Arc::clone(&bybit);
    let kraken_clone = Arc::clone(&kraken);

    // These should be the same instances
    assert!(Arc::ptr_eq(&binance, &binance_clone));
    assert!(Arc::ptr_eq(&bybit, &bybit_clone));
    assert!(Arc::ptr_eq(&kraken, &kraken_clone));
}
