use exchange_connectors::{Binance, Bybit, Kraken, OrderBookService};
use aggregator_core::PriceLevelUpdate;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration, Instant};
use std::sync::Arc;

mod common;

#[tokio::test]
async fn test_channel_throughput() {
    let (tx, mut rx) = mpsc::channel::<PriceLevelUpdate>(10000);
    
    let start = Instant::now();
    let num_messages = 1000;
    
    // Spawn a task to send messages
    let tx_clone = tx.clone();
    let send_task = tokio::spawn(async move {
        for i in 0..num_messages {
            let update = common::create_test_price_level_update(
                aggregator_core::Exchange::Binance,
                &format!("TEST{}", i % 10)
            );
            if tx_clone.send(update).await.is_err() {
                break;
            }
        }
    });
    
    // Receive messages
    let mut received_count = 0;
    while received_count < num_messages {
        match timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Some(_)) => received_count += 1,
            Ok(None) => break,
            Err(_) => break,
        }
    }
    
    let duration = start.elapsed();
    send_task.abort();
    
    println!("Processed {} messages in {:?}", received_count, duration);
    assert!(received_count > 0, "Should have received at least some messages");
}

#[tokio::test]
async fn test_multiple_exchange_concurrent_creation() {
    let start = Instant::now();
    
    let handles = vec![
        tokio::spawn(async { Binance::new() }),
        tokio::spawn(async { Bybit::new() }),
        tokio::spawn(async { Kraken::new() }),
        tokio::spawn(async { Binance::default() }),
        tokio::spawn(async { Bybit::default() }),
        tokio::spawn(async { Kraken::default() }),
    ];
    
    for handle in handles {
        handle.await.expect("Task should complete successfully");
    }
    
    let duration = start.elapsed();
    println!("Created 6 exchange instances in {:?}", duration);
    assert!(duration < Duration::from_millis(100), "Creation should be fast");
}

#[tokio::test]
async fn test_config_creation_performance() {
    let start = Instant::now();
    let iterations = 10000;
    
    for _ in 0..iterations {
        let _bybit_config = exchange_connectors::bybit::BybitConfig::default();
        let _kraken_config = exchange_connectors::kraken::KrakenConfig::default();
    }
    
    let duration = start.elapsed();
    println!("Created {} config instances in {:?}", iterations * 2, duration);
    assert!(duration < Duration::from_millis(100), "Config creation should be very fast");
}

#[test]
fn test_memory_usage() {
    let binance = Binance::new();
    let bybit = Bybit::new();
    let kraken = Kraken::new();
    
    println!("Binance size: {} bytes", std::mem::size_of_val(&binance));
    println!("Bybit size: {} bytes", std::mem::size_of_val(&bybit));
    println!("Kraken size: {} bytes", std::mem::size_of_val(&kraken));
    
    // These should be relatively small structs
    assert!(std::mem::size_of_val(&binance) < 1024, "Binance should be small");
    assert!(std::mem::size_of_val(&bybit) < 1024, "Bybit should be small");
    assert!(std::mem::size_of_val(&kraken) < 1024, "Kraken should be small");
}