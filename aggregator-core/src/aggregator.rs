use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::types::{
    ArbitrageOpportunity, Exchange, HealthStatus, Metrics, PriceLevelUpdate, Summary, TradingPair,
};
use crate::{AggregatorError, Result};

pub struct Aggregator {
    config: Arc<Config>,
    summaries: Arc<RwLock<HashMap<TradingPair, Summary>>>,
    health_status: Arc<RwLock<HashMap<Exchange, HealthStatus>>>,
    metrics: Arc<RwLock<HashMap<Exchange, Metrics>>>,
    summary_sender: broadcast::Sender<Summary>,
    arbitrage_sender: broadcast::Sender<ArbitrageOpportunity>,
    shutdown_sender: broadcast::Sender<()>,
}

impl Aggregator {
    pub fn new(config: Config) -> Self {
        let (summary_sender, _) = broadcast::channel(1000);
        let (arbitrage_sender, _) = broadcast::channel(1000);
        let (shutdown_sender, _) = broadcast::channel(1);

        Self {
            config: Arc::new(config),
            summaries: Arc::new(RwLock::new(HashMap::new())),
            health_status: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            summary_sender,
            arbitrage_sender,
            shutdown_sender,
        }
    }

    pub fn subscribe_summaries(&self) -> broadcast::Receiver<Summary> {
        self.summary_sender.subscribe()
    }

    pub fn subscribe_arbitrage(&self) -> broadcast::Receiver<ArbitrageOpportunity> {
        self.arbitrage_sender.subscribe()
    }

    pub fn subscribe_shutdown(&self) -> broadcast::Receiver<()> {
        self.shutdown_sender.subscribe()
    }

    pub async fn start(&self) -> Result<Vec<JoinHandle<Result<()>>>> {
        info!("Starting cryptocurrency orderbook aggregator");

        let mut handles = Vec::new();

        self.initialize_health_status().await?;

        for exchange in self.config.enabled_exchanges() {
            let exchange_handles = self.start_exchange_connector(exchange).await?;
            handles.extend(exchange_handles);
        }

        let aggregation_handle = self.start_aggregation_processor().await?;
        handles.push(aggregation_handle);

        let arbitrage_handle = self.start_arbitrage_detector().await?;
        handles.push(arbitrage_handle);

        let health_handle = self.start_health_monitor().await?;
        handles.push(health_handle);

        info!("Aggregator started successfully");
        Ok(handles)
    }

    pub async fn stop(&self) -> Result<()> {
        info!("Stopping aggregator");
        self.shutdown_sender
            .send(())
            .map_err(|e| AggregatorError::ChannelSend {
                message: format!("Failed to send shutdown signal: {}", e),
            })?;
        Ok(())
    }

    pub async fn get_summary(&self, pair: &TradingPair) -> Option<Summary> {
        let summaries = self.summaries.read().await;
        summaries.get(pair).cloned()
    }

    pub async fn get_all_summaries(&self) -> HashMap<TradingPair, Summary> {
        let summaries = self.summaries.read().await;
        summaries.clone()
    }

    pub async fn get_health_status(&self, exchange: &Exchange) -> Option<HealthStatus> {
        let health_status = self.health_status.read().await;
        health_status.get(exchange).cloned()
    }

    pub async fn get_all_health_statuses(&self) -> HashMap<Exchange, HealthStatus> {
        let health_status = self.health_status.read().await;
        health_status.clone()
    }

    pub async fn get_metrics(&self, exchange: &Exchange) -> Option<Metrics> {
        let metrics = self.metrics.read().await;
        metrics.get(exchange).cloned()
    }

    pub async fn get_all_metrics(&self) -> HashMap<Exchange, Metrics> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    async fn initialize_health_status(&self) -> Result<()> {
        let mut health_status = self.health_status.write().await;

        for exchange in Exchange::all() {
            health_status.insert(
                exchange.clone(),
                HealthStatus {
                    exchange: exchange.clone(),
                    is_healthy: false,
                    last_update: chrono::Utc::now(),
                    error_message: None,
                },
            );
        }

        Ok(())
    }

