use super::*;
use crate::config::Config;
use crate::types::{
    ArbitrageOpportunity, Exchange, HealthStatus, Metrics, PriceLevel, PriceLevelUpdate, Summary,
    TradingPair,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::timeout;

#[tokio::test]
async fn test_aggregator_new_and_subscriptions() {
    let config = Config::default();
    let aggregator = Aggregator::new(config);
    // Test that subscriptions do not panic
    let _ = aggregator.subscribe_summaries();
    let _ = aggregator.subscribe_arbitrage();
    let _ = aggregator.subscribe_shutdown();
}

#[tokio::test]
async fn test_aggregator_start_and_stop() {
    let config = Config::default();
    let aggregator = Aggregator::new(config);
    let handles = aggregator.start().await;
    assert!(handles.is_ok());
    let stop_result = aggregator.stop().await;
    assert!(stop_result.is_ok());
}

#[tokio::test]
async fn test_summary_and_health_metrics_accessors() {
    let config = Config::default();
    let aggregator = Aggregator::new(config);
    let pair = TradingPair::new("BTCUSDT", "USDT");
    let exchange = Exchange::Binance;
    // Initially, all should be empty
    assert!(aggregator.get_summary(&pair).await.is_none());
    assert!(aggregator.get_health_status(&exchange).await.is_none());
    assert!(aggregator.get_metrics(&exchange).await.is_none());
    assert!(aggregator.get_all_summaries().await.is_empty());
    assert!(aggregator.get_all_health_statuses().await.is_empty());
    assert!(aggregator.get_all_metrics().await.is_empty());
}

#[tokio::test]
async fn test_initialize_health_status() {
    let config = Config::default();
    let aggregator = Aggregator::new(config);
    aggregator.initialize_health_status().await.unwrap();
    let health_statuses = aggregator.get_all_health_statuses().await;
    for exchange in Exchange::all() {
        assert!(health_statuses.contains_key(&exchange));
        let status = health_statuses.get(&exchange).unwrap();
        assert!(!status.is_healthy);
        assert!(status.error_message.is_none());
    }
}

#[tokio::test]
async fn test_process_price_level_update_and_summary_broadcast() {
    let config = Config::default();
    let aggregator = Aggregator::new(config);
    let summary_sender = aggregator.summary_sender.clone();
    let price_level_update = PriceLevelUpdate {
        id: uuid::Uuid::new_v4(),
        symbol: "BTCUSDT".to_string(),
        exchange: Exchange::Binance,
        bids: vec![PriceLevel {
            price: 100.0,
            quantity: 1.0,
            exchange: Exchange::Binance,
            timestamp: chrono::Utc::now(),
        }],
        asks: vec![PriceLevel {
            price: 101.0,
            quantity: 1.0,
            exchange: Exchange::Binance,
            timestamp: chrono::Utc::now(),
        }],
        timestamp: chrono::Utc::now(),
    };
    let result = Aggregator::process_price_level_update(price_level_update, &summary_sender).await;
    assert!(result.is_ok());
    // Check that a summary was broadcast
    let mut rx = summary_sender.subscribe();
    let summary = timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    assert!(summary.is_ok());
}

#[tokio::test]
async fn test_arbitrage_detector_no_opportunity() {
    let config = Config::default();
    let aggregator = Aggregator::new(config);
    let pair = TradingPair::new("BTCUSDT", "USDT");
    let summary = Summary {
        symbol: "BTCUSDT".to_string(),
        spread: 0.0,
        bids: vec![],
        asks: vec![],
        timestamp: chrono::Utc::now(),
    };
    let result = Aggregator::detect_arbitrage_opportunity(&pair, &summary).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_health_monitor_marks_unhealthy() {
    let config = Config::default();
    let aggregator = Aggregator::new(config);
    aggregator.initialize_health_status().await.unwrap();
    // Simulate old last_update
    let mut health_status = aggregator.health_status.write().await;
    for status in health_status.values_mut() {
        status.last_update = chrono::Utc::now() - chrono::Duration::seconds(31);
    }
    drop(health_status);
    // Run health monitor once
    let health_status = aggregator.health_status.clone();
    let mut shutdown_rx = aggregator.shutdown_sender.subscribe();
    let handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(10));
        tokio::select! {
            _ = interval.tick() => {
                let mut health_map = health_status.write().await;
                let now = chrono::Utc::now();
                for (_exchange, status) in health_map.iter_mut() {
                    let time_since_update = now - status.last_update;
                    if time_since_update.num_seconds() > 30 {
                        status.is_healthy = false;
                        if status.error_message.is_none() {
                            status.error_message = Some("No recent updates".to_string());
                        }
                    }
                }
            }
            _ = shutdown_rx.recv() => {}
        }
    });
    handle.await.unwrap();
    let health_statuses = aggregator.get_all_health_statuses().await;
    for status in health_statuses.values() {
        assert!(!status.is_healthy);
        assert_eq!(status.error_message.as_deref(), Some("No recent updates"));
    }
}