    async fn start_exchange_connector(
        &self,
        exchange: Exchange,
    ) -> Result<Vec<JoinHandle<Result<()>>>> {
        info!("Starting exchange connector for {}", exchange);

        let (price_level_tx, price_level_rx) = mpsc::channel(10000);
        let mut handles = Vec::new();

        let processor_handle = self
            .start_price_level_processor(exchange.clone(), price_level_rx)
            .await?;
        handles.push(processor_handle);

        match exchange {
            Exchange::Binance => {
                // Use the actual Binance connector implementation
                // For now, we'll create a placeholder that demonstrates the pattern
                let handle = tokio::spawn(async move {
                    info!("Binance connector started");
                    // This would use exchange_connectors::Binance::new().spawn_order_book_service()
                    // with the trading pairs from config
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        // Simulate price level updates
                        let update = PriceLevelUpdate {
                            id: uuid::Uuid::new_v4(),
                            symbol: "BTCUSDT".to_string(),
                            exchange: Exchange::Binance,
                            bids: vec![],
                            asks: vec![],
                            timestamp: chrono::Utc::now(),
                        };
                        if price_level_tx.send(update).await.is_err() {
                            break;
                        }
                    }
                    Ok(())
                });
                handles.push(handle);
            }
            Exchange::Bybit => {
                let handle = tokio::spawn(async move {
                    info!("Bybit connector started");
                    // This would use exchange_connectors::Bybit::new().spawn_order_book_service()
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        let update = PriceLevelUpdate {
                            id: uuid::Uuid::new_v4(),
                            symbol: "BTCUSDT".to_string(),
                            exchange: Exchange::Bybit,
                            bids: vec![],
                            asks: vec![],
                            timestamp: chrono::Utc::now(),
                        };
                        if price_level_tx.send(update).await.is_err() {
                            break;
                        }
                    }
                    Ok(())
                });
                handles.push(handle);
            }
            Exchange::Kraken => {
                let handle = tokio::spawn(async move {
                    info!("Kraken connector started");
                    // This would use exchange_connectors::Kraken::new().spawn_order_book_service()
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        let update = PriceLevelUpdate {
                            id: uuid::Uuid::new_v4(),
                            symbol: "BTCUSDT".to_string(),
                            exchange: Exchange::Kraken,
                            bids: vec![],
                            asks: vec![],
                            timestamp: chrono::Utc::now(),
                        };
                        if price_level_tx.send(update).await.is_err() {
                            break;
                        }
                    }
                    Ok(())
                });
                handles.push(handle);
            }
            _ => {
                warn!("Exchange connector not implemented for {}", exchange);
            }
        }

        Ok(handles)
    }

    async fn start_price_level_processor(
        &self,
        exchange: Exchange,
        mut price_level_rx: mpsc::Receiver<PriceLevelUpdate>,
    ) -> Result<JoinHandle<Result<()>>> {
        let summary_sender = self.summary_sender.clone();
        let health_status = self.health_status.clone();
        let metrics = self.metrics.clone();
        let mut shutdown_rx = self.shutdown_sender.subscribe();

        let handle = tokio::spawn(async move {
            let mut last_update = chrono::Utc::now();
            let mut update_count = 0u64;

            loop {
                tokio::select! {
                    Some(update) = price_level_rx.recv() => {
                        // Process price level update
                        match Self::process_price_level_update(update, &summary_sender).await {
                            Ok(_) => {
                                update_count += 1;
                                last_update = chrono::Utc::now();

                                // Update health status
                                let mut health = health_status.write().await;
                                if let Some(status) = health.get_mut(&exchange) {
                                    status.is_healthy = true;
                                    status.last_update = last_update;
                                    status.error_message = None;
                                }

                                // Update metrics
                                let mut metrics_map = metrics.write().await;
                                if let Some(metric) = metrics_map.get_mut(&exchange) {
                                    metric.updates_per_second = update_count as f64 / last_update.timestamp() as f64;
                                    metric.last_update = last_update;
                                }
                            }
                            Err(e) => {
                                error!("Failed to process price level update: {}", e);

                                // Update health status with error
                                let mut health = health_status.write().await;
                                if let Some(status) = health.get_mut(&exchange) {
                                    status.is_healthy = false;
                                    status.error_message = Some(e.to_string());
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Price level processor for {} shutting down", exchange);
                        break;
                    }
                }
            }

            Ok(())
        });

        Ok(handle)
    }

    async fn process_price_level_update(
        update: PriceLevelUpdate,
        summary_sender: &broadcast::Sender<Summary>,
    ) -> Result<()> {
        // Create a summary from the price level update
        let mut bids = Vec::new();
        let mut asks = Vec::new();

        for bid in update.bids {
            bids.push(crate::types::PriceLevel {
                price: bid.price,
                quantity: bid.quantity,
                exchange: bid.exchange,
                timestamp: bid.timestamp,
            });
        }

        for ask in update.asks {
            asks.push(crate::types::PriceLevel {
                price: ask.price,
                quantity: ask.quantity,
                exchange: ask.exchange,
                timestamp: ask.timestamp,
            });
        }

        let spread = if let (Some(best_bid), Some(best_ask)) = (bids.first(), asks.first()) {
            best_ask.price - best_bid.price
        } else {
            0.0
        };

        let summary = Summary {
            symbol: update.symbol,
            spread,
            bids,
            asks,
            timestamp: update.timestamp,
        };

        summary_sender
            .send(summary)
            .map_err(|e| AggregatorError::ChannelSend {
                message: format!("Failed to send summary: {}", e),
            })?;

        Ok(())
    }

    async fn start_aggregation_processor(&self) -> Result<JoinHandle<Result<()>>> {
        let summaries = self.summaries.clone();
        let mut summary_rx = self.summary_sender.subscribe();
        let mut shutdown_rx = self.shutdown_sender.subscribe();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(summary) = summary_rx.recv() => {
                        // Update summaries map
                        let pair = TradingPair::new(&summary.symbol, "USDT"); // Simplified
                        let mut summaries_map = summaries.write().await;
                        summaries_map.insert(pair, summary);
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Aggregation processor shutting down");
                        break;
                    }
                }
            }
            Ok(())
        });

        Ok(handle)
    }

    async fn start_arbitrage_detector(&self) -> Result<JoinHandle<Result<()>>> {
        let arbitrage_sender = self.arbitrage_sender.clone();
        let summaries = self.summaries.clone();
        let mut shutdown_rx = self.shutdown_sender.subscribe();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Check for arbitrage opportunities
                        let summaries_map = summaries.read().await;

                        // Simple arbitrage detection logic
                        for (pair, summary) in summaries_map.iter() {
                            if let Some(opportunity) = Self::detect_arbitrage_opportunity(pair, summary).await {
                                if let Err(e) = arbitrage_sender.send(opportunity) {
                                    error!("Failed to send arbitrage opportunity: {}", e);
                                }
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Arbitrage detector shutting down");
                        break;
                    }
                }
            }
            Ok(())
        });

        Ok(handle)
    }

    async fn detect_arbitrage_opportunity(
        _pair: &TradingPair,
        _summary: &Summary,
    ) -> Option<ArbitrageOpportunity> {
        // TODO: Implement actual arbitrage detection logic
        // This would compare prices across exchanges and identify opportunities
        None
    }

    async fn start_health_monitor(&self) -> Result<JoinHandle<Result<()>>> {
        let health_status = self.health_status.clone();
        let mut shutdown_rx = self.shutdown_sender.subscribe();

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Check health status of all exchanges
                        let mut health_map = health_status.write().await;
                        let now = chrono::Utc::now();

                        for (exchange, status) in health_map.iter_mut() {
                            let time_since_update = now - status.last_update;

                            // Mark as unhealthy if no updates for more than 30 seconds
                            if time_since_update.num_seconds() > 30 {
                                status.is_healthy = false;
                                if status.error_message.is_none() {
                                    status.error_message = Some("No recent updates".to_string());
                                }
                                warn!("Exchange {} marked as unhealthy", exchange);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Health monitor shutting down");
                        break;
                    }
                }
            }
            Ok(())
        });

        Ok(handle)
    }
}
